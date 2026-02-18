use browser_tester::Harness;

#[test]
fn ignores_json_ld_script_blocks_and_runs_executable_script() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result">init</div>
    <script type="application/ld+json">
      {"@context":"https://schema.org","@type":"FAQPage"}
    </script>
    <script>
      document.getElementById("result").textContent = "ok";
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn script_end_extractor_handles_regex_literals_with_quotes() -> browser_tester::Result<()> {
    let html = r##"
    <div id="result">init</div>
    <script>
      const sanitizer = /["]/g;
      document.getElementById("result").textContent = "ok";
    </script>
    "##;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}
