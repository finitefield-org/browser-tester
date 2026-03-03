use super::*;

#[test]
fn clipboard_write_records_clipboard_item_png_payload_for_assertions() -> Result<()> {
    let html = r#"
      <button id='copy'>copy</button>
      <p id='out'></p>
      <script>
        document.getElementById('copy').addEventListener('click', async () => {
          const pngBlob = new Blob([new Uint8Array([137, 80, 78, 71, 1, 2, 3])], {
            type: 'image/png'
          });
          await navigator.clipboard.write([
            new ClipboardItem({ 'image/png': pngBlob })
          ]);
          document.getElementById('out').textContent = 'copied';
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#copy")?;
    h.assert_text("#out", "copied")?;
    assert_eq!(
        h.take_clipboard_writes(),
        vec![ClipboardWriteArtifact {
            payloads: vec![ClipboardPayloadArtifact {
                mime_type: "image/png".to_string(),
                bytes: vec![137, 80, 78, 71, 1, 2, 3],
            }],
        }]
    );
    Ok(())
}

#[test]
fn clipboard_write_honors_mocked_rejection_and_does_not_record_payloads() -> Result<()> {
    let html = r#"
      <button id='copy'>copy</button>
      <p id='out'></p>
      <script>
        document.getElementById('copy').addEventListener('click', async () => {
          const pngBlob = new Blob([new Uint8Array([1, 2, 3])], { type: 'image/png' });
          try {
            await navigator.clipboard.write([
              new ClipboardItem({ 'image/png': pngBlob })
            ]);
            document.getElementById('out').textContent = 'ok';
          } catch (err) {
            document.getElementById('out').textContent = `err:${String(err)}`;
          }
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_write_error(Some("WriteBlocked"));
    h.click("#copy")?;
    h.assert_text("#out", "err:WriteBlocked")?;
    assert!(h.take_clipboard_writes().is_empty());
    Ok(())
}
