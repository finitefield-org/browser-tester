use super::*;

#[test]
fn dispatch_paste_exposes_clipboard_data_payload() -> Result<()> {
    let html = r#"
      <input id='i' />
      <p id='out'></p>
      <script>
        const input = document.getElementById('i');
        input.addEventListener('paste', (event) => {
          const val = event.clipboardData && event.clipboardData.getData
            ? event.clipboardData.getData('text/plain')
            : '';
          document.getElementById('out').textContent = val || '(empty)';
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("A001\t10.01\t9.99");
    h.dispatch("#i", "paste")?;
    h.assert_text("#out", "A001\t10.01\t9.99")?;
    Ok(())
}

#[test]
fn dispatch_paste_get_data_returns_empty_string_for_unknown_format() -> Result<()> {
    let html = r#"
      <input id='i' />
      <p id='out'></p>
      <script>
        document.getElementById('i').addEventListener('paste', (event) => {
          document.getElementById('out').textContent = [
            event.clipboardData.getData('application/json') === '',
            event.clipboardData.getData('text')
          ].join(':');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("hello");
    h.dispatch("#i", "paste")?;
    h.assert_text("#out", "true:hello")?;
    Ok(())
}

#[test]
fn dispatch_paste_clipboard_data_raw_getter_paths_work() -> Result<()> {
    let html = r#"
      <input id='i' />
      <p id='out'></p>
      <script>
        document.getElementById('i').addEventListener('paste', (event) => {
          const getData = event.clipboardData['getData'];
          const setData = event.clipboardData.setData;
          const clearData = Object.create(event.clipboardData).clearData;

          const before = getData.call(event.clipboardData, 'text/plain');
          setData.call(event.clipboardData, 'text/html', '<b>hello</b>');
          const html = getData.call(event.clipboardData, 'text/html');
          clearData.call(event.clipboardData, 'text/html');
          const cleared = getData.call(event.clipboardData, 'text/html') === '';

          let incompatible = false;
          try {
            getData.call({}, 'text/plain');
          } catch (error) {
            incompatible = String(error).includes('DataTransfer');
          }

          document.getElementById('out').textContent = [
            getData.name,
            getData.length,
            setData.name,
            setData.length,
            before,
            html,
            cleared,
            incompatible
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("hello");
    h.dispatch("#i", "paste")?;
    h.assert_text("#out", "getData|1|setData|2|hello|<b>hello</b>|true|true")?;
    Ok(())
}

#[test]
fn dispatch_paste_clipboard_data_descriptor_and_delete_shadowing_work() -> Result<()> {
    let html = r#"
      <input id='i' />
      <p id='out'></p>
      <script>
        document.getElementById('i').addEventListener('paste', (event) => {
          const getDataDesc = Object.getOwnPropertyDescriptor(event.clipboardData, 'getData');
          const before = [
            getDataDesc.value.name,
            getDataDesc.value.length,
            getDataDesc.enumerable,
            Object.keys(event.clipboardData).includes('getData'),
            getDataDesc.value.call(event.clipboardData, 'text/plain')
          ].join(':');

          const deleted = delete event.clipboardData.getData;
          const afterDelete = event.clipboardData.getData === undefined;

          Object.defineProperty(event.clipboardData, 'getData', {
            value(format) { return 'override:' + format; },
            configurable: true
          });

          const overrideDesc = Object.getOwnPropertyDescriptor(event.clipboardData, 'getData');
          const overrideValue = event.clipboardData.getData('text/plain');
          const overrideCall = overrideDesc.value('text/html');

          const redeleted = delete event.clipboardData.getData;
          const afterRedelete = event.clipboardData.getData === undefined;

          document.getElementById('out').textContent = [
            before,
            deleted,
            afterDelete,
            overrideValue,
            overrideCall,
            redeleted,
            afterRedelete
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("hello");
    h.dispatch("#i", "paste")?;
    h.assert_text(
        "#out",
        "getData:1:false:false:hello|true|true|override:text/plain|override:text/html|true|true",
    )?;
    Ok(())
}

#[test]
fn dispatch_paste_clipboard_types_wrapper_survives_shadow_delete_and_mutation_work() -> Result<()> {
    let html = r#"
      <input id='i' />
      <p id='out'></p>
      <script>
        document.getElementById('i').addEventListener('paste', (event) => {
          const liveTypes = event.clipboardData.types;

          Object.defineProperty(event.clipboardData, 'types', {
            value: 'shadow-types',
            configurable: true
          });
          event.clipboardData.setData('text/html', '<b>hello</b>');

          const shadowAfterSet = event.clipboardData.types === 'shadow-types';
          const inheritedShadow = Object.create(event.clipboardData).types === 'shadow-types';
          const liveAfterSet = [
            liveTypes.length,
            liveTypes[0],
            liveTypes[1]
          ].join(':');

          const deleted = delete event.clipboardData.types;
          event.clipboardData.clearData('text/html');

          document.getElementById('out').textContent = [
            String(shadowAfterSet),
            String(inheritedShadow),
            liveAfterSet,
            String(deleted),
            String(event.clipboardData.types === undefined),
            String(Object.create(event.clipboardData).types === undefined),
            [liveTypes.length, liveTypes[0]].join(':'),
            event.clipboardData.getData('text/plain')
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("hello");
    h.dispatch("#i", "paste")?;
    h.assert_text(
        "#out",
        "true|true|2:text/plain:text/html|true|true|true|1:text/plain|hello",
    )?;
    Ok(())
}
