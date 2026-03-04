use super::*;

#[test]
fn issue_105_intl_number_format_format_preserves_bigint_and_string_precision() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const nf = new Intl.NumberFormat('en-US');
        const numberValue = nf.format(1234567891234567891);
        const bigintValue = nf.format(1234567891234567891n);
        const stringValue = nf.format('1234567891234567891');
        const exponentStringValue = nf.format('1000000000000000110000E-6');
        const checks = [
          numberValue !== bigintValue,
          bigintValue === stringValue,
          exponentStringValue === '1,000,000,000,000,000.11'
        ];
        document.getElementById('out').textContent =
          checks.join(':') + '|' + [bigintValue, stringValue, exponentStringValue].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "true:true:true|1,234,567,891,234,567,891|1,234,567,891,234,567,891|1,000,000,000,000,000.11",
    )?;
    Ok(())
}

#[test]
fn issue_105_intl_number_format_format_getter_is_bound_and_currency_string_is_exact() -> Result<()>
{
    let html = r#"
      <p id='out'></p>
      <script>
        const values = [123456.789, 987654.321, 456789.123];
        const nf = new Intl.NumberFormat('en-US');
        const format = nf.format;
        const mapped = values.map((n) => format(n)).join(';');
        const usd = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' });
        const exactCurrency = usd.format('987654321987654321');
        document.getElementById('out').textContent = `${mapped}|${exactCurrency}`;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "123,456.789;987,654.321;456,789.123|$987,654,321,987,654,321.00",
    )?;
    Ok(())
}
