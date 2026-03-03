use super::*;

#[test]
fn input_file_mock_is_decodable_with_create_image_bitmap_even_without_explicit_bytes() -> Result<()>
{
    let html = r#"
      <input id='file' type='file' accept='image/jpeg,image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        input.addEventListener('change', async () => {
          const file = input.files[0];
          try {
            const bmp = await createImageBitmap(file);
            document.getElementById('out').textContent = `ok:${bmp.width}x${bmp.height}`;
          } catch {
            document.getElementById('out').textContent = 'error';
          }
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.jpg");
    file.size = 1024;
    file.mime_type = "image/jpeg".to_string();
    h.set_input_files("#file", &[file])?;
    h.assert_text("#out", "ok:1x1")?;
    Ok(())
}

#[test]
fn create_image_bitmap_reads_png_dimensions_from_mock_file_bytes() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        input.addEventListener('change', async () => {
          const file = input.files[0];
          const bmp = await createImageBitmap(file);
          document.getElementById('out').textContent = `${bmp.width}x${bmp.height}`;
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    let png_with_2x3_ihdr = [
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 2, 0, 0, 0, 3, 8, 2,
        0, 0, 0,
    ];
    let mut file = MockFile::new("shape.png").with_bytes(&png_with_2x3_ihdr);
    file.mime_type = "image/png".to_string();
    h.set_input_files("#file", &[file])?;
    h.assert_text("#out", "2x3")?;
    Ok(())
}
