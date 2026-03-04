use super::*;

#[test]
fn canvas_rendering_context_2d_exposes_core_defaults() -> Result<()> {
    let html = r#"
        <canvas id='canvas' width='100' height='80'></canvas>
        <p id='result'></p>
        <script>
          const canvas = document.getElementById('canvas');
          const ctx = canvas.getContext('2d');
          const attrs = ctx.getContextAttributes();
          const dash = ctx.getLineDash();
          document.getElementById('result').textContent = [
            ctx.canvas === canvas,
            ctx.fillStyle,
            ctx.strokeStyle,
            ctx.lineWidth,
            ctx.lineCap,
            ctx.lineJoin,
            ctx.miterLimit,
            ctx.font,
            ctx.textAlign,
            ctx.textBaseline,
            ctx.direction,
            ctx.globalAlpha,
            ctx.globalCompositeOperation,
            ctx.imageSmoothingEnabled,
            ctx.imageSmoothingQuality,
            ctx.filter,
            attrs.alpha,
            ctx.isContextLost(),
            dash.length,
            typeof ctx.fillRect,
            typeof ctx.createLinearGradient,
            ctx.toString()
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true:#000000:#000000:1:butt:miter:10:10px sans-serif:start:alphabetic:inherit:1:source-over:true:low:none:true:false:0:function:function:[object Object]",
    )?;
    Ok(())
}

#[test]
fn canvas_rendering_context_2d_tracks_dash_transform_and_image_data_calls() -> Result<()> {
    let html = r#"
        <canvas id='canvas' width='100' height='80'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const canvas = document.getElementById('canvas');
            const ctx = canvas.getContext('2d');

            ctx.setLineDash([5, 2, 1]);
            const dash = ctx.getLineDash().join(',');

            ctx.setTransform(1, 0, 0, 1, 12, 34);
            ctx.transform(2, 0, 0, 2, 1, 1);
            const t1 = ctx.getTransform();

            ctx.setTransform({ a: 3, d: 4, e: 5, f: 6 });
            const t2 = ctx.getTransform();

            const textWidth = ctx.measureText('abcd').width;
            const gradient = ctx.createLinearGradient(0, 0, 10, 10);
            gradient.addColorStop(0, 'red');
            const pattern = ctx.createPattern(canvas, 'repeat');
            pattern.setTransform({});

            const imageA = ctx.createImageData(2, 3);
            const imageB = ctx.getImageData(0, 0, 4, 1);
            const imageC = ctx.createImageData(imageB);

            ctx.putImageData(imageA, 0, 0);
            ctx.fillText('a', 1, 2);
            ctx.strokeText('b', 3, 4, 100);

            ctx.beginPath();
            ctx.moveTo(0, 0);
            ctx.lineTo(10, 10);
            ctx.bezierCurveTo(1, 1, 2, 2, 3, 3);
            ctx.quadraticCurveTo(1, 1, 2, 2);
            ctx.arc(5, 5, 2, 0, 3.14);
            ctx.arcTo(1, 1, 2, 2, 3);
            ctx.ellipse(5, 5, 2, 3, 0, 0, 3.14);
            ctx.rect(0, 0, 4, 4);
            ctx.roundRect(0, 0, 4, 4, 1);
            ctx.fill();
            ctx.stroke();
            ctx.clip();
            ctx.drawImage(canvas, 0, 0);

            document.getElementById('result').textContent = [
              dash,
              t1.a,
              t1.d,
              t1.e,
              t1.f,
              t2.a,
              t2.d,
              t2.e,
              t2.f,
              textWidth,
              typeof gradient.addColorStop,
              pattern !== null,
              imageA.width,
              imageA.height,
              imageA.data.length,
              imageB.data.length,
              imageC.width,
              imageC.height,
              ctx.isPointInPath(1, 1),
              ctx.isPointInStroke(1, 1)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "5,2,1,5,2,1:2:2:13:35:3:4:5:6:40:function:true:2:3:24:16:4:1:false:false",
    )?;
    Ok(())
}

#[test]
fn canvas_rendering_context_2d_reset_and_property_normalization_work() -> Result<()> {
    let html = r#"
        <canvas id='canvas' width='100' height='80'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const canvas = document.getElementById('canvas');
            const ctx = canvas.getContext('2d');

            ctx.globalAlpha = 2;
            ctx.lineWidth = -1;
            ctx.lineCap = 'invalid';
            ctx.imageSmoothingEnabled = 0;
            ctx.imageSmoothingQuality = 'high';

            const normalized = [
              ctx.globalAlpha,
              ctx.lineWidth,
              ctx.lineCap,
              ctx.imageSmoothingEnabled,
              ctx.imageSmoothingQuality
            ].join(',');

            ctx.lineWidth = 9;
            ctx.lineCap = 'round';
            ctx.fillStyle = 'green';
            ctx.globalAlpha = 0.3;
            ctx.filter = 'blur(1px)';
            ctx.setLineDash([4, 1]);
            ctx.setTransform(2, 0, 0, 2, 9, 9);
            ctx.reset();

            const t = ctx.getTransform();
            document.getElementById('result').textContent = [
              normalized,
              ctx.lineWidth,
              ctx.lineCap,
              ctx.fillStyle,
              ctx.globalAlpha,
              ctx.filter,
              ctx.getLineDash().length,
              t.a,
              t.d,
              t.e,
              t.f
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1,1,butt,false,high:1:butt:#000000:1:none:0:1:1:0:0",
    )?;
    Ok(())
}

#[test]
fn canvas_rendering_context_2d_draw_image_accepts_image_bitmap_sources() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        input.addEventListener('change', async () => {
          try {
            const file = input.files[0];
            const bmp = await createImageBitmap(file);
            const canvas = document.createElement('canvas');
            canvas.width = bmp.width;
            canvas.height = bmp.height;
            const ctx = canvas.getContext('2d');
            if (!ctx) {
              document.getElementById('out').textContent = 'noctx';
              return;
            }
            ctx.drawImage(bmp, 0, 0);
            document.getElementById('out').textContent =
              `${typeof ctx.drawImage}:${canvas.width}x${canvas.height}`;
          } catch (e) {
            document.getElementById('out').textContent = 'error:' + String(e);
          }
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
    h.assert_text("#out", "function:2x3")?;
    Ok(())
}
