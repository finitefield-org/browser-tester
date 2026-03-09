use super::*;

#[test]
fn iframe_defaults_and_src_name_reflection_work() -> Result<()> {
    let html = r#"
        <iframe id='frame' title='Inline Frame Example' src='/embedded/page.html'></iframe>
        <iframe id='blank'></iframe>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const frame = document.getElementById('frame');
            const blank = document.getElementById('blank');
            const initial =
              blank.width + 'x' + blank.height + ':' +
              frame.src + ':' +
              frame.getAttribute('src') + ':' +
              frame.role + ':' +
              frame.getAttribute('title');

            blank.width = 640;
            blank.height = 360;
            frame.src = '/next.html';
            frame.name = 'inlineFrameExample';

            const assigned =
              blank.width + 'x' + blank.height + ':' +
              blank.getAttribute('width') + ':' +
              blank.getAttribute('height') + ':' +
              frame.src + ':' +
              frame.getAttribute('src') + ':' +
              frame.name + ':' +
              frame.getAttribute('name');

            document.getElementById('result').textContent = initial + '|' + assigned;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "300x150:https://app.local/embedded/page.html:/embedded/page.html::Inline Frame Example|640x360:640:360:https://app.local/next.html:/next.html:inlineFrameExample:inlineFrameExample",
    )?;
    Ok(())
}

#[test]
fn iframe_referrerpolicy_sandbox_and_role_roundtrip_work() -> Result<()> {
    let html = r#"
        <iframe id='frame' sandbox='allow-forms' srcdoc='<p>Hello</p>'></iframe>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const frame = document.getElementById('frame');
            const initial =
              frame.role + ':' +
              frame.referrerPolicy + ':' +
              frame.getAttribute('sandbox') + ':' +
              frame.getAttribute('srcdoc').includes('<p>');

            frame.referrerPolicy = 'no-referrer';
            frame.setAttribute('sandbox', 'allow-scripts allow-same-origin');
            frame.setAttribute('loading', 'lazy');
            const assigned =
              frame.referrerPolicy + ':' +
              frame.getAttribute('referrerpolicy') + ':' +
              frame.getAttribute('sandbox') + ':' +
              frame.getAttribute('loading');

            frame.role = 'document';
            const roleAssigned = frame.role + ':' + frame.getAttribute('role');
            frame.removeAttribute('role');
            const roleRestored = frame.role + ':' + (frame.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + roleAssigned + '|' + roleRestored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "::allow-forms:true|no-referrer:no-referrer:allow-scripts allow-same-origin:lazy|document:document|:true",
    )?;
    Ok(())
}

#[test]
fn iframe_srcdoc_shadow_define_property_delete_and_fast_path_parity_work() -> Result<()> {
    let html = r#"
        <iframe id='frame' srcdoc='<p>Hello</p>'></iframe>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const frame = document.getElementById('frame');
            const shadow = { srcdoc: 'shadow-srcdoc' };

            Object.defineProperty(frame, 'srcdoc', {
              get() { return shadow.srcdoc; },
              set(value) { shadow.srcdoc = 'set:' + value; },
              configurable: true
            });

            document.getElementById('frame').srcdoc = '<p>Next</p>';

            const first = [
              document.getElementById('frame').srcdoc,
              frame['srcdoc'],
              shadow.srcdoc,
              frame.getAttribute('srcdoc')
            ].join(':');

            Reflect.set(frame, 'srcdoc', '<p>Reflect</p>');

            const second = [
              document.getElementById('frame').srcdoc,
              frame['srcdoc'],
              shadow.srcdoc,
              frame.getAttribute('srcdoc')
            ].join(':');

            delete frame.srcdoc;

            const third = [
              document.getElementById('frame').srcdoc,
              frame['srcdoc'],
              frame.getAttribute('srcdoc')
            ].join(':');

            document.getElementById('result').textContent = [
              first,
              second,
              third
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "set:<p>Next</p>:set:<p>Next</p>:set:<p>Next</p>:<p>Hello</p>|set:<p>Reflect</p>:set:<p>Reflect</p>:set:<p>Reflect</p>:<p>Hello</p>|<p>Hello</p>:<p>Hello</p>:<p>Hello</p>",
    )?;
    Ok(())
}

