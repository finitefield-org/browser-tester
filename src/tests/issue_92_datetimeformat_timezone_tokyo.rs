use super::*;

#[test]
fn issue_92_datetimeformat_accepts_asia_tokyo_timezone_option() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <p id='err'></p>
      <script>
        try {
          const dtf = new Intl.DateTimeFormat('en-GB', {
            timeZone: 'Asia/Tokyo',
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
            hour12: false,
          });
          document.getElementById('out').textContent =
            dtf.format(new Date(1700000000000));
        } catch (error) {
          document.getElementById('err').textContent =
            String(error && error.message ? error.message : error);
        }
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "15/11/2023, 07:13:20")?;
    harness.assert_text("#err", "")?;
    Ok(())
}
