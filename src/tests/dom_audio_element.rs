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
fn audio_media_method_reflective_surface_and_stable_identity_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/media/theme.mp3'></audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const playDesc = Object.getOwnPropertyDescriptor(player, 'play');
          const pauseDesc = Object.getOwnPropertyDescriptor(player, 'pause');
          const loadDesc = Object.getOwnPropertyDescriptor(player, 'load');

          const before = [
            typeof player.play,
            String(player.play === player.play),
            String(playDesc.value === player.play),
            playDesc.value.name,
            String(playDesc.value.length),
            String(playDesc.enumerable === false),
            String(pauseDesc.value.name),
            String(loadDesc.value.name),
            String(Object.keys(player).includes('play')),
            String(Object.getOwnPropertyNames(player).includes('pause')),
            String(Reflect.ownKeys(player).includes('load')),
            String(Object.hasOwn(player, 'play'))
          ].join(':');

          Object.defineProperty(player, 'pause', {
            value: 'shadow-pause',
            enumerable: true,
            configurable: true
          });

          const shadowed = [
            String(player.pause),
            Object.keys(player).join(','),
            String(Object.getOwnPropertyDescriptor(player, 'pause').value === 'shadow-pause')
          ].join(':');

          delete player.pause;

          const restored = [
            typeof player.pause,
            String(player.pause === pauseDesc.value),
            String(Object.keys(player).includes('pause')),
            String(Object.getOwnPropertyDescriptor(player, 'pause').enumerable === false),
            String(Object.hasOwn(player, 'pause'))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/media/index.html", html)?;
    h.assert_text(
        "#result",
        "function:true:true:play:0:true:pause:load:false:true:true:true|shadow-pause:pause:true|function:true:false:true:true",
    )?;
    Ok(())
}

