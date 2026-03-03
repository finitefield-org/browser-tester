use super::*;

#[test]
fn data_transfer_set_drag_image_returns_undefined_during_dragstart() -> Result<()> {
    let html = r#"
      <img id='img' src='about:blank' />
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          const ret = dt.setDragImage(document.getElementById('img'), 10, 20);
          document.getElementById('out').textContent = [
            ret === undefined,
            dt.getData('text/plain'),
            dt.types.length
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "true|alpha|1")?;
    Ok(())
}

#[test]
fn data_transfer_set_drag_image_is_noop_outside_dragstart() -> Result<()> {
    let html = r#"
      <img id='img' src='about:blank' />
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragover', (event) => {
          const ret = event.dataTransfer.setDragImage(document.getElementById('img'), 1, 2);
          document.getElementById('out').textContent = String(ret === undefined);
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragover")?;
    h.assert_text("#out", "true")?;
    Ok(())
}

#[test]
fn data_transfer_set_drag_image_requires_three_arguments() {
    let html = r#"
      <img id='img' src='about:blank' />
      <div id='source' draggable='true'></div>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          event.dataTransfer.setDragImage(document.getElementById('img'), 1);
        });
      </script>
    "#;

    let mut h = Harness::from_html(html).expect("harness should initialize");
    let err = h
        .dispatch("#source", "dragstart")
        .expect_err("setDragImage should require exactly three arguments");
    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(
                msg,
                "dataTransfer.setDragImage requires exactly three arguments"
            )
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn data_transfer_set_drag_image_requires_element_first_argument() {
    let html = r#"
      <div id='source' draggable='true'></div>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          event.dataTransfer.setDragImage('not-element', 1, 2);
        });
      </script>
    "#;

    let mut h = Harness::from_html(html).expect("harness should initialize");
    let err = h
        .dispatch("#source", "dragstart")
        .expect_err("setDragImage should reject non-element first argument");
    match err {
        Error::ScriptRuntime(msg) => assert_eq!(
            msg,
            "TypeError: Failed to execute 'setDragImage': parameter 1 is not of type 'Element'"
        ),
        other => panic!("unexpected error: {other:?}"),
    }
}
