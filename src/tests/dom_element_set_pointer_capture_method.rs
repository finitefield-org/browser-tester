use super::*;

#[test]
fn element_set_pointer_capture_sets_capture_target_and_returns_undefined() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <div id='other'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const other = document.getElementById('other');
            const before = target.hasPointerCapture(21);
            const setReturn = target.setPointerCapture(21);
            const ownAfterSet = target.hasPointerCapture(21);
            const otherAfterSet = other.hasPointerCapture(21);

            other.setPointerCapture(21);
            const ownAfterTransfer = target.hasPointerCapture(21);
            const otherAfterTransfer = other.hasPointerCapture(21);

            document.getElementById('result').textContent = [
              before,
              setReturn === undefined,
              ownAfterSet,
              otherAfterSet,
              ownAfterTransfer,
              otherAfterTransfer
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:true:false:false:true")?;
    Ok(())
}

#[test]
fn element_set_pointer_capture_throws_not_found_error_for_non_active_pointer() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            let notFound = false;
            try {
              target.setPointerCapture(0);
            } catch (e) {
              notFound = String(e).includes('NotFoundError');
            }
            document.getElementById('result').textContent = String(notFound);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn element_set_pointer_capture_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            target.setPointerCapture();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("setPointerCapture requires exactly one pointerId argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
