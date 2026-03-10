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

#[test]
fn reduced_canvas_to_blob_contract_is_same_task_png_fallback_and_clipboard_compatible_work()
-> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', async () => {
          const canvas = document.createElement('canvas');
          const steps = [];

          canvas.toBlob((blob) => {
            steps.push(`callback:${blob.type}:${blob.size > 0}`);
            navigator.clipboard.write([
              new ClipboardItem({ [blob.type]: blob })
            ]);
          }, 'text/plain');

          steps.push('after');
          document.getElementById('out').textContent = steps.join('|');
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "callback:image/png:true|after")?;
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
