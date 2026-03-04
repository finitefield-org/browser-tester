use super::*;

#[test]
fn issue_112_structured_throw_in_promise_chain_preserves_code_in_async_catch() -> Result<()> {
    let html = r#"
      <button id='run' type='button'>run</button>
      <p id='out'></p>
      <script>
        function validate(text) {
          if (text.includes('!')) {
            throw { code: 'invalid_char' };
          }
          return text;
        }

        document.getElementById('run').addEventListener('click', async () => {
          try {
            await Promise.resolve().then(() => validate('A!'));
            document.getElementById('out').textContent = 'ok';
          } catch (error) {
            const code = error && error.code ? error.code : '';
            document.getElementById('out').textContent = code || 'generic';
          }
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "invalid_char")?;
    Ok(())
}

#[test]
fn issue_112_structured_rejection_reason_preserves_code_in_async_catch() -> Result<()> {
    let html = r#"
      <button id='run' type='button'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', async () => {
          try {
            await Promise.reject({ code: 'missing_padding' });
            document.getElementById('out').textContent = 'ok';
          } catch (error) {
            const code = error && error.code ? error.code : '';
            document.getElementById('out').textContent = code || 'generic';
          }
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "missing_padding")?;
    Ok(())
}
