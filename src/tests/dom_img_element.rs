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

#[test]
fn img_reflective_own_property_surface_and_object_copy_work() -> Result<()> {
    let html = r#"
        <img
          id='photo'
          src='/shared-assets/images/examples/grapefruit-slice.jpg'
          srcset='/img-1x.png 1x, /img-2x.png 2x'
          sizes='100vw'>
        <p id='result'></p>
        <script>
          const photo = document.getElementById('photo');
          const beforeAssigned = Object.assign({}, photo);
          const beforeSpread = { ...photo };

          const before = [
            photo.src,
            photo.srcset,
            photo.sizes,
            String(Object.hasOwn(photo, 'src')),
            String(Object.hasOwn(photo, 'srcset')),
            String(Object.hasOwn(photo, 'sizes')),
            String(Object.getOwnPropertyDescriptor(photo, 'src') === undefined),
            String(Object.getOwnPropertyNames(photo).includes('srcset')),
            String(Reflect.ownKeys(photo).includes('sizes')),
            String('src' in beforeAssigned),
            String('srcset' in beforeSpread)
          ].join(':');

          Object.defineProperty(photo, 'src', {
            value: 'shadow-src',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(photo, 'srcset', {
            value: 'shadow-srcset',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(photo, 'sizes', {
            value: 'shadow-sizes',
            writable: true,
            enumerable: true,
            configurable: true
          });
          photo.extra = 'expando';

          const shadowAssigned = Object.assign({}, photo);
          const shadowSpread = { ...photo };

          const shadowed = [
            photo.src,
            photo.srcset,
            photo.sizes,
            String(Object.keys(photo).sort().join(',') === 'extra,sizes,src,srcset'),
            shadowAssigned.src,
            shadowAssigned.srcset,
            shadowAssigned.sizes,
            shadowAssigned.extra,
            shadowSpread.src,
            shadowSpread.srcset,
            shadowSpread.sizes,
            shadowSpread.extra
          ].join(':');

          delete photo.src;
          delete photo.srcset;
          delete photo.sizes;

          const restoredAssigned = Object.assign({}, photo);
          const restoredSpread = { ...photo };

          const restored = [
            photo.src,
            photo.srcset,
            photo.sizes,
            String(Object.hasOwn(photo, 'src')),
            String(Object.hasOwn(photo, 'srcset')),
            String(Object.hasOwn(photo, 'sizes')),
            restoredAssigned.extra,
            String('src' in restoredAssigned),
            String('srcset' in restoredAssigned),
            String('sizes' in restoredAssigned),
            restoredSpread.extra,
            String('src' in restoredSpread),
            String('srcset' in restoredSpread),
            String('sizes' in restoredSpread)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/shared-assets/images/examples/grapefruit-slice.jpg:/img-1x.png 1x, /img-2x.png 2x:100vw:false:false:false:true:false:false:false:false|shadow-src:shadow-srcset:shadow-sizes:true:shadow-src:shadow-srcset:shadow-sizes:expando:shadow-src:shadow-srcset:shadow-sizes:expando|https://app.local/shared-assets/images/examples/grapefruit-slice.jpg:/img-1x.png 1x, /img-2x.png 2x:100vw:false:false:false:expando:false:false:false:expando:false:false:false",
    )?;
    Ok(())
}

#[test]
fn img_cross_origin_and_current_src_shadow_define_property_delete_and_fast_path_parity_work()
-> Result<()> {
    let html = r#"
        <picture>
          <source srcset='/img/hero.webp 1x' type='image/webp'>
          <img id='photo' src='/img/fallback.jpg' crossorigin='anonymous' alt='Artwork'>
        </picture>
        <p id='result'></p>
        <script>
          const photo = document.getElementById('photo');

          const before = [
            photo.crossOrigin,
            photo.currentSrc,
            photo.getAttribute('crossorigin')
          ].join(':');

          Object.defineProperty(photo, 'crossOrigin', {
            value: 'shadow-cors',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(photo, 'currentSrc', {
            value: 'shadow-current',
            writable: true,
            enumerable: true,
            configurable: true
          });

          photo.crossOrigin = 'set-cors';
          photo.currentSrc = 'set-current';

          const shadowed = [
            photo.crossOrigin,
            photo.currentSrc,
            photo.getAttribute('crossorigin'),
            String(Object.keys(photo).sort().join(',') === 'crossOrigin,currentSrc')
          ].join(':');

          delete photo.crossOrigin;
          delete photo.currentSrc;

          const restored = [
            photo.crossOrigin,
            photo.currentSrc,
            photo.getAttribute('crossorigin'),
            String(Object.hasOwn(photo, 'crossOrigin')),
            String(Object.hasOwn(photo, 'currentSrc'))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/gallery/index.html", html)?;
    h.assert_text(
        "#result",
        "anonymous:https://app.local/img/hero.webp:anonymous|set-cors:set-current:anonymous:true|anonymous:https://app.local/img/hero.webp:anonymous:false:false",
    )?;
    Ok(())
}

#[test]
fn img_load_facing_state_and_picture_current_src_matrix_work() -> Result<()> {
    let html = r#"
        <picture>
          <source id='hero' srcset='/img/hero.webp 1x' media='(width >= 900px)' type='image/webp'>
          <img id='photo' src='/img/fallback.jpg' alt='Artwork'>
        </picture>
        <p id='result'></p>
        <script>
          const hero = document.getElementById('hero');
          const photo = document.getElementById('photo');

          const snapshot = () => [
            String(photo.complete),
            String(photo.naturalWidth),
            String(photo.naturalHeight),
            photo.currentSrc
          ].join(':');

          window.innerWidth = 500;
          const initial = snapshot();

          window.innerWidth = 1200;
          const matched = snapshot();

          hero.type = 'video/mp4';
          const unsupported = snapshot();

          hero.type = 'image/webp';
          hero.media = '(width >= 1500px)';
          const filtered = snapshot();

          photo.removeAttribute('src');
          hero.removeAttribute('srcset');
          const empty = snapshot();

          document.getElementById('result').textContent = [
            initial,
            matched,
            unsupported,
            filtered,
            empty
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/gallery/index.html", html)?;
    h.assert_text(
        "#result",
        "true:1:1:https://app.local/img/fallback.jpg|true:1:1:https://app.local/img/hero.webp|true:1:1:https://app.local/img/fallback.jpg|true:1:1:https://app.local/img/fallback.jpg|true:0:0:",
    )?;
    Ok(())
}
