use super::*;

#[test]
fn element_remove_attribute_removes_existing_attribute_and_returns_undefined() -> Result<()> {
    let html = r#"
        <div id='box' disabled data-keep='v'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const removed = box.removeAttribute('DISABLED');
            const removedMissing = box.removeAttribute('missing');
            document.getElementById('result').textContent = [
              removed === undefined,
              removedMissing === undefined,
              box.hasAttribute('disabled'),
              box.disabled,
              box.getAttribute('data-keep')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:false:false:v")?;
    Ok(())
}

#[test]
fn element_remove_attribute_is_noop_when_attribute_is_absent() -> Result<()> {
    let html = r#"
        <div id='box' data-flag='on'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const removed = box.removeAttribute('not-there');
            document.getElementById('result').textContent = [
              removed === undefined,
              box.getAttribute('data-flag'),
              box.hasAttribute('not-there')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:on:false")?;
    Ok(())
}

#[test]
fn element_remove_attribute_rejects_non_single_argument_count() -> Result<()> {
    let html = r#"
        <div id='box' data-x='1'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const removed = box.removeAttribute('data-x', 'extra');
            document.body.setAttribute('data-removed', String(removed));
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("removeAttribute requires exactly one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
