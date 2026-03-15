use super::*;

fn find_function<'a>(stmts: &'a [Stmt], target: &str) -> &'a ScriptHandler {
    stmts
        .iter()
        .find_map(|stmt| match stmt {
            Stmt::FunctionDecl { name, handler, .. } if name == target => Some(handler),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing function declaration: {target}"))
}

#[test]
fn nested_function_scope_bindings_keep_caller_local_close() -> Result<()> {
    let stmts = test_support::parse_block_statements(
        r#"
          function make(source) {
            let index = 0;

            function current() {
              return source[index] || "";
            }

            function consume() {
              const char = source[index] || "";
              index += 1;
              return char;
            }

            function parseSequence(stopChar) {
              let seen = "";
              while (index < source.length && current() !== stopChar) {
                seen += consume();
              }
              return "seen=" + seen + "|stop=" + stopChar + "|curr=" + (current() || "<eof>") + "|index=" + index;
            }

            function parseBracketGroup() {
              const open = consume();
              const close = open === "(" ? ")" : "]";
              const inner = parseSequence(close);
              return "after=" + (current() || "<eof>") + "|close=" + close + "|index=" + index + "|" + inner;
            }

            return parseBracketGroup();
          }
        "#,
    )?;

    let make = find_function(&stmts, "make");
    let parse_bracket_group = find_function(&make.stmts, "parseBracketGroup");
    let bindings = Harness::collect_function_scope_bindings(parse_bracket_group);

    assert!(
        bindings.contains("open"),
        "missing open binding: {bindings:?}"
    );
    assert!(
        bindings.contains("close"),
        "missing close binding: {bindings:?}"
    );
    assert!(
        bindings.contains("inner"),
        "missing inner binding: {bindings:?}"
    );
    Ok(())
}
