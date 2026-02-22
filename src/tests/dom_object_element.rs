use super::*;

#[test]
fn object_data_type_size_name_and_fallback_content_work() -> Result<()> {
    let html = r#"
        <object
          id='asset'
          type='video/mp4'
          data='/shared-assets/videos/flower.mp4'
          width='250'
          height='200'
          name='promo'>
          <img
            id='fallback'
            src='/shared-assets/images/examples/flowers.jpg'
            alt='Some beautiful flowers'>
        </object>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const asset = document.getElementById('asset');
            const fallback = document.getElementById('fallback');

            const initial =
              asset.role + ':' +
              asset.tagName + ':' +
              asset.getAttribute('data') + ':' +
              asset.type + ':' +
              asset.width + ':' +
              asset.height + ':' +
              asset.name + ':' +
              fallback.tagName + ':' +
              fallback.getAttribute('alt');

            asset.setAttribute('data', '/media/movie.webm');
            asset.type = 'video/webm';
            asset.width = 600;
            asset.height = 140;
            asset.name = 'promo-player';
            asset.setAttribute('usemap', '#infographic');

            const assigned =
              asset.getAttribute('data') + ':' +
              asset.type + ':' +
              asset.width + ':' +
              asset.height + ':' +
              asset.getAttribute('width') + ':' +
              asset.getAttribute('height') + ':' +
              asset.name + ':' +
              asset.getAttribute('name') + ':' +
              asset.getAttribute('usemap');

            document.getElementById('result').textContent = initial + '|' + assigned;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":OBJECT:/shared-assets/videos/flower.mp4:video/mp4:250:200:promo:IMG:Some beautiful flowers|/media/movie.webm:video/webm:600:140:600:140:promo-player:promo-player:#infographic",
    )?;
    Ok(())
}

#[test]
fn object_role_override_form_attr_and_transparent_content_work() -> Result<()> {
    let html = r#"
        <form id='upload-form' action='#'></form>
        <object id='control' form='upload-form' data='/bin/resource.bin' type='application/octet-stream'>
          <p id='inside'>Fallback <a id='alt-link' href='/fallback'>link</a></p>
        </object>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const control = document.getElementById('control');
            const inside = document.getElementById('inside');

            const initial =
              control.role + ':' +
              control.getAttribute('form') + ':' +
              inside.textContent.replace(/\s+/g, ' ').trim() + ':' +
              document.getElementById('alt-link').href + ':' +
              control.querySelectorAll('param').length;

            control.role = 'img';
            const assigned = control.role + ':' + control.getAttribute('role');
            control.removeAttribute('role');
            const restored = control.role + ':' + (control.getAttribute('role') === null);

            control.setAttribute('archive', '/a.jar /b.jar');
            control.setAttribute('standby', 'Loading...');
            const deprecated =
              control.getAttribute('archive').includes('/a.jar') + ':' +
              control.getAttribute('standby');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + deprecated;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":upload-form:Fallback link:https://app.local/fallback:0|img:img|:true|true:Loading...",
    )?;
    Ok(())
}
