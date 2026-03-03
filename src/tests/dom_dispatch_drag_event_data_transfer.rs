use super::*;

#[test]
fn dispatch_dragover_exposes_data_transfer_object() -> Result<()> {
    let html = r#"
      <div id='drop'></div>
      <p id='out'></p>
      <script>
        const drop = document.getElementById('drop');
        drop.addEventListener('dragover', (event) => {
          event.preventDefault();
          event.dataTransfer.dropEffect = 'copy';
          drop.classList.add('dragging');
          document.getElementById('out').textContent = [
            drop.classList.contains('dragging'),
            event.dataTransfer.dropEffect
          ].join(':');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#drop", "dragover")?;
    h.assert_text("#out", "true:copy")?;
    Ok(())
}
