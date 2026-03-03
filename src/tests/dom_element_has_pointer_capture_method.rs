use super::*;

#[test]
fn element_has_pointer_capture_tracks_owner_per_pointer_id() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <div id='other'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const other = document.getElementById('other');

            const before = target.hasPointerCapture(7);
            const setReturn = target.setPointerCapture(7);
            const ownAfterSet = target.hasPointerCapture(7);
            const otherAfterSet = other.hasPointerCapture(7);

            other.setPointerCapture(7);
            const ownAfterTransfer = target.hasPointerCapture(7);
            const otherAfterTransfer = other.hasPointerCapture(7);

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
fn element_has_pointer_capture_reflects_release_pointer_capture_and_pointer_id_coercion()
-> Result<()> {
    let html = r#"
        <div id='target'></div>
        <div id='other'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const other = document.getElementById('other');

            target.setPointerCapture('7.9');
            other.releasePointerCapture(7);
            const stillCaptured = target.hasPointerCapture(7);

            const releaseReturn = target.releasePointerCapture(7.2);
            const afterRelease = target.hasPointerCapture(7);

            document.getElementById('result').textContent = [
              stillCaptured,
              releaseReturn === undefined,
              afterRelease
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
fn element_has_pointer_capture_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            target.hasPointerCapture();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("hasPointerCapture requires exactly one pointerId argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
