use super::*;

#[test]
fn issue_100_array_literal_spread_over_array_expression_filter_result() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <p id='err'></p>
      <script>
        try {
          const value = 'x';
          const history = ['a', 'b', 'x'];
          const next = [value, ...history.filter((item) => item !== value)];
          document.getElementById('out').textContent = next.join(',');
        } catch (error) {
          document.getElementById('err').textContent =
            String(error && error.message ? error.message : error);
        }
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "x,a,b")?;
    harness.assert_text("#err", "")?;
    Ok(())
}
