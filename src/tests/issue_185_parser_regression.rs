use super::*;

#[test]
fn inline_object_literal_computed_lookup_remains_member_access_in_ast() -> Result<()> {
    let expr = test_support::parse_expr(
        r#"
          {
            checker: "background:#f8fafc;",
            dark: "background:#0f172a;"
          }["dark"] || "background:#ffffff;"
        "#,
    )?;

    let debug = format!("{expr:#?}");
    assert!(debug.contains("op: Or"), "expected logical-or, got {debug}");
    assert!(
        debug.contains("member: \"dark\""),
        "expected computed lookup member, got {debug}"
    );
    assert!(
        debug.contains("ObjectLiteral"),
        "expected inline object literal target, got {debug}"
    );
    Ok(())
}

#[test]
fn const_decl_with_inline_object_literal_lookup_stays_single_statement() -> Result<()> {
    let stmts = test_support::parse_block_statements(
        r#"
          const backgroundCss = {
            checker: "background:#f8fafc;",
            dark: "background:#0f172a;"
          }["dark"] || "background:#ffffff;";
        "#,
    )?;

    assert_eq!(stmts.len(), 1, "unexpected statement split: {stmts:#?}");
    let debug = format!("{:#?}", &stmts[0]);
    assert!(debug.contains("name: \"backgroundCss\""));
    assert!(debug.contains("member: \"dark\""));
    assert!(debug.contains("ObjectLiteral"));
    Ok(())
}
