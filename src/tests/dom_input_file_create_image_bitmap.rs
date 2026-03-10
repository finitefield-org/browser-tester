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
fn create_image_bitmap_from_canvas_snapshots_dimensions_before_later_resize_work() -> Result<()> {
    let html = r#"
      <canvas id='canvas' width='20' height='15'></canvas>
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', async () => {
          const canvas = document.getElementById('canvas');
          const pending = createImageBitmap(canvas);
          canvas.width = 40;
          canvas.height = 30;
          const bmp = await pending;
          document.getElementById('out').textContent =
            `${bmp.width}x${bmp.height}|${canvas.width}x${canvas.height}`;
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "20x15|40x30")?;
    Ok(())
}

#[test]
fn reduced_create_image_bitmap_canvas_contract_snapshots_dimensions_before_resize_work()
-> Result<()> {
    let html = r#"
      <canvas id='canvas' width='12' height='9'></canvas>
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', async () => {
          const canvas = document.getElementById('canvas');
          const promise = createImageBitmap(canvas);
          canvas.width = 24;
          canvas.height = 18;
          const bitmap = await promise;
          document.getElementById('out').textContent =
            `${bitmap.width}x${bitmap.height}|${canvas.width}x${canvas.height}`;
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "12x9|24x18")?;
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

#[test]
fn create_image_bitmap_exposes_image_bitmap_constructor_surface_and_branding() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        input.addEventListener('change', async () => {
          const file = input.files[0];
          const bmp = await createImageBitmap(file);
          const proto = ImageBitmap.prototype;
          const widthDesc = Object.getOwnPropertyDescriptor(proto, 'width');
          const heightDesc = Object.getOwnPropertyDescriptor(proto, 'height');
          const closeDesc = Object.getOwnPropertyDescriptor(proto, 'close');
          let illegal = '';
          try {
            ImageBitmap();
          } catch (e) {
            illegal = String(e);
          }
          document.getElementById('out').textContent = [
            typeof ImageBitmap,
            String(window.ImageBitmap === ImageBitmap),
            String(bmp.constructor === ImageBitmap),
            String(Object.getPrototypeOf(bmp) === ImageBitmap.prototype),
            String(Object.getPrototypeOf(ImageBitmap.prototype) === Object.prototype),
            Object.getOwnPropertyNames(ImageBitmap.prototype).sort().join(','),
            String(typeof widthDesc.get),
            String(widthDesc.enumerable),
            String(typeof heightDesc.get),
            String(typeof closeDesc.value),
            String(closeDesc.enumerable),
            Object.prototype.toString.call(bmp),
            String(illegal.includes('Illegal constructor'))
          ].join('|');
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
    h.assert_text(
        "#out",
        "function|true|true|true|true|close,constructor,height,width|function|false|function|function|false|[object ImageBitmap]|true",
    )?;
    Ok(())
}

#[test]
fn image_bitmap_close_and_reflective_surface_support_extracted_calls_and_shadow_restore()
-> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        input.addEventListener('change', async () => {
          const file = input.files[0];
          const bmp = await createImageBitmap(file);
          const proto = ImageBitmap.prototype;
          const widthGetter = Object.getOwnPropertyDescriptor(proto, 'width').get;
          const heightGetter = Object.getOwnPropertyDescriptor(proto, 'height').get;
          const close = bmp.close;
          let widthError = '';
          let closeError = '';
          try {
            widthGetter.call({});
          } catch (e) {
            widthError = String(e);
          }
          try {
            close.call({});
          } catch (e) {
            closeError = String(e);
          }

          const before = [
            String(close === ImageBitmap.prototype.close),
            String(widthGetter.call(bmp)),
            String(heightGetter.call(bmp)),
            String(Object.keys(bmp).length),
            String(Object.keys(Object.assign({}, bmp)).length),
            String(Object.keys({ ...bmp }).length)
          ].join(':');

          Object.defineProperty(ImageBitmap, 'marker', {
            value: 'ctor',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(ImageBitmap.prototype, 'marker', {
            value: 'proto',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(bmp, 'width', {
            value: 'shadow-width',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(bmp, 'marker', {
            value: 'bitmap',
            enumerable: true,
            configurable: true
          });

          const shadowed = [
            ImageBitmap.marker,
            ImageBitmap.prototype.marker,
            bmp.width,
            bmp.marker,
            Object.keys(ImageBitmap).join(','),
            Object.keys(ImageBitmap.prototype).join(','),
            Object.keys(bmp).sort().join(','),
            Object.assign({}, bmp).width,
            Object.assign({}, bmp).marker
          ].join(':');

          delete ImageBitmap.marker;
          delete ImageBitmap.prototype.marker;
          delete bmp.width;
          delete bmp.marker;
          close.call(bmp);

          const restored = [
            String(Object.hasOwn(ImageBitmap, 'marker')),
            String(Object.hasOwn(ImageBitmap.prototype, 'marker')),
            String(Object.hasOwn(bmp, 'width')),
            String(Object.hasOwn(bmp, 'marker')),
            String(widthGetter.call(bmp)),
            String(heightGetter.call(bmp)),
            String(widthError.includes('ImageBitmap method called on incompatible receiver')),
            String(closeError.includes('ImageBitmap method called on incompatible receiver')),
            String(Object.keys(bmp).length)
          ].join(':');

          document.getElementById('out').textContent = [before, shadowed, restored].join('|');
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
    h.assert_text(
        "#out",
        "true:2:3:0:0:0|ctor:proto:shadow-width:bitmap:marker:marker:marker,width:shadow-width:bitmap|false:false:false:false:0:0:true:true:0",
    )?;
    Ok(())
}
