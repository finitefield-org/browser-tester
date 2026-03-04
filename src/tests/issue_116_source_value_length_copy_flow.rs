use super::*;

#[test]
fn issue_116_source_value_length_on_plain_object_is_not_treated_as_dom_target() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const source = { value: 'hello' };
        document.getElementById('out').textContent = String(source.value.length);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "5")?;
    Ok(())
}

#[test]
fn issue_116_copy_share_flow_with_source_object_works_for_click_and_enter() -> Result<()> {
    let html = r#"
      <button id='copy' type='button'>copy</button>
      <p id='out'></p>
      <script>
        const state = {
          selected: 0,
          sources: [
            { kind: 'unix', value: '1700000000' }
          ],
        };

        async function copyCurrent() {
          const source = state.sources[state.selected];
          await navigator.clipboard.writeText(source.value);
          document.getElementById('out').textContent =
            source.kind + ':' + source.value.length;
        }

        const copyButton = document.getElementById('copy');
        copyButton.addEventListener('click', async () => {
          try {
            await copyCurrent();
          } catch (error) {
            document.getElementById('out').textContent =
              'err:' + String(error && error.message ? error.message : error);
          }
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#copy")?;
    harness.assert_text("#out", "unix:10")?;

    harness.dispatch("#copy", "click")?;
    harness.assert_text("#out", "unix:10")?;

    harness.press_enter("#copy")?;
    harness.assert_text("#out", "unix:10")?;
    Ok(())
}