#[test]
fn audio_can_play_type_and_fast_seek_reflective_surface_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/media/theme.mp3'></audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const canPlayTypeDesc = Object.getOwnPropertyDescriptor(player, 'canPlayType');
          const fastSeekDesc = Object.getOwnPropertyDescriptor(player, 'fastSeek');
          const played = player.played;
          let receiverError = '';
          let argError = '';

          try {
            player.fastSeek.call({}, 1);
          } catch (error) {
            receiverError = String(error);
          }

          try {
            player.fastSeek.call(player);
          } catch (error) {
            argError = String(error);
          }

          const before = [
            typeof player.canPlayType,
            String(player.canPlayType === player.canPlayType),
            String(canPlayTypeDesc.value === player.canPlayType),
            canPlayTypeDesc.value.name,
            String(canPlayTypeDesc.value.length),
            String(fastSeekDesc.value === player.fastSeek),
            fastSeekDesc.value.name,
            String(fastSeekDesc.value.length),
            String(Object.getOwnPropertyNames(player).includes('canPlayType')),
            String(Reflect.ownKeys(player).includes('fastSeek')),
            player.canPlayType('audio/mpeg'),
            player.canPlayType('text/plain'),
            String(player.canPlayType() === '')
          ].join(':');

          player.fastSeek(2.75);

          const afterSeek = [
            String(player.currentTime),
            String(played === player.played),
            String(played.length),
            String(played.end(0))
          ].join(':');

          Object.defineProperty(player, 'canPlayType', {
            value: 'shadow-can-play',
            enumerable: true,
            configurable: true
          });

          const shadowed = [
            String(player.canPlayType),
            Object.keys(player).join(','),
            String(Object.getOwnPropertyDescriptor(player, 'canPlayType').value === 'shadow-can-play')
          ].join(':');

          delete player.canPlayType;

          const restored = [
            typeof player.canPlayType,
            String(player.canPlayType === canPlayTypeDesc.value),
            String(Object.keys(player).includes('canPlayType')),
            String(Object.getOwnPropertyDescriptor(player, 'canPlayType').enumerable === false),
            String(receiverError.includes('HTMLMediaElement method called on incompatible receiver')),
            String(argError.includes('fastSeek requires an argument'))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterSeek,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/media/index.html", html)?;
    h.assert_text(
        "#result",
        "function:true:true:canPlayType:1:true:fastSeek:1:true:true:maybe::true|2.75:true:1:2.75|shadow-can-play:canPlayType:true|function:true:false:true:true:true",
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

#[test]
fn audio_source_child_attribute_mutation_updates_current_src_and_network_state_work() -> Result<()>
{
    let html = r#"
        <audio id='player'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
          <source id='backup' src='/audio/backup.mp3' type='audio/mpeg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          primary.setAttribute('src', '/audio/updated.ogg');

          const updated = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          primary.removeAttribute('src');
          primary.setAttribute('srcset', '/audio/from-srcset.ogg 1x');

          const srcsetFallback = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.insertBefore(backup, primary);

          const reordered = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          backup.removeAttribute('src');
          primary.removeAttribute('srcset');

          const emptied = [
            String(player.currentSrc === ''),
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            updated,
            srcsetFallback,
            reordered,
            emptied
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/listen/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/primary.ogg:1:0|https://app.local/audio/updated.ogg:1:0|https://app.local/audio/from-srcset.ogg:1:0|https://app.local/audio/backup.mp3:1:0|true:0:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_attribute_mutation_respects_type_media_and_srcset_work() -> Result<()> {
    let html = r#"
        <audio id='player'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg' media='all'>
          <source id='backup' src='/audio/backup.mp3' type='audio/mpeg' media='all'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          primary.type = 'text/plain';

          const afterUnsupportedType = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          primary.type = 'audio/ogg';
          primary.media = 'not all';

          const afterMediaSkip = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          primary.removeAttribute('src');
          primary.setAttribute('srcset', '/audio/from-srcset.ogg 1x');
          primary.media = 'all';

          const afterSrcsetFallback = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          primary.media = 'not all';
          backup.type = 'application/json';

          const emptied = [
            String(player.currentSrc === ''),
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterUnsupportedType,
            afterMediaSkip,
            afterSrcsetFallback,
            emptied
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/listen/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/primary.ogg:1:0|https://app.local/audio/backup.mp3:1:0|https://app.local/audio/backup.mp3:1:0|https://app.local/audio/from-srcset.ogg:1:0|true:0:0",
    )?;
    Ok(())
}

#[test]
fn audio_direct_src_precedence_flips_cleanly_against_nested_sources_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
          <source id='backup' src='/audio/backup.mp3' type='audio/mpeg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          primary.src = '/audio/updated.ogg';
          backup.removeAttribute('src');
          backup.setAttribute('srcset', '/audio/fallback-from-srcset.mp3 1x');

          const whileDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');

          const afterRemovingDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '/audio/direct-2.mp3';

          const afterRestoringDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');
          primary.removeAttribute('src');

          const afterFallingToBackup = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            whileDirect,
            afterRemovingDirect,
            afterRestoringDirect,
            afterFallingToBackup
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/listen/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/direct.mp3:1:0|https://app.local/audio/updated.ogg:1:0|https://app.local/audio/direct-2.mp3:1:0|https://app.local/audio/fallback-from-srcset.mp3:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_media_state_shadow_delete_parity_survives_source_precedence_churn_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
          <source id='backup' src='/audio/backup.mp3' type='audio/mpeg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');
          let currentValue = 'shadow-current';

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          Object.defineProperty(player, 'currentSrc', {
            get() {
              return currentValue;
            },
            set(value) {
              currentValue = 'set:' + value;
            },
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

          const reflected = [
            String(Reflect.set(player, 'currentSrc', 'reflect-current')),
            String(Reflect.set(player, 'networkState', 8)),
            String(Reflect.set(player, 'readyState', 10))
          ].join(':');

          primary.src = '/audio/nested-updated.ogg';
          player.removeAttribute('src');
          backup.removeAttribute('src');
          backup.setAttribute('srcset', '/audio/fallback-from-srcset.mp3 1x');

          const shadowed = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState),
            String(Object.keys(player).sort().join(',') === 'currentSrc,networkState,readyState')
          ].join(':');

          delete player.currentSrc;
          delete player.networkState;
          delete player.readyState;

          const restoredNested = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState),
            String(Object.hasOwn(player, 'currentSrc')),
            String(Object.hasOwn(player, 'networkState')),
            String(Object.hasOwn(player, 'readyState'))
          ].join(':');

          player.src = '/audio/direct-2.mp3';

          const restoredDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            reflected,
            shadowed,
            restoredNested,
            restoredDirect
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/listen/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|true:true:true|set:reflect-current:8:10:true|https://app.local/audio/nested-updated.ogg:1:0:false:false:false|https://app.local/audio/direct-2.mp3:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_selection_fallback_matrix_across_mixed_candidates_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='first' src='/audio/first.ogg' type='text/plain' media='all'>
          <source id='second' srcset='/audio/from-srcset.ogg 1x' type='audio/ogg' media='all'>
          <source id='third' src='/audio/from-third.mp3' type='audio/mpeg' media='not all'>
          <source id='fourth' src='/audio/from-fourth.wav' type='audio/wav' media='all'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const first = document.getElementById('first');
          const second = document.getElementById('second');
          const third = document.getElementById('third');
          const fourth = document.getElementById('fourth');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');

          const afterRemovingDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          second.type = 'application/json';

          const afterSecondInvalid = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          first.type = 'audio/ogg';
          first.src = '/audio/promoted-first.ogg';

          const afterFirstPromoted = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          first.removeAttribute('src');
          first.setAttribute('srcset', '/audio/promoted-first-srcset.ogg 1x');

          const afterFirstSrcset = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          first.media = 'not all';
          second.type = 'audio/ogg';
          second.removeAttribute('srcset');
          second.setAttribute('src', '/audio/restored-second.ogg');

          const afterSecondRestored = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          second.type = 'text/plain';
          third.media = 'all';

          const afterThirdEnabled = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          third.removeAttribute('src');
          fourth.media = 'not all';

          const emptied = [
            String(player.currentSrc === ''),
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterSecondInvalid,
            afterFirstPromoted,
            afterFirstSrcset,
            afterSecondRestored,
            afterThirdEnabled,
            emptied
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/listen/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/from-srcset.ogg:1:0|https://app.local/audio/from-fourth.wav:1:0|https://app.local/audio/promoted-first.ogg:1:0|https://app.local/audio/promoted-first-srcset.ogg:1:0|https://app.local/audio/restored-second.ogg:1:0|https://app.local/audio/from-third.mp3:1:0|true:0:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_normalization_edge_cases_keep_current_src_aligned_work() -> Result<()> {
    let html = r#"
        <audio id='player' src=' ./audio/direct.mp3 '>
          <source id='first' src='   ' type='audio/ogg'>
          <source id='second' srcset=' ./audio/from-srcset.ogg 1x, ./audio/ignored.ogg 2x ' type='audio/ogg'>
          <source id='third' src='./audio/third.mp3' type='audio/mpeg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const first = document.getElementById('first');
          const second = document.getElementById('second');
          const third = document.getElementById('third');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '   ';

          const whitespaceDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '';

          const emptyDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');

          const srcsetFallback = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          second.srcset = '   ';
          first.src = ' ./audio/first-restored.ogg ';

          const restoredFirst = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          first.src = '   ';
          third.src = ' ./audio/third-relative.mp3 ';

          const restoredThird = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          third.removeAttribute('src');

          const emptied = [
            String(player.currentSrc === ''),
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            whitespaceDirect,
            emptyDirect,
            srcsetFallback,
            restoredFirst,
            restoredThird,
            emptied
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/watch/audio/direct.mp3:1:0|https://app.local/watch/index.html:1:0|https://app.local/watch/index.html:1:0|https://app.local/watch/audio/from-srcset.ogg:1:0|https://app.local/watch/audio/first-restored.ogg:1:0|https://app.local/watch/audio/third-relative.mp3:1:0|true:0:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_reorder_disconnect_and_replace_matrix_keeps_current_src_aligned_work() -> Result<()>
{
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
          <source id='backup' src='/audio/backup.mp3' type='audio/mpeg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');

          const afterRemovingDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.appendChild(primary);

          const afterReorder = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          const detachedBackup = player.removeChild(backup);

          const afterDisconnect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          detachedBackup.src = '/audio/backup-return.mp3';
          player.insertBefore(detachedBackup, primary);

          const afterReinsert = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          const replacement = document.createElement('source');
          replacement.id = 'replacement';
          replacement.type = 'audio/wav';
          replacement.srcset = '/audio/replacement.wav 1x';
          player.replaceChild(replacement, detachedBackup);

          const afterReplace = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '/audio/direct-2.mp3';

          const restoredDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterReorder,
            afterDisconnect,
            afterReinsert,
            afterReplace,
            restoredDirect
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/listen/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/primary.ogg:1:0|https://app.local/audio/backup.mp3:1:0|https://app.local/audio/primary.ogg:1:0|https://app.local/audio/backup-return.mp3:1:0|https://app.local/audio/replacement.wav:1:0|https://app.local/audio/direct-2.mp3:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_clone_and_fragment_insertion_matrix_keeps_current_src_aligned_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='base' src='/audio/base.ogg' type='audio/ogg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const base = document.getElementById('base');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');

          const afterRemovingDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          const fragment = document.createDocumentFragment();
          const invalidClone = base.cloneNode();
          invalidClone.id = 'invalid';
          invalidClone.type = 'text/plain';
          invalidClone.src = '/audio/invalid.txt';
          const preferredClone = base.cloneNode();
          preferredClone.id = 'preferred';
          preferredClone.type = 'audio/mpeg';
          preferredClone.src = '/audio/clone-preferred.mp3';
          fragment.appendChild(invalidClone);
          fragment.appendChild(preferredClone);
          player.insertBefore(fragment, base);

          const afterFragmentInsert = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          const detachedPreferred = player.removeChild(preferredClone);

          const afterDetachPreferred = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          detachedPreferred.src = '/audio/reused-return.wav';
          const reinsertFragment = document.createDocumentFragment();
          const replacementClone = base.cloneNode();
          replacementClone.id = 'replacement';
          replacementClone.type = 'audio/mp4';
          replacementClone.removeAttribute('src');
          replacementClone.srcset = '/audio/replacement-from-srcset.m4a 1x';
          reinsertFragment.appendChild(detachedPreferred);
          reinsertFragment.appendChild(replacementClone);
          player.insertBefore(reinsertFragment, base);

          const afterReinsertBatch = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeChild(detachedPreferred);

          const afterReplacementWins = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '/audio/direct-2.mp3';

          const restoredDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterFragmentInsert,
            afterDetachPreferred,
            afterReinsertBatch,
            afterReplacementWins,
            restoredDirect
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/base.ogg:1:0|https://app.local/audio/clone-preferred.mp3:1:0|https://app.local/audio/base.ogg:1:0|https://app.local/audio/reused-return.wav:1:0|https://app.local/audio/replacement-from-srcset.m4a:1:0|https://app.local/audio/direct-2.mp3:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_adoption_and_cross_parent_reparent_matrix_keeps_current_src_aligned_work()
-> Result<()> {
    let html = r#"
        <audio id='left' src='/audio/left-direct.mp3'>
          <source id='shared' src='/audio/shared.mp3' type='audio/mpeg'>
          <source id='left-base' src='/audio/left-base.ogg' type='audio/ogg'>
        </audio>
        <audio id='right'>
          <source id='right-base' src='/audio/right-base.wav' type='audio/wav'>
        </audio>
        <div id='stash'></div>
        <p id='result'></p>
        <script>
          const left = document.getElementById('left');
          const right = document.getElementById('right');
          const shared = document.getElementById('shared');
          const leftBase = document.getElementById('left-base');
          const rightBase = document.getElementById('right-base');
          const stash = document.getElementById('stash');

          const before = [
            left.currentSrc,
            String(left.networkState),
            String(left.readyState),
            right.currentSrc,
            String(right.networkState),
            String(right.readyState)
          ].join(':');

          left.removeAttribute('src');

          const afterRemovingDirect = [
            left.currentSrc,
            String(left.networkState),
            String(left.readyState),
            right.currentSrc,
            String(right.networkState),
            String(right.readyState)
          ].join(':');

          right.insertBefore(shared, rightBase);

          const afterMoveToRight = [
            left.currentSrc,
            String(left.networkState),
            String(left.readyState),
            right.currentSrc,
            String(right.networkState),
            String(right.readyState)
          ].join(':');

          stash.appendChild(shared);

          const afterStash = [
            left.currentSrc,
            String(left.networkState),
            String(left.readyState),
            right.currentSrc,
            String(right.networkState),
            String(right.readyState)
          ].join(':');

          shared.src = '/audio/shared-return.wav';
          left.insertBefore(shared, leftBase);

          const afterReturnLeft = [
            left.currentSrc,
            String(left.networkState),
            String(left.readyState),
            right.currentSrc,
            String(right.networkState),
            String(right.readyState)
          ].join(':');

          left.src = '/audio/left-direct-2.mp3';

          const restoredDirect = [
            left.currentSrc,
            String(left.networkState),
            String(left.readyState),
            right.currentSrc,
            String(right.networkState),
            String(right.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterMoveToRight,
            afterStash,
            afterReturnLeft,
            restoredDirect
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/left-direct.mp3:1:0:https://app.local/audio/right-base.wav:1:0|https://app.local/audio/shared.mp3:1:0:https://app.local/audio/right-base.wav:1:0|https://app.local/audio/left-base.ogg:1:0:https://app.local/audio/shared.mp3:1:0|https://app.local/audio/left-base.ogg:1:0:https://app.local/audio/right-base.wav:1:0|https://app.local/audio/shared-return.wav:1:0:https://app.local/audio/right-base.wav:1:0|https://app.local/audio/left-direct-2.mp3:1:0:https://app.local/audio/right-base.wav:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_sibling_and_filter_mutation_matrix_keeps_current_src_aligned_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='bad-type' src='/audio/bad.txt' type='text/plain'>
          <source id='filtered' src='/audio/filtered.mp3' type='audio/mpeg' media='not all'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const badType = document.getElementById('bad-type');
          const filtered = document.getElementById('filtered');
          const primary = document.getElementById('primary');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');

          const afterRemovingDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          const spacer = document.createElement('span');
          spacer.id = 'spacer';
          player.insertBefore(spacer, primary);

          const afterSiblingInsert = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          badType.type = 'audio/mpeg';

          const afterTypeToggle = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          badType.src = '   ';
          filtered.removeAttribute('media');

          const afterMediaToggle = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          filtered.src = '   ';
          primary.src = '   ';

          const afterEmptyingCandidates = [
            String(player.currentSrc === ''),
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '/audio/direct-2.mp3';

          const restoredDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterSiblingInsert,
            afterTypeToggle,
            afterMediaToggle,
            afterEmptyingCandidates,
            restoredDirect
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/primary.ogg:1:0|https://app.local/audio/primary.ogg:1:0|https://app.local/audio/bad.txt:1:0|https://app.local/audio/filtered.mp3:1:0|true:0:0|https://app.local/audio/direct-2.mp3:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_batch_reset_matrix_keeps_current_src_aligned_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');

          const afterRemovingDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          const firstBatch = document.createDocumentFragment();
          const invalid = document.createElement('source');
          invalid.type = 'text/plain';
          invalid.src = '/audio/invalid.txt';
          const preferred = document.createElement('source');
          preferred.type = 'audio/mpeg';
          preferred.src = '/audio/first-reset.mp3';
          firstBatch.appendChild(invalid);
          firstBatch.appendChild(preferred);
          player.replaceChildren(firstBatch);

          const afterFirstReset = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.replaceChildren();

          const afterClearingAll = [
            String(player.currentSrc === ''),
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          const secondBatch = document.createDocumentFragment();
          const filtered = document.createElement('source');
          filtered.type = 'audio/mpeg';
          filtered.media = 'not all';
          filtered.src = '/audio/filtered.mp3';
          const empty = document.createElement('source');
          empty.type = 'audio/ogg';
          empty.src = '   ';
          const fallback = document.createElement('source');
          fallback.type = 'audio/wav';
          fallback.srcset = '/audio/second-reset.wav 1x';
          secondBatch.appendChild(filtered);
          secondBatch.appendChild(empty);
          secondBatch.appendChild(fallback);
          player.replaceChildren(secondBatch);

          const afterSecondReset = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '/audio/direct-2.mp3';

          const restoredDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterFirstReset,
            afterClearingAll,
            afterSecondReset,
            restoredDirect
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/primary.ogg:1:0|https://app.local/audio/first-reset.mp3:1:0|true:0:0|https://app.local/audio/second-reset.wav:1:0|https://app.local/audio/direct-2.mp3:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_string_rebuild_matrix_keeps_current_src_aligned_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');

          const afterRemovingDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.innerHTML = '<source id="bad" src="/audio/bad.txt" type="text/plain"><source id="rebuilt" src="/audio/rebuilt.mp3" type="audio/mpeg">';

          const afterInnerHtml = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.insertAdjacentHTML('afterbegin', '<source id="leading" src="/audio/leading.wav" type="audio/wav">');

          const afterInsertAdjacent = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('leading').removeAttribute('src');
          document.getElementById('bad').type = 'audio/mpeg';

          const afterMixedToggle = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.innerHTML = '<source id="filtered" src="/audio/filtered.mp3" type="audio/mpeg" media="not all"><source id="fallback" srcset="/audio/from-srcset.ogg 1x" type="audio/ogg">';

          const afterSecondInnerHtml = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '/audio/direct-2.mp3';

          const restoredDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterInnerHtml,
            afterInsertAdjacent,
            afterMixedToggle,
            afterSecondInnerHtml,
            restoredDirect
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/primary.ogg:1:0|https://app.local/audio/rebuilt.mp3:1:0|https://app.local/audio/leading.wav:1:0|https://app.local/audio/bad.txt:1:0|https://app.local/audio/from-srcset.ogg:1:0|https://app.local/audio/direct-2.mp3:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_source_mixed_rebuild_churn_keeps_current_src_aligned_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');

          const before = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');

          const afterRemovingDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.innerHTML = '<source id="bad" src="/audio/bad.txt" type="text/plain"><source id="rebuilt" src="/audio/rebuilt.mp3" type="audio/mpeg">';

          const afterInnerHtml = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          const domInserted = document.createElement('source');
          domInserted.id = 'dom-inserted';
          domInserted.type = 'audio/wav';
          domInserted.src = '/audio/dom-inserted.wav';
          player.insertBefore(domInserted, player.firstChild);

          const afterDomInsert = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeChild(domInserted);
          player.insertAdjacentHTML('afterbegin', '<source id="string-leading" src="/audio/string-leading.aac" type="audio/aac">');

          const afterStringInsert = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('string-leading').removeAttribute('src');
          document.getElementById('bad').type = 'audio/mpeg';

          const afterToggle = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '/audio/direct-2.mp3';

          const restoredDirect = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterInnerHtml,
            afterDomInsert,
            afterStringInsert,
            afterToggle,
            restoredDirect
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/primary.ogg:1:0|https://app.local/audio/rebuilt.mp3:1:0|https://app.local/audio/dom-inserted.wav:1:0|https://app.local/audio/string-leading.aac:1:0|https://app.local/audio/bad.txt:1:0|https://app.local/audio/direct-2.mp3:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_direct_src_reset_and_load_churn_keeps_current_src_aligned_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');

          function snapshot() {
            return [
              player.currentSrc,
              String(player.networkState),
              String(player.readyState)
            ].join(':');
          }

          const before = snapshot();

          player.removeAttribute('src');
          player.load();

          const afterRemovingDirect = snapshot();

          player.src = '/audio/direct-2.mp3';
          player.innerHTML = '<source id="bad" src="/audio/bad.txt" type="text/plain"><source id="rebuilt" src="/audio/rebuilt.mp3" type="audio/mpeg">';
          player.load();

          const afterResetWhileDirect = snapshot();

          player.removeAttribute('src');
          player.load();

          const afterNestedRestore = snapshot();

          player.src = '/audio/direct-3.mp3';
          player.load();

          const afterDirectAgain = snapshot();

          player.removeAttribute('src');
          document.getElementById('rebuilt').removeAttribute('src');
          player.insertAdjacentHTML('beforeend', '<source id="fallback" src="/audio/fallback.wav" type="audio/wav">');
          player.load();

          const afterFallback = snapshot();

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterResetWhileDirect,
            afterNestedRestore,
            afterDirectAgain,
            afterFallback
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/primary.ogg:1:0|https://app.local/audio/direct-2.mp3:1:0|https://app.local/audio/rebuilt.mp3:1:0|https://app.local/audio/direct-3.mp3:1:0|https://app.local/audio/fallback.wav:1:0",
    )?;
    Ok(())
}

#[test]
fn audio_load_triggered_direct_nested_precedence_matrix_stays_aligned_work() -> Result<()> {
    let html = r#"
        <audio id='player' src='/audio/direct.mp3'>
          <source id='bad' src='/audio/bad.txt' type='text/plain'>
          <source id='primary' src='/audio/primary.ogg' type='audio/ogg'>
          <source id='fallback' src='/audio/fallback.wav' type='audio/wav' media='not all'>
        </audio>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const bad = document.getElementById('bad');
          const primary = document.getElementById('primary');
          const fallback = document.getElementById('fallback');

          function snapshot() {
            return [
              player.currentSrc,
              String(player.networkState),
              String(player.readyState)
            ].join(':');
          }

          player.load();
          const before = snapshot();

          player.removeAttribute('src');
          player.load();
          const afterRemovingDirect = snapshot();

          primary.type = 'text/plain';
          fallback.media = 'all';
          player.load();
          const afterFilteredNestedFlip = snapshot();

          player.src = '/audio/direct-2.mp3';
          player.load();
          const afterRestoringDirect = snapshot();

          player.innerHTML = '<source id="rebuilt-bad" src="/audio/rebuilt-bad.txt" type="text/plain"><source id="rebuilt" src="/audio/rebuilt.mp3" type="audio/mpeg"><source id="rebuilt-filtered" src="/audio/rebuilt-filtered.wav" type="audio/wav" media="not all">';
          player.load();
          const afterNestedResetWhileDirect = snapshot();

          player.removeAttribute('src');
          player.load();
          const afterNestedRestore = snapshot();

          document.getElementById('rebuilt').removeAttribute('src');
          document.getElementById('rebuilt-bad').type = 'audio/mpeg';
          player.load();
          const afterReweightingNested = snapshot();

          bad.type = 'audio/mpeg';
          player.appendChild(bad);
          player.load();
          const afterReinsertingOriginal = snapshot();

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterFilteredNestedFlip,
            afterRestoringDirect,
            afterNestedResetWhileDirect,
            afterNestedRestore,
            afterReweightingNested,
            afterReinsertingOriginal
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/audio/direct.mp3:1:0|https://app.local/audio/primary.ogg:1:0|https://app.local/audio/fallback.wav:1:0|https://app.local/audio/direct-2.mp3:1:0|https://app.local/audio/direct-2.mp3:1:0|https://app.local/audio/rebuilt.mp3:1:0|https://app.local/audio/rebuilt-bad.txt:1:0|https://app.local/audio/rebuilt-bad.txt:1:0",
    )?;
    Ok(())
}
