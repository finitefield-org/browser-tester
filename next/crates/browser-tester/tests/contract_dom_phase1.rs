use browser_tester_next::Harness;

#[test]
fn from_html_builds_dom_and_supports_phase_one_selectors() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='app'><span data-state='ready'>Hello</span><input disabled></main>",
    )?;

    harness.assert_exists("#app")?;
    harness.assert_exists("main")?;
    harness.assert_exists("[data-state]")?;

    assert_eq!(
        harness.debug().dump_dom(),
        "#document\n  <main id=\"app\">\n    <span data-state=\"ready\">\n      \"Hello\"\n    </span>\n    <input disabled />\n  </main>"
    );
    Ok(())
}

#[test]
fn assert_exists_reports_missing_nodes_with_dom_dump() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main id='app'></main>")?;

    let error = harness
        .assert_exists("#missing")
        .expect_err("missing nodes should fail");

    let message = error.to_string();
    assert!(message.contains("expected selector `#missing` to match at least one node"));
    assert!(message.contains("#document"));
    assert!(message.contains("<main id=\"app\" />"));
    Ok(())
}

#[test]
fn malformed_html_is_rejected_with_a_parse_error() {
    let error = Harness::from_html("<main><span></main>").expect_err("malformed HTML should fail");

    let message = error.to_string();
    assert!(message.contains("HTML parse error"));
    assert!(message.contains("mismatched closing tag"));
}

#[test]
fn unsupported_selector_syntax_is_reported_explicitly() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html("<main><span class='app'></span></main>")?;
    let error = harness
        .assert_exists("main ~ .app")
        .expect_err("general sibling combinators are not part of the selector slices");

    let message = error.to_string();
    assert!(message.contains("Selector error"));
    assert!(
        message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], descendant combinators like `A B`, adjacent sibling combinators like `A + B`, and child combinators like `A > B`")
    );
    Ok(())
}
