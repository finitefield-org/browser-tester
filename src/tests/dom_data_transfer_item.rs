use super::*;

#[test]
fn data_transfer_item_string_kind_exposes_kind_type_and_string_methods() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          const item = dt.items[0];
          let callbackValue = '';
          const ret = item.getAsString((value) => {
            callbackValue = value;
          });
          item.getAsFileSystemHandle().then((handle) => {
            document.getElementById('out').textContent = [
              dt.items.length,
              item.kind,
              item.type,
              ret === undefined,
              item.getAsFile() === null,
              callbackValue,
              handle === null,
              item.webkitGetAsEntry() === null
            ].join('|');
          });
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "1|string|text/plain|true|true|alpha|true|true")?;
    Ok(())
}

#[test]
fn data_transfer_item_order_preserved_when_set_data_replaces_existing_type() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'one');
          dt.setData('text/uri-list', 'https://example.com');
          dt.setData('text/plain', 'two');

          const firstItem = dt.items[0];
          const secondItem = dt.items[1];
          let firstValue = '';
          firstItem.getAsString((value) => {
            firstValue = value;
          });
          document.getElementById('out').textContent = [
            dt.items.length,
            firstItem.type,
            firstValue,
            secondItem.type
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "2|text/plain|two|text/uri-list")?;
    Ok(())
}

#[test]
fn data_transfer_item_list_tracks_clear_data_removals() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          dt.setData('text/uri-list', 'https://example.com');
          dt.clearData('text/plain');

          const firstItem = dt.items[0];
          let value = '';
          firstItem.getAsString((text) => {
            value = text;
          });
          document.getElementById('out').textContent = [
            dt.items.length,
            firstItem.type,
            value
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "1|text/uri-list|https://example.com")?;
    Ok(())
}

#[test]
fn data_transfer_item_get_as_string_requires_callback() {
    let html = r#"
      <div id='source' draggable='true'></div>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          dt.items[0].getAsString();
        });
      </script>
    "#;

    let mut h = Harness::from_html(html).expect("harness should initialize");
    let err = h
        .dispatch("#source", "dragstart")
        .expect_err("getAsString should require a callback argument");
    match err {
        Error::ScriptRuntime(msg) => assert_eq!(
            msg,
            "DataTransferItem.getAsString requires exactly one callback argument"
        ),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn data_transfer_item_get_as_string_requires_callable_callback() {
    let html = r#"
      <div id='source' draggable='true'></div>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          dt.items[0].getAsString('not-a-function');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html).expect("harness should initialize");
    let err = h
        .dispatch("#source", "dragstart")
        .expect_err("getAsString should require a callable callback");
    match err {
        Error::ScriptRuntime(msg) => assert_eq!(
            msg,
            "DataTransferItem.getAsString callback must be callable"
        ),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn data_transfer_item_get_as_string_does_not_invoke_callback_for_file_item() -> Result<()> {
    let html = r#"
      <input id='upload' type='file'>
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        const source = document.getElementById('source');
        const upload = document.getElementById('upload');
        source.addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.files.push(upload.files[0]);
          dt.setData('text/plain', 'alpha');

          const fileItem = dt.items[1];
          let called = false;
          const ret = fileItem.getAsString(() => {
            called = true;
          });
          document.getElementById('out').textContent = [
            fileItem.kind,
            ret === undefined,
            called
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_input_files("#upload", &[MockFile::new("drop.txt").with_text("hello")])?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "file|true|false")?;
    Ok(())
}

#[test]
fn data_transfer_item_get_as_file_returns_mock_file_for_file_item() -> Result<()> {
    let html = r#"
      <input id='upload' type='file'>
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        const source = document.getElementById('source');
        const upload = document.getElementById('upload');
        source.addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.files.push(upload.files[0]);
          dt.setData('text/plain', 'alpha');

          const fileItem = dt.items[1];
          const file = fileItem.getAsFile();
          document.getElementById('out').textContent = [
            dt.items.length,
            fileItem.kind,
            fileItem.type,
            file && file.name,
            file && file.size
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_input_files("#upload", &[MockFile::new("drop.txt").with_text("hello")])?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "2|file|text/plain|drop.txt|5")?;
    Ok(())
}

#[test]
fn data_transfer_item_get_as_file_rejects_extra_arguments() {
    let html = r#"
      <div id='source' draggable='true'></div>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          dt.items[0].getAsFile('unexpected');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html).expect("harness should initialize");
    let err = h
        .dispatch("#source", "dragstart")
        .expect_err("getAsFile should reject extra arguments");
    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "DataTransferItem.getAsFile does not take arguments")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
