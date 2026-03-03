use super::*;

#[test]
fn issue_96_canvas_to_blob_is_available_for_created_canvas_and_allows_clipboard_write() -> Result<()>
{
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', async () => {
          try {
            const canvas = document.createElement('canvas');
            canvas.width = 8;
            canvas.height = 8;
            const ctx = canvas.getContext('2d');
            ctx.fillStyle = '#000';
            ctx.fillRect(0, 0, 8, 8);

            const pngBlob = await new Promise((resolve, reject) => {
              canvas.toBlob((blob) => {
                if (!blob) return reject(new Error('blob-null'));
                resolve(blob);
              }, 'image/png');
            });

            await navigator.clipboard.write([
              new ClipboardItem({ 'image/png': pngBlob })
            ]);
            document.getElementById('out').textContent = 'copied';
          } catch (err) {
            document.getElementById('out').textContent =
              'err:' + String(err && err.message ? err.message : err);
          }
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "copied")?;
    assert_eq!(
        h.take_clipboard_writes(),
        vec![ClipboardWriteArtifact {
            payloads: vec![ClipboardPayloadArtifact {
                mime_type: "image/png".to_string(),
                bytes: vec![137, 80, 78, 71],
            }],
        }]
    );
    Ok(())
}
