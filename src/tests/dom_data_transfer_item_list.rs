use super::*;

#[test]
fn data_transfer_item_list_add_replaces_string_item_without_reordering() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          const items = dt.items;
          items.add('one', 'text/plain');
          items.add('https://example.com', 'text/uri-list');
          const replaced = items.add('two', 'text/plain');
          const firstItem = items[0];
          const secondItem = items[1];

          let replacedValue = '';
          replaced.getAsString((value) => {
            replacedValue = value;
          });

          document.getElementById('out').textContent = [
            dt.items.length,
            firstItem.type,
            secondItem.type,
            replaced.kind,
            replaced.type,
            replacedValue,
            dt.getData('text/plain')
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text(
        "#out",
        "2|text/plain|text/uri-list|string|text/plain|two|two",
    )?;
    Ok(())
}

#[test]
fn data_transfer_item_list_add_file_appends_to_files_and_items() -> Result<()> {
    let html = r#"
      <input id='upload' type='file'>
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        const source = document.getElementById('source');
        const upload = document.getElementById('upload');

        source.addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          const items = dt.items;
          const item = items.add(upload.files[0]);
          const file = item.getAsFile();

          document.getElementById('out').textContent = [
            dt.items.length,
            dt.files[0] && dt.files[0].name,
            item.kind,
            item.type,
            file && file.name,
            file && file.size
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_input_files("#upload", &[MockFile::new("drop.txt").with_text("hello")])?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "1|drop.txt|file|text/plain|drop.txt|5")?;
    Ok(())
}

#[test]
fn data_transfer_item_list_remove_can_remove_string_and_file_items() -> Result<()> {
    let html = r#"
      <input id='upload' type='file'>
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        const source = document.getElementById('source');
        const upload = document.getElementById('upload');

        source.addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          const items = dt.items;
          items.add('alpha', 'text/plain');
          items.add('https://example.com', 'text/uri-list');
          items.add(upload.files[0]);

          const removeFirst = items.remove(1);
          const removeSecond = items.remove(1);

          document.getElementById('out').textContent = [
            removeFirst === undefined,
            removeSecond === undefined,
            dt.items.length,
            dt.types.length,
            dt.files[0] === undefined,
            dt.types[0],
            dt.getData('text/plain')
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_input_files("#upload", &[MockFile::new("drop.txt").with_text("hello")])?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "true|true|1|1|true|text/plain|alpha")?;
    Ok(())
}

#[test]
fn data_transfer_item_list_clear_removes_all_items() -> Result<()> {
    let html = r#"
      <input id='upload' type='file'>
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        const source = document.getElementById('source');
        const upload = document.getElementById('upload');

        source.addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          const items = dt.items;
          items.add('alpha', 'text/plain');
          items.add(upload.files[0]);
          const ret = items.clear();

          document.getElementById('out').textContent = [
            ret === undefined,
            dt.items.length,
            dt.types['length'],
            dt.files[0] === undefined,
            dt.getData('text/plain')
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_input_files("#upload", &[MockFile::new("drop.txt").with_text("hello")])?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "true|0|0|true|")?;
    Ok(())
}

#[test]
fn data_transfer_item_list_methods_are_noop_outside_dragstart() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragover', (event) => {
          const dt = event.dataTransfer;
          const items = dt.items;
          const addResult = items.add('alpha', 'text/plain');
          const clearResult = items.clear();
          const removeResult = items.remove(0);

          document.getElementById('out').textContent = [
            addResult === null,
            clearResult === undefined,
            removeResult === undefined,
            dt.items.length,
            dt.types['length'],
            dt.getData('text/plain')
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragover")?;
    h.assert_text("#out", "true|true|true|0|0|")?;
    Ok(())
}

#[test]
fn data_transfer_item_list_add_rejects_non_file_single_argument() {
    let html = r#"
      <div id='source' draggable='true'></div>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const items = event.dataTransfer.items;
          items.add({});
        });
      </script>
    "#;

    let mut h = Harness::from_html(html).expect("harness should initialize");
    let err = h
        .dispatch("#source", "dragstart")
        .expect_err("add should reject non-File single argument");
    match err {
        Error::ScriptRuntime(msg) => assert_eq!(
            msg,
            "TypeError: Failed to execute 'add' on 'DataTransferItemList': parameter 1 is not of type 'File'"
        ),
        other => panic!("unexpected error: {other:?}"),
    }
}
