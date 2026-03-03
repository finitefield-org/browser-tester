use super::*;

#[test]
fn data_transfer_clear_data_without_args_clears_all_types() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          dt.setData('text/uri-list', 'https://example.com');
          const ret = dt.clearData();
          document.getElementById('out').textContent = [
            ret === undefined,
            dt.types.length,
            dt.getData('text/plain') === '',
            dt.getData('text/uri-list') === ''
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "true|0|true|true")?;
    Ok(())
}

#[test]
fn data_transfer_clear_data_empty_string_clears_all_types() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          dt.setData('text/uri-list', 'https://example.com');
          dt.clearData('');
          document.getElementById('out').textContent = String(dt.types.length);
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "0")?;
    Ok(())
}

#[test]
fn data_transfer_clear_data_unknown_format_is_noop() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          dt.setData('text/uri-list', 'https://example.com');
          dt.clearData('application/json');
          document.getElementById('out').textContent = [
            dt.types.length,
            dt.types[0],
            dt.types[1],
            dt.getData('text/plain'),
            dt.getData('text/uri-list')
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text(
        "#out",
        "2|text/plain|text/uri-list|alpha|https://example.com",
    )?;
    Ok(())
}

#[test]
fn data_transfer_clear_data_is_noop_outside_dragstart() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragover', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          const before = dt.types.length;
          const ret = dt.clearData();
          const after = dt.types.length;
          document.getElementById('out').textContent = [
            ret === undefined,
            before,
            after
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragover")?;
    h.assert_text("#out", "true|1|1")?;
    Ok(())
}
