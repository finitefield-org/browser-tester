use super::*;

#[test]
fn issue_120_open_button_file_input_optional_click_does_not_hang() -> Result<()> {
    let html = r#"
      <div id='csv-editor-fullscreen-dialog' class='hidden'></div>
      <div id='csv-editor-dropzone' tabindex='0'>
        <input id='csv-editor-file-input' type='file' class='hidden' />
      </div>
      <button id='csv-editor-open-button'>open</button>
      <p id='trace'></p>
      <script>
        const state = { dialogOpen: false, ready: false };
        const el = {
          openButton: document.getElementById('csv-editor-open-button'),
          fileInput: document.getElementById('csv-editor-file-input'),
          dropzone: document.getElementById('csv-editor-dropzone'),
          dialog: document.getElementById('csv-editor-fullscreen-dialog'),
          columnSearch: null,
          trace: document.getElementById('trace'),
        };

        function setDialogOpen(open) {
          const show = !!open;
          state.dialogOpen = show;
          el.dialog.classList.toggle('hidden', !show);
          if (show) {
            if (state.ready) el.columnSearch?.focus();
            else el.dropzone?.focus();
          }
        }

        el.fileInput?.addEventListener('click', () => {
          el.trace.textContent += 'file>';
        });

        el.dropzone?.addEventListener('click', () => {
          el.trace.textContent += 'drop>';
          el.fileInput?.click();
        });

        el.openButton?.addEventListener('click', () => {
          el.trace.textContent += 'open>';
          setDialogOpen(true);
          el.fileInput?.click();
          el.trace.textContent += 'done>';
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#csv-editor-open-button")?;
    harness.assert_text("#trace", "open>file>drop>done>")?;
    let dialog_html = harness.dump_dom("#csv-editor-fullscreen-dialog")?;
    assert!(
        !dialog_html.contains("hidden"),
        "dialog should be visible after open click"
    );
    Ok(())
}
