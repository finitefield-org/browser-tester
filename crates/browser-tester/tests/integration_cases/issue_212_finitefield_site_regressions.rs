use browser_tester::Harness;

#[test]
fn issue_212_typeof_window_history_replace_state_member_reference() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        document.getElementById("out").textContent =
          typeof window.history.replaceState === "function" ? "ok" : "blocked";
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "ok")?;
    Ok(())
}

#[test]
fn issue_212_typeof_window_location_replace_member_reference() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        document.getElementById("out").textContent =
          typeof window.location.replace === "function" ? "ok" : "blocked";
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "ok")?;
    Ok(())
}