#[test]
fn iframe_reflective_own_property_surface_and_object_copy_work() -> Result<()> {
    let html = r#"
        <iframe id='frame' src='/embedded/page.html' srcdoc='<p>Hello</p>'></iframe>
        <p id='result'></p>
        <script>
          const frame = document.getElementById('frame');
          const beforeAssigned = Object.assign({}, frame);
          const beforeSpread = { ...frame };

          const before = [
            frame.src,
            frame.srcdoc,
            String(Object.hasOwn(frame, 'src')),
            String(Object.hasOwn(frame, 'srcdoc')),
            String(Object.getOwnPropertyDescriptor(frame, 'src') === undefined),
            String(Object.getOwnPropertyDescriptor(frame, 'srcdoc') === undefined),
            String(Object.getOwnPropertyNames(frame).includes('src')),
            String(Reflect.ownKeys(frame).includes('srcdoc')),
            String('src' in beforeAssigned),
            String('srcdoc' in beforeSpread)
          ].join(':');

          Object.defineProperty(frame, 'src', {
            value: 'shadow-src',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(frame, 'srcdoc', {
            value: 'shadow-srcdoc',
            writable: true,
            enumerable: true,
            configurable: true
          });
          frame.extra = 'expando';

          const shadowAssigned = Object.assign({}, frame);
          const shadowSpread = { ...frame };

          const shadowed = [
            frame.src,
            frame.srcdoc,
            String(Object.keys(frame).join(',') === 'extra,src,srcdoc'),
            shadowAssigned.src,
            shadowAssigned.srcdoc,
            shadowAssigned.extra,
            shadowSpread.src,
            shadowSpread.srcdoc,
            shadowSpread.extra
          ].join(':');

          delete frame.src;
          delete frame.srcdoc;

          const restoredAssigned = Object.assign({}, frame);
          const restoredSpread = { ...frame };

          const restored = [
            frame.src,
            frame.srcdoc,
            String(Object.hasOwn(frame, 'src')),
            String(Object.hasOwn(frame, 'srcdoc')),
            restoredAssigned.extra,
            String('src' in restoredAssigned),
            String('srcdoc' in restoredAssigned),
            restoredSpread.extra,
            String('src' in restoredSpread),
            String('srcdoc' in restoredSpread)
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
        "https://app.local/embedded/page.html:<p>Hello</p>:false:false:true:true:false:false:false:false|shadow-src:shadow-srcdoc:true:shadow-src:shadow-srcdoc:expando:shadow-src:shadow-srcdoc:expando|https://app.local/embedded/page.html:<p>Hello</p>:false:false:expando:false:false:expando:false:false",
    )?;
    Ok(())
}

#[test]
fn iframe_manual_load_error_dispatch_and_resource_surface_work() -> Result<()> {
    let html = r#"
        <iframe id='frame' src='/embedded/page.html' srcdoc='<p>Hello</p>'></iframe>
        <p id='result'></p>
        <script>
          const frame = document.getElementById('frame');
          const log = [];
          const render = () => {
            document.getElementById('result').textContent = [
              log.join(','),
              frame.src,
              frame.srcdoc
            ].join('|');
          };

          frame.onload = (event) => {
            log.push('load:' + event.type + ':' + String(event.currentTarget === frame));
            render();
          };
          frame.addEventListener('error', (event) => {
            log.push('error:' + event.type + ':' + String(event.currentTarget === frame));
            render();
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/index.html", html)?;
    h.dispatch("#frame", "load")?;
    h.dispatch("#frame", "error")?;
    h.assert_text(
        "#result",
        "load:load:true,error:error:true|https://app.local/embedded/page.html|<p>Hello</p>",
    )?;
    Ok(())
}
