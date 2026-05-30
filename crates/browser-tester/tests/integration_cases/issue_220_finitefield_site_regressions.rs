use browser_tester::Harness;

#[test]
fn issue_220_intl_number_format_alias_keeps_fraction_digits() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const NumberFormat = Intl.NumberFormat;
        document.getElementById("out").textContent = new NumberFormat("en-US", {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        }).format(1234.5);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1,234.50")?;
    Ok(())
}
