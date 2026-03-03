use super::*;

#[test]
fn issue_72_non_intl_format_member_call_does_not_require_intl_instance() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          window.qrcode = {
            format: function(value) {
              return 'qr:' + value;
            }
          };
          const nf = new Intl.NumberFormat('en');
          document.getElementById('result').textContent = window.qrcode.format(nf.format(1234));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "qr:1,234")?;
    Ok(())
}

#[test]
fn issue_72_relative_time_format_with_two_args_still_works() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'always' });
          document.getElementById('result').textContent = rtf.format(-1, 'day');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1 day ago")?;
    Ok(())
}
