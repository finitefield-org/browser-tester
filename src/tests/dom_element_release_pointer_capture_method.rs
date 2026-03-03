use super::*;

#[test]
fn element_release_pointer_capture_releases_owned_capture_and_returns_undefined() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            target.setPointerCapture(21);
            const before = target.hasPointerCapture(21);
            const releaseReturn = target.releasePointerCapture(21);
            const after = target.hasPointerCapture(21);
            document.getElementById('result').textContent = [
              before,
              releaseReturn === undefined,
              after
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn element_release_pointer_capture_does_not_release_capture_owned_by_other_element() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <div id='other'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const other = document.getElementById('other');
            target.setPointerCapture(7);
            const releaseReturn = other.releasePointerCapture(7);
            document.getElementById('result').textContent = [
              releaseReturn === undefined,
              target.hasPointerCapture(7),
              other.hasPointerCapture(7)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn element_release_pointer_capture_throws_not_found_error_when_pointer_is_not_active() -> Result<()>
{
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            let notFound = false;
            try {
              target.releasePointerCapture(999);
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
fn element_release_pointer_capture_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            target.releasePointerCapture();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("releasePointerCapture requires exactly one pointerId argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
