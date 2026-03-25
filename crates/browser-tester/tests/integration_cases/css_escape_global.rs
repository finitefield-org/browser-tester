use browser_tester::Harness;

#[test]
fn css_escape_is_available_on_the_global_css_object() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        document.getElementById("out").textContent = [
          CSS.escape("0"),
          CSS.escape("alpha-beta")
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "\\30 |alpha-beta")?;
    Ok(())
}
