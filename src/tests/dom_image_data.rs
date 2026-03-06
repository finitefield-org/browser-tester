use super::*;

#[test]
fn image_data_constructor_with_dimensions_defaults_to_unorm8_srgb() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const image = new ImageData(2, 3);
          image.data[0] = 255;
          document.getElementById('result').textContent = [
            image.width,
            image.height,
            image.data.length,
            image.data[0],
            image.colorSpace,
            image.pixelFormat
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "2:3:24:255:srgb:rgba-unorm8")?;
    Ok(())
}

#[test]
fn image_data_constructor_with_uint8_data_infers_height_and_copies_data() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const input = new Uint8ClampedArray([1, 2, 3, 4, 5, 6, 7, 8]);
          const image = new ImageData(input, 1);
          input[0] = 99;
          document.getElementById('result').textContent = [
            image.width,
            image.height,
            image.data.length,
            image.data[0],
            input[0],
            image.pixelFormat
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:2:8:1:99:rgba-unorm8")?;
    Ok(())
}

#[test]
fn image_data_constructor_accepts_float16_data_and_display_p3() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const data = new Float16Array(8);
          data[0] = 1.5;
          const image = new ImageData(data, 1, {
            colorSpace: 'display-p3',
            pixelFormat: 'rgba-float16'
          });
          document.getElementById('result').textContent = [
            image.height,
            image.colorSpace,
            image.pixelFormat,
            image.data.length,
            image.data[0]
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "2:display-p3:rgba-float16:8:1.5")?;
    Ok(())
}

#[test]
fn image_data_constructor_rejects_mismatched_pixel_format_and_data_kind() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let threw = false;
          try {
            new ImageData(new Uint8ClampedArray(4), 1, { pixelFormat: 'rgba-float16' });
          } catch (error) {
            threw = String(error).includes('pixelFormat does not match data typed array kind');
          }
          document.getElementById('result').textContent = String(threw);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn canvas_context_image_data_includes_color_space_and_pixel_format() -> Result<()> {
    let html = r#"
        <canvas id='canvas'></canvas>
        <p id='result'></p>
        <script>
          const ctx = document.getElementById('canvas').getContext('2d');
          const imageA = ctx.createImageData(2, 1);
          const imageB = ctx.getImageData(0, 0, 2, 1);
          imageA.data[0] = 128;
          document.getElementById('result').textContent = [
            imageA.data.length,
            imageA.data[0],
            imageA.colorSpace,
            imageA.pixelFormat,
            imageB.colorSpace,
            imageB.pixelFormat
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "8:128:srgb:rgba-unorm8:srgb:rgba-unorm8")?;
    Ok(())
}

#[test]
fn image_data_constructor_rejects_non_object_settings() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let threw = false;
          try {
            new ImageData(1, 1, 123);
          } catch (error) {
            threw = String(error).includes('settings argument must be an object');
          }
          document.getElementById('result').textContent = String(threw);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true")?;
    Ok(())
}
