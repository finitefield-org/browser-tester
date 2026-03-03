use super::*;

#[test]
fn issue_84_url_create_object_url_accepts_mock_file_from_set_input_files() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        input.addEventListener('change', () => {
          try {
            const file = input.files[0];
            const url = URL.createObjectURL(file);
            document.getElementById('out').textContent = url.startsWith('blob:bt-') ? 'ok' : 'ng';
          } catch (error) {
            document.getElementById('out').textContent =
              String(error && error.message ? error.message : error);
          }
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.png");
    file.size = 1024;
    file.mime_type = "image/png".to_string();
    h.set_input_files("#file", &[file])?;
    h.assert_text("#out", "ok")?;
    Ok(())
}

#[test]
fn issue_89_mock_image_file_url_preview_and_bitmap_decode_pipeline_works() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/jpeg,image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        input.addEventListener('change', async () => {
          try {
            const file = input.files[0];
            const previewUrl = URL.createObjectURL(file);
            const bmp = await createImageBitmap(file);
            document.getElementById('out').textContent =
              `${previewUrl.startsWith('blob:bt-')}:${bmp.width}x${bmp.height}`;
          } catch (error) {
            document.getElementById('out').textContent =
              'err:' + String(error && error.message ? error.message : error);
          }
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.jpg");
    file.size = 2048;
    file.mime_type = "image/jpeg".to_string();
    h.set_input_files("#file", &[file])?;
    h.assert_text("#out", "true:1x1")?;
    Ok(())
}
