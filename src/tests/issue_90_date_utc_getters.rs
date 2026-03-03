use super::*;

#[test]
fn issue_90_date_utc_getters_are_available_and_return_expected_values() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const d = new Date(1700000000000);
        document.getElementById('out').textContent = [
          d.getUTCFullYear(),
          d.getUTCMonth(),
          d.getUTCDate(),
          d.getUTCHours(),
          d.getUTCMinutes(),
          d.getUTCSeconds()
        ].join(':');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "2023:10:14:22:13:20")?;
    Ok(())
}
