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
fn data_transfer_raw_getters_and_inherited_property_paths_work() -> Result<()> {
    let html = r#"
      <div id='drag-image'></div>
      <p id='out'></p>
      <script>
        const dt = new DataTransfer();
        const element = document.getElementById('drag-image');

        const setData = dt['setData'];
        const getData = Object.create(dt).getData;
        const clearData = dt.clearData;
        const setDragImage = dt['setDragImage'];
        const addElement = dt.addElement;
        const addItem = dt.items['add'];
        const removeItem = Object.create(dt.items).remove;
        const clearItems = dt.items.clear;

        const firstItem = addItem.call(dt.items, 'alpha', 'text/plain');
        addItem.call(dt.items, 'https://example.com', 'text/uri-list');

        let firstValue = '';
        const getAsString = firstItem['getAsString'];
        const getAsFile = firstItem.getAsFile;
        getAsString.call(firstItem, (value) => { firstValue = value; });

        setData.call(dt, 'text/html', '<b>beta</b>');
        const htmlValue = getData.call(dt, 'text/html');
        const removeRet = removeItem.call(dt.items, 1);
        const setDragRet = setDragImage.call(dt, element, 1, 2);
        const addElementRet = addElement.call(dt, element);
        clearData.call(dt, 'text/html');
        const cleared = getData.call(dt, 'text/html');
        const clearRet = clearItems.call(dt.items);

        let incompatible = false;
        try {
          addItem.call([], 'x', 'text/plain');
        } catch (error) {
          incompatible = String(error).includes('DataTransferItemList');
        }

        document.getElementById('out').textContent = [
          setData.name,
          setData.length,
          getAsString.name,
          getAsString.length,
          firstValue,
          getAsFile.call(firstItem) === null,
          htmlValue,
          removeRet === undefined,
          setDragRet === undefined,
          addElementRet === undefined,
          cleared === '',
          clearRet === undefined,
          dt.items.length,
          incompatible
        ].join('|');
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#out",
        "setData|2|getAsString|1|alpha|true|<b>beta</b>|true|true|true|true|true|0|true",
    )?;
    Ok(())
}

#[test]
fn data_transfer_placeholder_method_descriptor_and_delete_shadowing_work() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const dt = new DataTransfer();
        const setDataDesc = Object.getOwnPropertyDescriptor(dt, 'setData');
        const addDesc = Object.getOwnPropertyDescriptor(dt.items, 'add');
        const item = addDesc.value.call(dt.items, 'alpha', 'text/plain');
        const itemDesc = Object.getOwnPropertyDescriptor(item, 'getAsString');
        const keysSnapshot = [
          Object.keys(dt).includes('setData'),
          Object.keys(dt.items).includes('add'),
          Object.keys(item).includes('getAsString')
        ].join(':');

        let itemValue = '';
        itemDesc.value.call(item, (value) => { itemValue = value; });

        const deletedSetData = delete dt.setData;
        const afterSetData = dt.setData === undefined;
        const deletedAdd = delete dt.items.add;
        const afterAdd = dt.items.add === undefined;
        Object.defineProperty(dt.items, 'add', {
          value(data, type) { return 'shadow:' + data + ':' + type; },
          configurable: true
        });
        const overrideAdd = dt.items.add('beta', 'text/plain');
        const deletedGetAsString = delete item.getAsString;
        const afterGetAsString = item.getAsString === undefined;

        document.getElementById('out').textContent = [
          setDataDesc.value.name,
          setDataDesc.value.length,
          setDataDesc.enumerable,
          addDesc.value.name,
          addDesc.value.length,
          addDesc.enumerable,
          itemDesc.value.name,
          itemDesc.value.length,
          itemDesc.enumerable,
          keysSnapshot,
          itemValue,
          deletedSetData,
          afterSetData,
          deletedAdd,
          afterAdd,
          overrideAdd,
          deletedGetAsString,
          afterGetAsString
        ].join('|');
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#out",
        "setData|2|false|add|1|false|getAsString|1|false|false:false:false|alpha|true|true|true|true|shadow:beta:text/plain|true|true",
    )?;
    Ok(())
}

#[test]
fn data_transfer_live_wrappers_survive_shadow_delete_and_item_list_mutations_work() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const dt = new DataTransfer();
        const liveTypes = dt.types;
        const liveItems = dt.items;
        const liveFiles = dt.files;

        Object.defineProperty(dt, 'types', {
          value: 'shadow-types',
          configurable: true
        });
        Object.defineProperty(dt, 'items', {
          value: 'shadow-items',
          configurable: true
        });
        Object.defineProperty(dt, 'files', {
          value: 'shadow-files',
          configurable: true
        });

        liveItems.add('alpha', 'text/plain');
        liveItems.add('beta', 'text/html');
        liveItems.add(new File(['ab'], 'sample.csv', { type: 'text/csv' }));

        const inheritedShadow = [
          Object.create(dt).types === 'shadow-types',
          Object.create(dt).items === 'shadow-items',
          Object.create(dt).files === 'shadow-files'
        ].join(':');
        const directShadow = [
          dt.types === 'shadow-types',
          dt.items === 'shadow-items',
          dt.files === 'shadow-files'
        ].join(':');
        const liveAfterAdd = [
          liveTypes.length,
          liveTypes[0],
          liveTypes[1],
          liveItems.length,
          liveFiles.length,
          liveFiles[0].name
        ].join(':');

        const deletedTypes = delete dt.types;
        const deletedItems = delete dt.items;
        const deletedFiles = delete dt.files;

        liveItems.remove(1);
        liveItems.clear();

        document.getElementById('out').textContent = [
          inheritedShadow,
          directShadow,
          liveAfterAdd,
          [deletedTypes, deletedItems, deletedFiles].join(':'),
          [
            dt.types === undefined,
            dt.items === undefined,
            dt.files === undefined
          ].join(':'),
          [
            liveTypes.length,
            liveItems.length,
            liveFiles.length
          ].join(':')
        ].join('|');
      </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#out",
        "true:true:true|true:true:true|2:text/plain:text/html:3:1:sample.csv|true:true:true|true:true:true|0:0:0",
    )?;
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
