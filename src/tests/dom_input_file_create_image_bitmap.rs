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

#[test]
fn create_image_bitmap_supports_crop_resize_and_image_bitmap_source() -> Result<()> {
    let html = r#"
      <canvas id='canvas' width='20' height='15'></canvas>
      <input id='file' type='file' accept='image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        input.addEventListener('change', async () => {
          try {
            const file = input.files[0];
            const base = await createImageBitmap(file);
            const crop = await createImageBitmap(file, 1, 2, 3, 4);
            const cropNeg = await createImageBitmap(file, 0, 0, -5, -6);
            const resized = await createImageBitmap(file, { resizeWidth: 7, resizeHeight: 9 });
            const cropResized = await createImageBitmap(file, 0, 0, 3, 4, {
              resizeWidth: 11,
              resizeHeight: 13
            });
            const clone = await createImageBitmap(base, { resizeWidth: 12 });
            const fromCanvas = await createImageBitmap(document.getElementById('canvas'));
            document.getElementById('out').textContent =
              `${base.width}x${base.height}|${crop.width}x${crop.height}|` +
              `${cropNeg.width}x${cropNeg.height}|${resized.width}x${resized.height}|` +
              `${cropResized.width}x${cropResized.height}|${clone.width}x${clone.height}|` +
              `${fromCanvas.width}x${fromCanvas.height}`;
          } catch (e) {
            document.getElementById('out').textContent = 'error:' + String(e);
          }
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    let png_with_10x8_ihdr = [
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 10, 0, 0, 0, 8, 8,
        2, 0, 0, 0,
    ];
    let mut file = MockFile::new("sheet.png").with_bytes(&png_with_10x8_ihdr);
    file.mime_type = "image/png".to_string();
    h.set_input_files("#file", &[file])?;
    h.assert_text("#out", "10x8|3x4|5x6|7x9|11x13|12x8|20x15")?;
    Ok(())
}

#[test]
fn create_image_bitmap_rejects_invalid_signatures_and_options() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        input.addEventListener('change', async () => {
          const file = input.files[0];
          const results = [];
          try {
            await createImageBitmap(file, 1, 2, 3);
            results.push('badA');
          } catch (e) {
            results.push(String(e).includes('supports 1, 2, 5, or 6 arguments'));
          }
          try {
            await createImageBitmap(file, { resizeWidth: 0 });
            results.push('badB');
          } catch (e) {
            results.push(String(e).includes('resizeWidth'));
          }
          try {
            await createImageBitmap(file, 0, 0, 0, 2);
            results.push('badC');
          } catch (e) {
            results.push(String(e).includes('crop width/height'));
          }
          try {
            await createImageBitmap(file, 0, 0, 2, 2, 42);
            results.push('badD');
          } catch (e) {
            results.push(String(e).includes('options must be an object'));
          }
          document.getElementById('out').textContent = results.join(':');
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
    h.assert_text("#out", "true:true:true:true")?;
    Ok(())
}
