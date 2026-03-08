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

#[test]
fn drag_event_reuses_data_transfer_wrapper_across_listeners_work() -> Result<()> {
    let html = r#"
      <div id='drop'></div>
      <p id='out'></p>
      <script>
        const drop = document.getElementById('drop');
        let firstRef;

        drop.addEventListener('dragover', (event) => {
          event.preventDefault();
          firstRef = event.dataTransfer;
          event.dataTransfer.dropEffect = 'copy';
          event.dataTransfer.customFlag = 'persisted';
          event.dataTransfer.getData = () => 'shadow-data';
        });

        drop.addEventListener('dragover', (event) => {
          document.getElementById('out').textContent = [
            String(firstRef === event.dataTransfer),
            event.dataTransfer.dropEffect,
            event.dataTransfer.customFlag,
            event.dataTransfer.getData('text/plain')
          ].join(':');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#drop", "dragover")?;
    h.assert_text("#out", "true:copy:persisted:shadow-data")?;
    Ok(())
}
