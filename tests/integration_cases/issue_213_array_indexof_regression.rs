use browser_tester::Harness;

#[test]
fn issue_213_array_index_of_and_last_index_of_match_standard_array_search()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const values = ["alpha", "beta", "gamma", "beta"];
        document.getElementById("out").textContent = [
          String(values.indexOf("beta")),
          String(values.indexOf("beta", 2)),
          String(values.indexOf("beta", -2)),
          String(values.lastIndexOf("beta")),
          String(values.lastIndexOf("beta", 2)),
          String(values.lastIndexOf("beta", -3))
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|3|3|3|1|1")?;
    Ok(())
}
