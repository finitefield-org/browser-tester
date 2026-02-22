use super::*;

#[test]
fn audio_src_and_core_media_attributes_reflect_via_properties() -> Result<()> {
    let html = r#"
        <audio id='player'></audio>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('player').src = '/media/theme.mp3';
            document.getElementById('player').autoplay = true;
            document.getElementById('player').controls = true;
            document.getElementById('player').controlsList = 'nodownload noremoteplayback';
            document.getElementById('player').crossOrigin = 'anonymous';
            document.getElementById('player').disableRemotePlayback = true;
            document.getElementById('player').loop = true;
            document.getElementById('player').muted = true;
            document.getElementById('player').preload = 'metadata';

            document.getElementById('result').textContent =
              document.getElementById('player').src + '|' +
              document.getElementById('player').autoplay + '|' +
              document.getElementById('player').controls + '|' +
              document.getElementById('player').controlsList + '|' +
              document.getElementById('player').crossOrigin + '|' +
              document.getElementById('player').disableRemotePlayback + '|' +
              document.getElementById('player').loop + '|' +
              document.getElementById('player').muted + '|' +
              document.getElementById('player').preload + '|' +
              document.getElementById('player').getAttribute('controlslist') + '|' +
              document.getElementById('player').getAttribute('disableremoteplayback');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/media/theme.mp3|true|true|nodownload noremoteplayback|anonymous|true|true|true|metadata|nodownload noremoteplayback|true",
    )?;
    Ok(())
}

#[test]
fn audio_src_uses_first_nested_source_when_src_attribute_is_missing() -> Result<()> {
    let html = r#"
        <audio id='player'>
          <source src='/audio/primary.ogg' type='audio/ogg'>
          <source src='/audio/backup.mp3' type='audio/mpeg'>
          fallback
        </audio>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = document.getElementById('player').src;

            document.getElementById('player').controls = true;
            document.getElementById('player').autoplay = true;
            document.getElementById('player').muted = true;
            document.getElementById('player').controls = false;
            document.getElementById('player').autoplay = false;
            document.getElementById('player').muted = false;

            document.getElementById('result').textContent =
              before + '|' +
              document.getElementById('player').controls + ':' +
              (document.getElementById('player').getAttribute('controls') === null) + '|' +
              document.getElementById('player').autoplay + ':' +
              (document.getElementById('player').getAttribute('autoplay') === null) + '|' +
              document.getElementById('player').muted + ':' +
              (document.getElementById('player').getAttribute('muted') === null) + '|' +
              document.querySelectorAll('audio source').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://example.com/base/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/audio/primary.ogg|false:true|false:true|false:true|2",
    )?;
    Ok(())
}

#[test]
fn audio_has_no_implicit_role_and_supports_explicit_role_assignment() -> Result<()> {
    let html = r#"
        <audio id='player' src='/media/theme.mp3'></audio>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initial = document.getElementById('player').role;
            document.getElementById('player').role = 'application';
            const assigned = document.getElementById('player').role + ':' +
              document.getElementById('player').getAttribute('role');
            document.getElementById('player').removeAttribute('role');
            const restored = document.getElementById('player').role + ':' +
              (document.getElementById('player').getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "|application:application|:true")?;
    Ok(())
}
