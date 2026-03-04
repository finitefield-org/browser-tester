use super::*;

#[test]
fn issue_107_datetime_format_to_parts_matches_expected_tokens() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const date = Date.UTC(2012, 11, 17, 3, 0, 42);
        const formatter = new Intl.DateTimeFormat('en-US', {
          weekday: 'long',
          year: 'numeric',
          month: 'numeric',
          day: 'numeric',
          hour: 'numeric',
          minute: 'numeric',
          second: 'numeric',
          fractionalSecondDigits: 3,
          hour12: true,
          timeZone: 'UTC',
        });

        const parts = formatter.formatToParts(date);
        const rendered = parts.map((part) => part.value).join('');
        const types = parts.map((part) => part.type).join(',');
        const checks = [
          rendered === 'Monday, 12/17/2012, 3:00:42.000 AM',
          types === 'weekday,literal,month,literal,day,literal,year,literal,hour,literal,minute,literal,second,literal,fractionalSecond,literal,dayPeriod',
          parts.some((part) => part.type === 'fractionalSecond' && part.value === '000'),
          parts.some((part) => part.type === 'dayPeriod' && part.value === 'AM'),
        ];

        document.getElementById('out').textContent = checks.join(':');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true:true:true:true")?;
    Ok(())
}

#[test]
fn issue_107_datetime_format_to_parts_uses_now_for_undefined_and_rejects_invalid_string()
-> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const formatter = new Intl.DateTimeFormat('en-US', { timeZone: 'UTC' });
        const formattedNow = formatter.format();
        const fromUndefined = formatter.formatToParts(undefined).map((part) => part.value).join('');

        let raisedRangeError = false;
        try {
          formatter.formatToParts('2012-12-20');
        } catch (e) {
          raisedRangeError = String(e).includes('RangeError');
        }

        document.getElementById('out').textContent =
          [fromUndefined === formattedNow, raisedRangeError].join(':');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true:true")?;
    Ok(())
}
