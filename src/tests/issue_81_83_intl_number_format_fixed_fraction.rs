use super::*;

#[test]
fn issue_81_minimum_fraction_digits_is_honored_for_problematic_values() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const f = new Intl.NumberFormat('en', {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        });
        const a = f.format(28.000000000000004);
        const b = f.format(43.55555555555556);
        document.getElementById('out').textContent = `${a}|${b}`;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "28.00|43.56")?;
    Ok(())
}

#[test]
fn issue_83_fixed_fraction_formatting_remains_stable_for_construction_examples() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const f = new Intl.NumberFormat('en', {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        });
        const values = [1.5, 0.226194671, 13.015];
        document.getElementById('out').textContent =
          values.map((v) => `${f.format(v)} m³`).join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1.50 m³|0.23 m³|13.02 m³")?;
    Ok(())
}
