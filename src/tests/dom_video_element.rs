use super::*;

#[test]
fn video_src_media_and_video_specific_attributes_reflect_via_properties() -> Result<()> {
    let html = r#"
        <video id='player'></video>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const player = document.getElementById('player');
            player.src = '/media/clip.mp4';
            player.autoplay = true;
            player.controls = true;
            player.controlsList = 'nodownload nofullscreen';
            player.crossOrigin = 'anonymous';
            player.disableRemotePlayback = true;
            player.disablePictureInPicture = true;
            player.loop = true;
            player.muted = true;
            player.playsInline = true;
            player.poster = '/img/poster.jpg';
            player.preload = 'metadata';
            player.width = 640;
            player.height = 360;

            document.getElementById('result').textContent =
              player.src + '|' +
              player.autoplay + '|' +
              player.controls + '|' +
              player.controlsList + '|' +
              player.crossOrigin + '|' +
              player.disableRemotePlayback + '|' +
              player.disablePictureInPicture + '|' +
              player.loop + '|' +
              player.muted + '|' +
              player.playsInline + '|' +
              player.poster + '|' +
              player.preload + '|' +
              player.width + 'x' + player.height + '|' +
              player.getAttribute('disablepictureinpicture') + '|' +
              player.getAttribute('playsinline') + '|' +
              player.getAttribute('poster');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/media/clip.mp4|true|true|nodownload nofullscreen|anonymous|true|true|true|true|true|https://app.local/img/poster.jpg|metadata|640x360|true|true|/img/poster.jpg",
    )?;
    Ok(())
}

#[test]
fn video_uses_first_source_when_src_missing_and_boolean_toggles_remove_attributes() -> Result<()> {
    let html = r#"
        <video id='player'>
          <source src='/video/primary.webm' type='video/webm'>
          <source src='/video/backup.mp4' type='video/mp4'>
          <track kind='captions' srclang='en' src='/video/captions.vtt'>
          fallback text
        </video>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const player = document.getElementById('player');
            const before = player.src;

            player.controls = true;
            player.autoplay = true;
            player.muted = true;
            player.loop = true;
            player.playsInline = true;
            player.disablePictureInPicture = true;

            player.controls = false;
            player.autoplay = false;
            player.muted = false;
            player.loop = false;
            player.playsInline = false;
            player.disablePictureInPicture = false;

            document.getElementById('result').textContent =
              before + '|' +
              player.controls + ':' + (player.getAttribute('controls') === null) + '|' +
              player.autoplay + ':' + (player.getAttribute('autoplay') === null) + '|' +
              player.muted + ':' + (player.getAttribute('muted') === null) + '|' +
              player.loop + ':' + (player.getAttribute('loop') === null) + '|' +
              player.playsInline + ':' + (player.getAttribute('playsinline') === null) + '|' +
              player.disablePictureInPicture + ':' + (player.getAttribute('disablepictureinpicture') === null) + '|' +
              document.querySelectorAll('video > source').length + ':' +
              document.querySelectorAll('video > track').length + ':' +
              player.textContent.includes('fallback');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://media.local/base/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://media.local/video/primary.webm|false:true|false:true|false:true|false:true|false:true|false:true|2:1:true",
    )?;
    Ok(())
}

#[test]
fn video_has_no_implicit_role_and_supports_explicit_application_role() -> Result<()> {
    let html = r#"
        <video id='player' src='/media/clip.mp4'></video>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const player = document.getElementById('player');
            const initial = player.role + ':' + player.tagName;
            player.role = 'application';
            const assigned = player.role + ':' + player.getAttribute('role');
            player.removeAttribute('role');
            const restored = player.role + ':' + (player.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":VIDEO|application:application|:true")?;
    Ok(())
}
