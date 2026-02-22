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
