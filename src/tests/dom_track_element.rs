use super::*;

#[test]
fn track_kind_defaults_and_attribute_reflection_work() -> Result<()> {
    let html = r#"
        <video id='player' controls src='/shared-assets/videos/friday.mp4'>
          <track
            id='captions'
            default
            srclang='en'
            label='English'
            src='/shared-assets/misc/friday.vtt'>
          <track
            id='invalid'
            kind='bad-kind'
            src='/shared-assets/misc/friday_alt.vtt'>
        </video>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const captions = document.getElementById('captions');
            const invalid = document.getElementById('invalid');

            const initial =
              captions.tagName + ':' +
              captions.role + ':' +
              captions.kind + ':' +
              captions.srclang + ':' +
              captions.label + ':' +
              captions.default + ':' +
              captions.getAttribute('src') + ':' +
              invalid.kind + ':' +
              document.querySelectorAll('video > track').length;

            captions.kind = 'captions';
            captions.srclang = 'ja';
            captions.label = 'Japanese';
            captions.default = false;

            invalid.kind = 'chapters';
            invalid.default = true;

            const updated =
              captions.kind + ':' +
              captions.getAttribute('kind') + ':' +
              captions.srclang + ':' +
              captions.label + ':' +
              captions.default + ':' +
              (captions.getAttribute('default') === null) + ':' +
              invalid.kind + ':' +
              invalid.getAttribute('kind') + ':' +
              invalid.default + ':' +
              invalid.getAttribute('default');

            document.getElementById('result').textContent = initial + '|' + updated;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "TRACK::subtitles:en:English:true:/shared-assets/misc/friday.vtt:metadata:2|captions:captions:ja:Japanese:false:true:chapters:chapters:true:true",
    )?;
    Ok(())
}

#[test]
fn track_role_override_and_kind_normalization_roundtrip_work() -> Result<()> {
    let html = r#"
        <audio id='audio' controls src='/shared-assets/audio/theme.mp3'>
          <track
            id='meta'
            kind='metadata'
            srclang='en'
            label='Key Stage 1'
            src='/shared-assets/misc/keyStage1.vtt'>
        </audio>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const meta = document.getElementById('meta');

            const initial =
              meta.role + ':' +
              meta.kind + ':' +
              meta.srclang + ':' +
              meta.label + ':' +
              meta.default + ':' +
              (meta.getAttribute('default') === null);

            meta.kind = 'bogus-kind';
            const invalidKind = meta.kind + ':' + meta.getAttribute('kind');

            meta.setAttribute('kind', 'subtitles');
            meta.removeAttribute('srclang');
            meta.default = true;
            const updated =
              meta.kind + ':' +
              meta.default + ':' +
              meta.getAttribute('default') + ':' +
              meta.srclang + ':' +
              (meta.getAttribute('srclang') === null);

            meta.role = 'none';
            const assignedRole = meta.role + ':' + meta.getAttribute('role');
            meta.removeAttribute('role');
            const restoredRole = meta.role + ':' + (meta.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + invalidKind + '|' + updated + '|' + assignedRole + '|' + restoredRole + '|' + meta.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":metadata:en:Key Stage 1:false:true|metadata:bogus-kind|subtitles:true:true::true|none:none|:true|TRACK",
    )?;
    Ok(())
}

#[test]
fn track_src_shadow_define_property_delete_and_generic_lookup_work() -> Result<()> {
    let html = r#"
        <video controls>
          <track id='captions' src='/tracks/base.vtt' kind='captions' srclang='en'>
        </video>
        <p id='result'></p>
        <script>
          const captions = document.getElementById('captions');

          const before = [
            captions.src,
            captions.getAttribute('src')
          ].join(':');

          Object.defineProperty(captions, 'src', {
            value: 'shadow-src',
            writable: true,
            enumerable: true,
            configurable: true
          });

          captions.src = 'set-src';

          const shadowed = [
            captions.src,
            captions.getAttribute('src'),
            String(Object.keys(captions).join(',') === 'src')
          ].join(':');

          delete captions.src;

          const restored = [
            captions.src,
            captions.getAttribute('src'),
            String(Object.hasOwn(captions, 'src'))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/tracks/base.vtt:/tracks/base.vtt|set-src:/tracks/base.vtt:true|https://app.local/tracks/base.vtt:/tracks/base.vtt:false",
    )?;
    Ok(())
}

#[test]
fn track_reflected_boolean_and_string_shadow_define_property_delete_work() -> Result<()> {
    let html = r#"
        <video controls>
          <track
            id='captions'
            kind='captions'
            srclang='en'
            label='English'
            default
            src='/tracks/base.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const captions = document.getElementById('captions');

          const before = [
            captions.kind,
            captions.label,
            captions.srcLang,
            String(captions.default)
          ].join(':');

          Object.defineProperty(captions, 'kind', {
            value: 'shadow-kind',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(captions, 'label', {
            value: 'shadow-label',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(captions, 'srcLang', {
            value: 'shadow-lang',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(captions, 'default', {
            value: false,
            writable: true,
            enumerable: true,
            configurable: true
          });

          captions.kind = 'set-kind';
          captions.label = 'set-label';
          captions.srcLang = 'set-lang';
          captions.default = 'set-default';

          const shadowed = [
            captions.kind,
            captions.label,
            captions.srcLang,
            String(captions.default),
            String(Object.keys(captions).sort().join(',') === 'default,kind,label,srcLang')
          ].join(':');

          delete captions.kind;
          delete captions.label;
          delete captions.srcLang;
          delete captions.default;

          const restored = [
            captions.kind,
            captions.label,
            captions.srcLang,
            String(captions.default),
            String(Object.hasOwn(captions, 'kind')),
            String(Object.hasOwn(captions, 'label')),
            String(Object.hasOwn(captions, 'srcLang')),
            String(Object.hasOwn(captions, 'default'))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "captions:English:en:true|set-kind:set-label:set-lang:set-default:true|captions:English:en:true:false:false:false:false",
    )?;
    Ok(())
}

#[test]
fn track_ready_state_shadow_define_property_delete_work() -> Result<()> {
    let html = r#"
        <video controls>
          <track id='captions' kind='captions' srclang='en' src='/tracks/base.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const captions = document.getElementById('captions');

          const before = String(captions.readyState);

          Object.defineProperty(captions, 'readyState', {
            value: 4,
            writable: true,
            enumerable: true,
            configurable: true
          });

          captions.readyState = 7;

          const shadowed = [
            String(captions.readyState),
            String(Object.keys(captions).join(',') === 'readyState')
          ].join(':');

          delete captions.readyState;

          const restored = [
            String(captions.readyState),
            String(Object.hasOwn(captions, 'readyState'))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "0|7:true|0:false")?;
    Ok(())
}
