use super::*;

#[test]
fn html_audio_element_global_and_audio_constructor_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const viaCtor = new Audio('/car_horn.wav');
            const viaCall = Audio('/bell.wav');
            const created = document.createElement('audio');

            document.getElementById('result').textContent = [
              typeof Audio,
              typeof HTMLAudioElement,
              window.Audio === Audio,
              window.HTMLAudioElement === HTMLAudioElement,
              viaCtor instanceof Audio,
              viaCtor instanceof HTMLAudioElement,
              viaCall instanceof HTMLAudioElement,
              created instanceof Audio,
              created instanceof HTMLAudioElement,
              viaCtor.tagName,
              viaCtor.src,
              viaCall.src,
              created.src
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/base/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "function|function|true|true|true|true|true|true|true|AUDIO|https://app.local/car_horn.wav|https://app.local/bell.wav|",
    )?;
    Ok(())
}

#[test]
fn audio_constructor_accepts_zero_or_one_argument() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const empty = new Audio();
            const withSrc = new Audio('/ok.mp3');

            document.getElementById('result').textContent = [
              empty.tagName,
              empty.src === '',
              withSrc.src
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://media.local/home", html)?;
    h.click("#run")?;
    h.assert_text("#result", "AUDIO|true|https://media.local/ok.mp3")?;
    Ok(())
}

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

#[test]
fn audio_reflective_own_property_surface_and_object_copy_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/media/theme.mp3' preload='metadata'></audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const beforeAssigned = Object.assign({}, player);
          const beforeSpread = { ...player };

          const before = [
            player.src,
            player.preload,
            String(Object.hasOwn(player, 'src')),
            String(Object.hasOwn(player, 'preload')),
            String(Object.getOwnPropertyDescriptor(player, 'src') === undefined),
            String(Object.getOwnPropertyNames(player).includes('preload')),
            String(Reflect.ownKeys(player).includes('src')),
            String('src' in beforeAssigned),
            String('preload' in beforeSpread)
          ].join(':');

          Object.defineProperty(player, 'src', {
            value: 'shadow-src',
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
            player.preload,
            String(Object.keys(player).sort().join(',') === 'extra,preload,src'),
            shadowAssigned.src,
            shadowAssigned.preload,
            shadowAssigned.extra,
            shadowSpread.src,
            shadowSpread.preload,
            shadowSpread.extra
          ].join(':');

          delete player.src;
          delete player.preload;

          const restoredAssigned = Object.assign({}, player);
          const restoredSpread = { ...player };

          const restored = [
            player.src,
            player.preload,
            String(Object.hasOwn(player, 'src')),
            String(Object.hasOwn(player, 'preload')),
            restoredAssigned.extra,
            String('src' in restoredAssigned),
            String('preload' in restoredAssigned),
            restoredSpread.extra,
            String('src' in restoredSpread),
            String('preload' in restoredSpread)
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
        "https://app.local/media/theme.mp3:metadata:false:false:true:false:false:false:false|shadow-src:shadow-preload:true:shadow-src:shadow-preload:expando:shadow-src:shadow-preload:expando|https://app.local/media/theme.mp3:metadata:false:false:expando:false:false:expando:false:false",
    )?;
    Ok(())
}

#[test]
fn audio_cross_origin_and_current_src_shadow_define_property_delete_and_fast_path_parity_work()
-> Result<()> {
    let html = r#"
        <audio id='player' crossorigin='use-credentials'>
          <source src='/audio/primary.ogg' type='audio/ogg'>
        </audio>
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

    let h = Harness::from_html_with_url("https://app.local/listen/index.html", html)?;
    h.assert_text(
        "#result",
        "use-credentials:https://app.local/audio/primary.ogg:use-credentials|set-cors:set-current:use-credentials:true|use-credentials:https://app.local/audio/primary.ogg:use-credentials:false:false",
    )?;
    Ok(())
}

#[test]
fn audio_controls_list_and_disable_remote_playback_shadow_define_property_delete_and_fast_path_parity_work()
-> Result<()> {
    let html = r#"
        <audio id='player' controlslist='nodownload' disableremoteplayback></audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');

          const before = [
            player.controlsList,
            String(player.disableRemotePlayback),
            player.getAttribute('controlslist'),
            player.getAttribute('disableremoteplayback')
          ].join(':');

          Object.defineProperty(player, 'controlsList', {
            value: 'shadow-controls',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'disableRemotePlayback', {
            value: 'shadow-disable',
            writable: true,
            enumerable: true,
            configurable: true
          });

          player.controlsList = 'set-controls';
          player.disableRemotePlayback = 'set-disable';

          const shadowed = [
            player.controlsList,
            String(player.disableRemotePlayback),
            player.getAttribute('controlslist'),
            player.getAttribute('disableremoteplayback'),
            String(Object.keys(player).sort().join(',') === 'controlsList,disableRemotePlayback')
          ].join(':');

          delete player.controlsList;
          delete player.disableRemotePlayback;

          const restored = [
            player.controlsList,
            String(player.disableRemotePlayback),
            String(Object.hasOwn(player, 'controlsList')),
            String(Object.hasOwn(player, 'disableRemotePlayback'))
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
        "nodownload:true:nodownload:true|set-controls:set-disable:nodownload:true:true|nodownload:true:false:false",
    )?;
    Ok(())
}

#[test]
fn audio_readonly_state_shadow_define_property_delete_and_restore_work() -> Result<()> {
    let html = r#"
        <audio id='player'></audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');

          const before = [
            String(player.paused),
            String(player.ended),
            String(player.seeking),
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          Object.defineProperty(player, 'paused', {
            value: 'shadow-paused',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'ended', {
            value: 'shadow-ended',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'seeking', {
            value: 'shadow-seeking',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'networkState', {
            value: 7,
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'readyState', {
            value: 9,
            writable: true,
            enumerable: true,
            configurable: true
          });

          player.paused = 'set-paused';
          player.ended = 'set-ended';
          player.seeking = 'set-seeking';
          player.networkState = 8;
          player.readyState = 10;

          const shadowed = [
            String(player.paused),
            String(player.ended),
            String(player.seeking),
            String(player.networkState),
            String(player.readyState),
            String(Object.keys(player).sort().join(',') === 'ended,networkState,paused,readyState,seeking')
          ].join(':');

          delete player.paused;
          delete player.ended;
          delete player.seeking;
          delete player.networkState;
          delete player.readyState;

          const restored = [
            String(player.paused),
            String(player.ended),
            String(player.seeking),
            String(player.networkState),
            String(player.readyState),
            String(Object.hasOwn(player, 'paused')),
            String(Object.hasOwn(player, 'networkState'))
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
        "true:false:false:0:0|set-paused:set-ended:set-seeking:8:10:true|true:false:false:0:0:false:false",
    )?;
    Ok(())
}
