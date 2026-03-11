use super::*;

#[test]
fn nested_array_map_join_preserves_outer_newline_separator_in_ast() -> Result<()> {
    let expr = test_support::parse_expr(r#"rows.map(rowToLine).join("\n")"#)?;
    match expr {
        Expr::MemberCall {
            target,
            member,
            args,
            optional: false,
            optional_call: false,
        } => {
            assert_eq!(member, "join");
            assert_eq!(args, vec![Expr::String("\n".to_string())]);
            match *target {
                Expr::MemberCall {
                    member,
                    args,
                    optional: false,
                    optional_call: false,
                    ..
                } => {
                    assert_eq!(member, "map");
                    assert_eq!(args, vec![Expr::Var("rowToLine".to_string())]);
                }
                other => panic!("unexpected inner target: {other:?}"),
            }
        }
        other => panic!("unexpected expression: {other:?}"),
    }
    Ok(())
}
