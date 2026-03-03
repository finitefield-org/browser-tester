use super::*;

#[test]
fn data_transfer_constructor_creates_empty_data_transfer_object() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const dt = new DataTransfer();
        document.getElementById('out').textContent = [
          dt.dropEffect,
          dt.effectAllowed,
          dt['types']['length'],
          dt['items']['length'],
          dt['files']['length'],
          dt.getData('text/plain') === ''
        ].join('|');
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "none|all|0|0|0|true")?;
    Ok(())
}

#[test]
fn data_transfer_constructor_supports_set_data_get_data_and_clear_data() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const dt = new DataTransfer();
        const setRet = dt.setData('text/plain', 'alpha');
        const beforeClear = dt.getData('text');
        const clearRet = dt.clearData('text/plain');
        document.getElementById('out').textContent = [
          setRet === undefined,
          beforeClear,
          clearRet === undefined,
          dt.getData('text/plain') === '',
          dt['types']['length'],
          dt['items']['length']
        ].join('|');
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "true|alpha|true|true|0|0")?;
    Ok(())
}

#[test]
fn data_transfer_drop_effect_and_effect_allowed_enforce_allowed_values() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const dt = new DataTransfer();

        dt.dropEffect = 'copy';
        const dropAfterValid = dt.dropEffect;
        dt.dropEffect = 'INVALID';
        const dropAfterInvalid = dt.dropEffect;

        dt.effectAllowed = 'copyLink';
        const effectAfterValid = dt.effectAllowed;
        dt.effectAllowed = 'INVALID';
        const effectAfterInvalid = dt.effectAllowed;
        dt.effectAllowed = 'COPYMOVE';
        const effectAfterUpper = dt.effectAllowed;

        document.getElementById('out').textContent = [
          dropAfterValid,
          dropAfterInvalid,
          effectAfterValid,
          effectAfterInvalid,
          effectAfterUpper
        ].join('|');
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "copy|none|copyLink|copyLink|copyMove")?;
    Ok(())
}

#[test]
fn data_transfer_types_items_and_files_are_read_only_on_assignment() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const dt = new DataTransfer();
        dt.setData('text/plain', 'alpha');

        dt.types = ['application/json'];
        dt.items = [];
        dt.files = [];

        document.getElementById('out').textContent = [
          dt['types']['length'],
          dt['types'][0],
          dt['items']['length'],
          dt['files']['length'],
          dt.getData('text/plain')
        ].join('|');
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "1|text/plain|1|0|alpha")?;
    Ok(())
}

#[test]
fn data_transfer_add_element_accepts_element_and_returns_undefined() -> Result<()> {
    let html = r#"
      <div id='source'></div>
      <p id='out'></p>
      <script>
        const dt = new DataTransfer();
        const ret = dt.addElement(document.getElementById('source'));
        document.getElementById('out').textContent = String(ret === undefined);
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "true")?;
    Ok(())
}

#[test]
fn data_transfer_add_element_rejects_non_element_argument() {
    let html = r#"
      <div id='source'></div>
      <script>
        document.getElementById('source').addEventListener('dragstart', () => {
          const dt = new DataTransfer();
          dt.addElement('not-element');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html).expect("harness should initialize");
    let err = h
        .dispatch("#source", "dragstart")
        .expect_err("addElement should reject non-element argument");
    match err {
        Error::ScriptRuntime(msg) => assert_eq!(
            msg,
            "TypeError: Failed to execute 'addElement': parameter 1 is not of type 'Element'"
        ),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn data_transfer_constructor_rejects_arguments() {
    let html = r#"
      <div id='source'></div>
      <script>
        document.getElementById('source').addEventListener('dragstart', () => {
          new DataTransfer('unexpected');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html).expect("harness should initialize");
    let err = h
        .dispatch("#source", "dragstart")
        .expect_err("DataTransfer constructor should reject arguments");
    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "DataTransfer constructor does not take arguments")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
