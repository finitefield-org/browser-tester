use super::*;

#[test]
fn data_transfer_get_data_returns_string_for_existing_formats_during_dragstart() -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragstart', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          dt.setData('text/uri-list', 'https://example.com');
          document.getElementById('out').textContent = [
            dt.getData('text'),
            dt.getData('text/plain'),
            dt.getData('text/uri-list'),
            dt.getData('application/json'),
            dt.types.length
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragstart")?;
    h.assert_text("#out", "alpha|alpha|https://example.com||2")?;
    Ok(())
}

#[test]
fn data_transfer_get_data_returns_empty_string_during_dragover_even_when_types_are_listed(
) -> Result<()> {
    let html = r#"
      <div id='source' draggable='true'></div>
      <p id='out'></p>
      <script>
        document.getElementById('source').addEventListener('dragover', (event) => {
          const dt = event.dataTransfer;
          dt.setData('text/plain', 'alpha');
          document.getElementById('out').textContent = [
            dt.getData('text/plain') === '',
            dt.types.length,
            dt.types[0]
          ].join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#source", "dragover")?;
    h.assert_text("#out", "true|1|text/plain")?;
    Ok(())
}
