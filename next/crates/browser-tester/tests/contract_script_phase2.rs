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
    assert!(message.contains("document.getElementById(\"missing\") returned no element"));
}

#[test]
fn inline_scripts_support_collection_for_each() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span class='item'>First</span><span class='item'>Second</span></main><div id='out'></div><script>const nodes = document.querySelectorAll('.item'); const children = document.getElementById('root').children; nodes.forEach((item, index, list) => { document.getElementById('out').textContent += 'N' + String(index) + ':' + item.textContent + ':' + String(list.length) + ';'; }, null); children.forEach((child, index, list) => { document.getElementById('out').textContent += 'H' + String(index) + ':' + child.textContent + ':' + String(list.length) + ';'; }, null);</script>",
    )?;

    harness.assert_text("#out", "N0:First:2;N1:Second:2;H0:First:2;H1:Second:2;")?;
    Ok(())
}
