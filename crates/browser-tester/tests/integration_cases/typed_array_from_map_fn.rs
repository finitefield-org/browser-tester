use browser_tester::Harness;

#[test]
fn typed_array_from_supports_map_function_argument() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const binary = "AZ";
        const bytes = Array.from(binary, (char) => char.charCodeAt(0));
        document.getElementById("out").textContent = Array.from(bytes).join(",");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "65,90")?;
    Ok(())
}
