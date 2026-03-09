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

#[test]
fn video_reflective_own_property_surface_and_object_copy_work() -> Result<()> {
    let html = r#"
        <video id='player' src='/media/clip.mp4' poster='/img/poster.jpg' preload='metadata'></video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const beforeAssigned = Object.assign({}, player);
          const beforeSpread = { ...player };

          const before = [
            player.src,
            player.poster,
            player.preload,
            String(Object.hasOwn(player, 'src')),
            String(Object.hasOwn(player, 'poster')),
            String(Object.hasOwn(player, 'preload')),
            String(Object.getOwnPropertyDescriptor(player, 'poster') === undefined),
            String(Object.getOwnPropertyNames(player).includes('src')),
            String(Reflect.ownKeys(player).includes('preload')),
            String('poster' in beforeAssigned),
            String('src' in beforeSpread)
          ].join(':');

          Object.defineProperty(player, 'src', {
            value: 'shadow-src',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'poster', {
            value: 'shadow-poster',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'preload', {
            value: 'shadow-preload',
            writable: true,
            enumerable: true,
            configurable: true
          });
          player.extra = 'expando';

          const shadowAssigned = Object.assign({}, player);
          const shadowSpread = { ...player };

          const shadowed = [
            player.src,
            player.poster,
            player.preload,
            String(Object.keys(player).sort().join(',') === 'extra,poster,preload,src'),
            shadowAssigned.src,
            shadowAssigned.poster,
            shadowAssigned.preload,
            shadowAssigned.extra,
            shadowSpread.src,
            shadowSpread.poster,
            shadowSpread.preload,
            shadowSpread.extra
          ].join(':');

          delete player.src;
          delete player.poster;
          delete player.preload;

          const restoredAssigned = Object.assign({}, player);
          const restoredSpread = { ...player };

          const restored = [
            player.src,
            player.poster,
            player.preload,
            String(Object.hasOwn(player, 'src')),
            String(Object.hasOwn(player, 'poster')),
            String(Object.hasOwn(player, 'preload')),
            restoredAssigned.extra,
            String('src' in restoredAssigned),
            String('poster' in restoredAssigned),
            String('preload' in restoredAssigned),
            restoredSpread.extra,
            String('src' in restoredSpread),
            String('poster' in restoredSpread),
            String('preload' in restoredSpread)
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
        "https://app.local/media/clip.mp4:https://app.local/img/poster.jpg:metadata:false:false:false:true:false:false:false:false|shadow-src:shadow-poster:shadow-preload:true:shadow-src:shadow-poster:shadow-preload:expando:shadow-src:shadow-poster:shadow-preload:expando|https://app.local/media/clip.mp4:https://app.local/img/poster.jpg:metadata:false:false:false:expando:false:false:false:expando:false:false:false",
    )?;
    Ok(())
}

#[test]
fn video_cross_origin_and_current_src_shadow_define_property_delete_and_fast_path_parity_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/media/clip.mp4' crossorigin='anonymous'></video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');

          const before = [
            player.crossOrigin,
            player.currentSrc,
            player.getAttribute('crossorigin')
          ].join(':');

          Object.defineProperty(player, 'crossOrigin', {
            value: 'shadow-cors',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'currentSrc', {
            value: 'shadow-current',
            writable: true,
            enumerable: true,
            configurable: true
          });

          player.crossOrigin = 'set-cors';
          player.currentSrc = 'set-current';

          const shadowed = [
            player.crossOrigin,
            player.currentSrc,
            player.getAttribute('crossorigin'),
            String(Object.keys(player).sort().join(',') === 'crossOrigin,currentSrc')
          ].join(':');

          delete player.crossOrigin;
          delete player.currentSrc;

          const restored = [
            player.crossOrigin,
            player.currentSrc,
            player.getAttribute('crossorigin'),
            String(Object.hasOwn(player, 'crossOrigin')),
            String(Object.hasOwn(player, 'currentSrc'))
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
        "anonymous:https://app.local/media/clip.mp4:anonymous|set-cors:set-current:anonymous:true|anonymous:https://app.local/media/clip.mp4:anonymous:false:false",
    )?;
    Ok(())
}

#[test]
fn video_picture_in_picture_and_inline_shadow_define_property_delete_and_fast_path_parity_work()
-> Result<()> {
    let html = r#"
        <video id='player' disablepictureinpicture playsinline></video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');

          const before = [
            String(player.disablePictureInPicture),
            String(player.playsInline),
            player.getAttribute('disablepictureinpicture'),
            player.getAttribute('playsinline')
          ].join(':');

          Object.defineProperty(player, 'disablePictureInPicture', {
            value: 'shadow-pip',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'playsInline', {
            value: 'shadow-inline',
            writable: true,
            enumerable: true,
            configurable: true
          });

          player.disablePictureInPicture = 'set-pip';
          player.playsInline = 'set-inline';

          const shadowed = [
            String(player.disablePictureInPicture),
            String(player.playsInline),
            player.getAttribute('disablepictureinpicture'),
            player.getAttribute('playsinline'),
            String(Object.keys(player).sort().join(',') === 'disablePictureInPicture,playsInline')
          ].join(':');

          delete player.disablePictureInPicture;
          delete player.playsInline;

          const restored = [
            String(player.disablePictureInPicture),
            String(player.playsInline),
            String(Object.hasOwn(player, 'disablePictureInPicture')),
            String(Object.hasOwn(player, 'playsInline'))
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
        "true:true:true:true|set-pip:set-inline:true:true:true|true:true:false:false",
    )?;
    Ok(())
}

#[test]
fn video_text_tracks_live_wrapper_identity_and_shadow_delete_parity_work() -> Result<()> {
    let html = r#"
        <video id='player'>
          <track id='captions-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const initial = player.textTracks;
          const again = player.textTracks;
          const beforeAssigned = Object.assign({}, player);
          const beforeSpread = { ...player };

          const before = [
            String(initial === again),
            String(initial.length),
            initial[0].id,
            String(Object.getOwnPropertyDescriptor(player, 'textTracks') === undefined),
            String('textTracks' in beforeAssigned),
            String('textTracks' in beforeSpread)
          ].join(':');

          Object.defineProperty(player, 'textTracks', {
            value: 'shadow-tracks',
            writable: true,
            enumerable: true,
            configurable: true
          });

          player.textTracks = 'set-tracks';

          const shadowAssigned = Object.assign({}, player);
          const shadowSpread = { ...player };

          const shadowed = [
            String(player.textTracks),
            shadowAssigned.textTracks,
            shadowSpread.textTracks,
            String(Object.keys(player).join(',') === 'textTracks')
          ].join(':');

          delete player.textTracks;

          const added = document.createElement('track');
          added.id = 'captions-ja';
          added.kind = 'subtitles';
          added.srclang = 'ja';
          added.src = '/tracks/ja.vtt';
          player.appendChild(added);

          const restoredAssigned = Object.assign({}, player);
          const restoredSpread = { ...player };

          const restored = [
            String(player.textTracks === initial),
            String(player.textTracks.length),
            player.textTracks[1].id,
            String(Object.hasOwn(player, 'textTracks')),
            String('textTracks' in restoredAssigned),
            String('textTracks' in restoredSpread)
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
        "true:1:captions-en:true:false:false|set-tracks:set-tracks:set-tracks:true|true:2:captions-ja:false:false:false",
    )?;
    Ok(())
}
