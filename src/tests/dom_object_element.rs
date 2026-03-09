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

#[test]
fn object_data_and_usemap_shadow_define_property_delete_and_fast_path_parity_work() -> Result<()> {
    let html = r#"
        <object id='asset' data='/media/movie.webm' usemap='#diagram'></object>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const asset = document.getElementById('asset');
            const shadow = {
              data: 'shadow-data',
              useMap: 'shadow-usemap'
            };

            Object.defineProperty(asset, 'data', {
              get() { return shadow.data; },
              set(value) { shadow.data = 'set:' + value; },
              configurable: true
            });
            Object.defineProperty(asset, 'useMap', {
              get() { return shadow.useMap; },
              set(value) { shadow.useMap = 'set:' + value; },
              configurable: true
            });

            document.getElementById('asset').data = '/next.bin';
            asset.useMap = '#next';

            const first = [
              document.getElementById('asset').data,
              asset['data'],
              shadow.data,
              asset.getAttribute('data'),
              asset.useMap,
              asset['useMap'],
              shadow.useMap,
              asset.getAttribute('usemap')
            ].join(':');

            Reflect.set(asset, 'data', 'reflect.bin');
            Reflect.set(asset, 'useMap', '#reflect');

            const second = [
              document.getElementById('asset').data,
              asset['data'],
              shadow.data,
              asset.getAttribute('data'),
              asset.useMap,
              asset['useMap'],
              shadow.useMap,
              asset.getAttribute('usemap')
            ].join(':');

            delete asset.data;
            delete asset.useMap;

            const third = [
              document.getElementById('asset').data,
              asset['data'],
              asset.getAttribute('data'),
              asset.useMap,
              asset['useMap'],
              asset.getAttribute('usemap')
            ].join(':');

            document.getElementById('result').textContent = [
              first,
              second,
              third
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/base/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "set:/next.bin:set:/next.bin:set:/next.bin:/media/movie.webm:set:#next:set:#next:set:#next:#diagram|set:reflect.bin:set:reflect.bin:set:reflect.bin:/media/movie.webm:set:#reflect:set:#reflect:set:#reflect:#diagram|https://app.local/media/movie.webm:https://app.local/media/movie.webm:/media/movie.webm:#diagram:#diagram:#diagram",
    )?;
    Ok(())
}

#[test]
fn object_reflective_own_property_surface_and_object_copy_work() -> Result<()> {
    let html = r#"
        <object id='asset' data='/media/movie.webm' usemap='#diagram'></object>
        <p id='result'></p>
        <script>
          const asset = document.getElementById('asset');
          const beforeAssigned = Object.assign({}, asset);
          const beforeSpread = { ...asset };

          const before = [
            asset.data,
            asset.useMap,
            String(Object.hasOwn(asset, 'data')),
            String(Object.hasOwn(asset, 'useMap')),
            String(Object.getOwnPropertyDescriptor(asset, 'data') === undefined),
            String(Object.getOwnPropertyDescriptor(asset, 'useMap') === undefined),
            String(Object.getOwnPropertyNames(asset).includes('data')),
            String(Reflect.ownKeys(asset).includes('useMap')),
            String('data' in beforeAssigned),
            String('useMap' in beforeSpread)
          ].join(':');

          Object.defineProperty(asset, 'data', {
            value: 'shadow-data',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(asset, 'useMap', {
            value: 'shadow-usemap',
            writable: true,
            enumerable: true,
            configurable: true
          });
          asset.extra = 'expando';

          const shadowAssigned = Object.assign({}, asset);
          const shadowSpread = { ...asset };

          const shadowed = [
            asset.data,
            asset.useMap,
            String(Object.keys(asset).join(',') === 'data,extra,useMap'),
            shadowAssigned.data,
            shadowAssigned.useMap,
            shadowAssigned.extra,
            shadowSpread.data,
            shadowSpread.useMap,
            shadowSpread.extra
          ].join(':');

          delete asset.data;
          delete asset.useMap;

          const restoredAssigned = Object.assign({}, asset);
          const restoredSpread = { ...asset };

          const restored = [
            asset.data,
            asset.useMap,
            String(Object.hasOwn(asset, 'data')),
            String(Object.hasOwn(asset, 'useMap')),
            restoredAssigned.extra,
            String('data' in restoredAssigned),
            String('useMap' in restoredAssigned),
            restoredSpread.extra,
            String('data' in restoredSpread),
            String('useMap' in restoredSpread)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/base/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/media/movie.webm:#diagram:false:false:true:true:false:false:false:false|shadow-data:shadow-usemap:true:shadow-data:shadow-usemap:expando:shadow-data:shadow-usemap:expando|https://app.local/media/movie.webm:#diagram:false:false:expando:false:false:expando:false:false",
    )?;
    Ok(())
}
