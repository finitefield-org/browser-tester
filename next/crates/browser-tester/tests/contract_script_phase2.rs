use browser_tester_next::Harness;

#[test]
fn from_html_bootstraps_inline_scripts_and_mutates_dom() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('out').textContent = 'Hello';</script>",
    )?;

    harness.assert_exists("#out")?;
    assert_eq!(
        harness.debug().dump_dom(),
        "#document\n  <main id=\"out\">\n    \"Hello\"\n  </main>\n  <script>\n    \"document.getElementById('out').textContent = 'Hello';\"\n  </script>"
    );
    Ok(())
}

#[test]
fn missing_element_access_reports_a_script_error() {
    let error = Harness::from_html(
        "<main id='out'></main><script>document.getElementById('missing').textContent = 'Hello';</script>",
    )
    .expect_err("missing elements should fail script bootstrap");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains(
        "document.getElementById(\"missing\") returned no element"
    ));
}

#[test]
fn unsupported_script_syntax_reports_explicitly() {
    let error = Harness::from_html(
        "<main id='out'></main><script>document.querySelector('#out').textContent = 'Hello';</script>",
    )
    .expect_err("unsupported syntax should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported Document method: querySelector"));
}
