use super::*;

#[test]
fn embed_src_type_size_and_title_roundtrip_work() -> Result<()> {
    let html = r#"
        <embed
          id='asset'
          type='image/jpeg'
          src='/shared-assets/images/examples/flowers.jpg'
          width='250'
          height='200'
          title='Flowers preview'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const asset = document.getElementById('asset');
            const initial =
              asset.role + ':' +
              asset.tagName + ':' +
              asset.src + ':' +
              asset.type + ':' +
              asset.width + ':' +
              asset.height + ':' +
              asset.getAttribute('title');

            asset.src = '/media/movie.mov';
            asset.type = 'video/quicktime';
            asset.width = 640;
            asset.height = 480;
            asset.setAttribute('title', 'Title of my video');

            const assigned =
              asset.src + ':' +
              asset.type + ':' +
              asset.width + ':' +
              asset.height + ':' +
              asset.getAttribute('src') + ':' +
              asset.getAttribute('type') + ':' +
              asset.getAttribute('width') + ':' +
              asset.getAttribute('height') + ':' +
              asset.getAttribute('title');

            document.getElementById('result').textContent = initial + '|' + assigned;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/page/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":EMBED:https://app.local/shared-assets/images/examples/flowers.jpg:image/jpeg:250:200:Flowers preview|https://app.local/media/movie.mov:video/quicktime:640:480:/media/movie.mov:video/quicktime:640:480:Title of my video",
    )?;
    Ok(())
}

#[test]
fn embed_is_void_and_role_assignment_roundtrip_work() -> Result<()> {
    let html = r#"
        <p id='line'>
          before
          <embed id='asset' src='/x.bin' type='application/octet-stream' width='1' height='1'>
          after
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const line = document.getElementById('line');
            const asset = document.getElementById('asset');
            const initial =
              asset.role + ':' +
              line.childElementCount + ':' +
              line.textContent.replace(/\s+/g, ' ').trim();

            asset.role = 'img';
            const assigned = asset.role + ':' + asset.getAttribute('role');
            asset.removeAttribute('role');
            const restored = asset.role + ':' + (asset.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":1:before after|img:img|:true")?;
    Ok(())
}
