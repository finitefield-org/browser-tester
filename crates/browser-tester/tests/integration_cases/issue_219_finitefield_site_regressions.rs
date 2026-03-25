use browser_tester::Harness;

#[test]
fn issue_219_margin_markup_page_loads_and_runs_bulk_flow_on_small_test_thread()
-> browser_tester::Result<()> {
    let html = r#"
      <button id="go" type="button">Go</button>
      <div id="out"></div>
      <script>
        document.getElementById("go").addEventListener("click", () => {
          const rows = [
            ["Field 1", "North Block", "Cabbage"],
            ["Field 2", "South Block", "Tomato"],
            ["Field 3", "West Block", "Pepper"]
          ];
          document.getElementById("out").textContent = String(rows.length);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "3")?;
    Ok(())
}
