use super::*;

#[test]
fn img_implicit_roles_and_core_properties_reflect_correctly() -> Result<()> {
    let html = r#"
        <img
          id='photo'
          src='/shared-assets/images/examples/grapefruit-slice.jpg'
          alt='Grapefruit slice atop a pile of other slices'>
        <img id='decor' src='/shared-assets/images/examples/favicon72.png' alt=''>
        <img id='missing' src='/shared-assets/images/examples/favicon144.png'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const photo = document.getElementById('photo');
            const decor = document.getElementById('decor');
            const missing = document.getElementById('missing');

            const initial =
              photo.role + ':' +
              decor.role + ':' +
              missing.role + ':' +
              photo.src + ':' +
              photo.width + 'x' + photo.height + ':' +
              photo.getAttribute('alt');

            photo.width = 144;
            photo.height = 96;
            photo.crossOrigin = 'anonymous';
            photo.referrerPolicy = 'no-referrer';
            photo.setAttribute('srcset', '/img-1x.png 1x, /img-2x.png 2x');
            photo.setAttribute('sizes', '(max-width: 600px) 200px, 50vw');
            photo.setAttribute('loading', 'lazy');

            const assigned =
              photo.width + 'x' + photo.height + ':' +
              photo.getAttribute('width') + ':' +
              photo.getAttribute('height') + ':' +
              photo.crossOrigin + ':' +
              photo.getAttribute('crossorigin') + ':' +
              photo.referrerPolicy + ':' +
              photo.getAttribute('referrerpolicy') + ':' +
              photo.getAttribute('srcset').includes('2x') + ':' +
              photo.getAttribute('sizes').includes('50vw') + ':' +
              photo.getAttribute('loading');

            document.getElementById('result').textContent = initial + '|' + assigned;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "img:presentation:img:https://app.local/shared-assets/images/examples/grapefruit-slice.jpg:0x0:Grapefruit slice atop a pile of other slices|144x96:144:96:anonymous:anonymous:no-referrer:no-referrer:true:true:lazy",
    )?;
    Ok(())
}

#[test]
fn img_alt_dependent_role_and_role_attribute_roundtrip_work() -> Result<()> {
    let html = r#"
        <img id='target' src='/shared-assets/images/examples/favicon72.png' alt=''>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');

            const start = target.role + ':' + (target.getAttribute('alt') === '');

            target.setAttribute('alt', 'Company logo');
            const descriptive = target.role + ':' + target.getAttribute('alt');

            target.setAttribute('alt', '');
            const decorative = target.role + ':' + (target.getAttribute('alt') === '');

            target.role = 'button';
            const assigned = target.role + ':' + target.getAttribute('role');
            target.removeAttribute('role');
            const restored = target.role + ':' + (target.getAttribute('role') === null);

            target.removeAttribute('alt');
            const withoutAlt = target.role + ':' + (target.getAttribute('alt') === null);

            document.getElementById('result').textContent =
              start + '|' +
              descriptive + '|' +
              decorative + '|' +
              assigned + '|' +
              restored + '|' +
              withoutAlt;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "presentation:true|img:Company logo|presentation:true|button:button|presentation:true|img:true",
    )?;
    Ok(())
}
