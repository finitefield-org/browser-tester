use super::*;

#[test]
fn issue_106_number_format_to_parts_preserves_string_and_bigint_precision() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const nf = new Intl.NumberFormat('en-US');
        const join = (parts) => parts.map((p) => p.value).join('');

        const partsBigInt = nf.formatToParts(1234567891234567891n);
        const partsString = nf.formatToParts('1234567891234567891');
        const partsNumber = nf.formatToParts(1234567891234567891);
        const partsExponent = nf.formatToParts('1000000000000000110000E-6');

        const exponentFraction = (partsExponent.find((p) => p.type === 'fraction') || {}).value || '';

        const checks = [
          join(partsBigInt) === '1,234,567,891,234,567,891',
          join(partsString) === join(partsBigInt),
          join(partsNumber) !== join(partsBigInt),
          partsBigInt.some((p) => p.type === 'group'),
          join(partsExponent) === '1,000,000,000,000,000.11',
          exponentFraction === '11'
        ];

        document.getElementById('out').textContent = checks.join(':');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true:true:true:true:true:true")?;
    Ok(())
}

#[test]
fn issue_106_number_format_to_parts_currency_tokens_match_formatted_output() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const nf = new Intl.NumberFormat('en-US', { style: 'currency', currency: 'USD' });
        const parts = nf.formatToParts('654321.987');
        const rendered = parts.map((p) => p.value).join('');
        const json = JSON.stringify(parts);
        const checks = [
          rendered === '$654,321.99',
          json.includes('\"type\":\"currency\"'),
          json.includes('\"type\":\"group\"'),
          json.includes('\"type\":\"decimal\"'),
          json.includes('\"type\":\"fraction\",\"value\":\"99\"')
        ];
        document.getElementById('out').textContent = checks.join(':');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true:true:true:true:true")?;
    Ok(())
}
