use super::*;

#[test]
fn data_transfer_set_data_replaces_existing_type_without_reordering_types() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          const ret1 = dt.setData('text/plain', 'first');
          dt.setData('text/uri-list', 'https://example.com');
          const ret2 = dt.setData('text/plain', 'updated');

          const types = dt.types;
          document.getElementById('out').textContent = [
            ret1 === undefined && ret2 === undefined,
            types.length,
            types[0],
            types[1],
            dt.getData('text/plain'),
            dt.getData('text'),
            dt.getData('text/uri-list')
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text(
        "#out",
        "true|2|text/plain|text/uri-list|updated|updated|https://example.com",
    )?;
    Ok(())
}

#[test]
fn data_transfer_set_data_appends_type_after_clear_data_for_that_type() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'one');
          dt.setData('text/uri-list', 'https://example.com/a');
          dt.clearData('text/plain');
          dt.setData('text/plain', 'two');

          const types = dt.types;
          document.getElementById('out').textContent = [
            types.length,
            types[0],
            types[1],
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
        "2|text/uri-list|text/plain|two|https://example.com/a",
    )?;
    Ok(())
}
