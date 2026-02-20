use browser_tester::Harness;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::TestCaseResult;

fn identifier_strategy() -> BoxedStrategy<String> {
    prop_oneof![
        Just("a"),
        Just("b"),
        Just("c"),
        Just("x"),
        Just("y"),
        Just("value"),
        Just("index"),
        Just("items"),
        Just("state"),
        Just("_tmp"),
    ]
    .prop_map(str::to_string)
    .boxed()
}

fn literal_strategy() -> BoxedStrategy<String> {
    prop_oneof![
        Just("undefined".to_string()),
        Just("null".to_string()),
        Just("true".to_string()),
        Just("false".to_string()),
        any::<i16>().prop_map(|v| v.to_string()),
        any::<u16>().prop_map(|v| v.to_string()),
        Just("'x'".to_string()),
        Just("'日本語'".to_string()),
        Just("\"double\"".to_string()),
        Just("`template`".to_string()),
    ]
    .boxed()
}

fn regex_literal_strategy() -> BoxedStrategy<String> {
    prop_oneof![
        Just("/a/".to_string()),
        Just("/\\d+/".to_string()),
        Just("/^\\w+$/".to_string()),
        Just("/foo(?=bar)/".to_string()),
        Just("/\\/(x|y)/".to_string()),
        Just("/[a-z]{1,3}/gi".to_string()),
    ]
    .boxed()
}

fn binary_operator_strategy() -> BoxedStrategy<&'static str> {
    prop_oneof![
        Just("+"),
        Just("-"),
        Just("*"),
        Just("/"),
        Just("%"),
        Just("&&"),
        Just("||"),
        Just("==="),
        Just("!=="),
        Just("<"),
        Just(">"),
        Just("<="),
        Just(">="),
    ]
    .boxed()
}

fn expression_strategy() -> BoxedStrategy<String> {
    let leaf = prop_oneof![
        identifier_strategy(),
        literal_strategy(),
        regex_literal_strategy(),
    ]
    .boxed();

    leaf.prop_recursive(4, 96, 8, |inner| {
        prop_oneof![
            inner.clone().prop_map(|expr| format!("({expr})")),
            inner.clone().prop_map(|expr| format!("!({expr})")),
            inner.clone().prop_map(|expr| format!("+({expr})")),
            inner.clone().prop_map(|expr| format!("-({expr})")),
            (inner.clone(), binary_operator_strategy(), inner.clone())
                .prop_map(|(lhs, op, rhs)| format!("({lhs} {op} {rhs})")),
            (inner.clone(), inner.clone(), inner.clone())
                .prop_map(|(cond, left, right)| format!("({cond} ? {left} : {right})")),
            vec(inner.clone(), 0..=3).prop_map(|items| format!("[{}]", items.join(", "))),
            (inner.clone(), inner.clone())
                .prop_map(|(left, right)| format!("{{ left: {left}, right: {right} }}")),
            (identifier_strategy(), vec(inner.clone(), 0..=3))
                .prop_map(|(name, args)| format!("{name}({})", args.join(", "))),
            (inner.clone(), inner.clone()).prop_map(|(target, index)| format!("{target}[{index}]")),
            (inner.clone(), inner.clone())
                .prop_map(|(target, key)| format!("{target}[String({key})]")),
        ]
    })
    .boxed()
}

fn simple_statement_strategy() -> BoxedStrategy<String> {
    let ident = identifier_strategy();
    let expr = expression_strategy();

    prop_oneof![
        (ident.clone(), expr.clone()).prop_map(|(name, value)| format!("let {name} = {value};")),
        (ident.clone(), expr.clone()).prop_map(|(name, value)| format!("const {name} = {value};")),
        (ident.clone(), expr.clone()).prop_map(|(name, value)| format!("{name} = {value};")),
        expr.clone().prop_map(|value| format!("{value};")),
        (expr.clone(), expr.clone()).prop_map(|(target, key)| format!(
            "Object.prototype.hasOwnProperty.call({target}, {key});"
        )),
    ]
    .boxed()
}

fn statement_strategy() -> BoxedStrategy<String> {
    let simple = simple_statement_strategy();

    simple
        .prop_recursive(4, 192, 8, |inner| {
            let expr = expression_strategy();
            let ident = identifier_strategy();

            prop_oneof![
                (
                    expr.clone(),
                    vec(inner.clone(), 1..=3),
                    vec(inner.clone(), 0..=2),
                )
                    .prop_map(|(cond, then_body, else_body)| {
                        if else_body.is_empty() {
                            format!("if ({cond}) {{ {} }}", then_body.join(" "))
                        } else {
                            format!(
                                "if ({cond}) {{ {} }} else {{ {} }}",
                                then_body.join(" "),
                                else_body.join(" ")
                            )
                        }
                    }),
                (ident.clone(), expr.clone(), expr.clone(), vec(inner.clone(), 1..=2))
                    .prop_map(|(name, start, end, body)| {
                        format!(
                            "for (let {name} = {start}; {name} < {end}; {name} = {name} + 1) {{ {} }}",
                            body.join(" ")
                        )
                    }),
                (expr.clone(), vec(inner.clone(), 1..=2)).prop_map(|(cond, body)| {
                    format!("while ({cond}) {{ {} break; }}", body.join(" "))
                }),
                (ident.clone(), vec(inner.clone(), 1..=3)).prop_map(|(name, body)| {
                    format!("function {name}(arg) {{ {} return arg; }}", body.join(" "))
                }),
                (vec(inner.clone(), 1..=2), vec(inner.clone(), 1..=2))
                    .prop_map(|(try_body, catch_body)| {
                        format!(
                            "try {{ {} }} catch (err) {{ {} }}",
                            try_body.join(" "),
                            catch_body.join(" ")
                        )
                    }),
            ]
        })
        .boxed()
}

fn callback_body_strategy() -> BoxedStrategy<String> {
    vec(statement_strategy(), 1..=10)
        .prop_map(|mut stmts| {
            stmts.push("return;".to_string());
            stmts.join("\n")
        })
        .boxed()
}

fn html_with_callback_body(callback_body: &str) -> String {
    format!(
        r#"
<button id="run">run</button>
<script>
document.getElementById("run").addEventListener("click", () => {{
{callback_body}
}});
</script>
"#
    )
}

fn assert_parser_path_never_panics(callback_body: &str) -> TestCaseResult {
    let html = html_with_callback_body(callback_body);
    let outcome = std::panic::catch_unwind(|| Harness::from_html(&html));
    prop_assert!(
        outcome.is_ok(),
        "Harness::from_html panicked for generated body:\n{callback_body}"
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: None,
        .. ProptestConfig::default()
    })]

    #[test]
    fn parser_generated_statement_blocks_do_not_panic(body in callback_body_strategy()) {
        assert_parser_path_never_panics(&body)?;
    }

    #[test]
    fn parser_generated_expression_combinations_do_not_panic(expr in expression_strategy()) {
        let body = format!(
            r#"
const seed = {expr};
const wrapped = [seed, {expr}];
const first = wrapped[0];
const fallback = first ? first : wrapped[1];
Object.prototype.hasOwnProperty.call({{}}, String(fallback));
return;
"#
        );
        assert_parser_path_never_panics(body.as_str())?;
    }
}
