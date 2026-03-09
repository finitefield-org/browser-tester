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

#[test]
fn video_text_track_list_branding_and_media_playback_state_shadow_parity_work() -> Result<()> {
    let html = r#"
        <video id='player'>
          <track id='captions-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const tracks = player.textTracks;

          const before = [
            String(tracks.constructor === TextTrackList),
            Object.prototype.toString.call(tracks),
            String(player.defaultMuted),
            String(player.currentTime),
            String(player.volume),
            String(Number.isNaN(player.duration))
          ].join(':');

          player.defaultMuted = true;
          player.currentTime = 12.5;
          player.volume = 0.25;
          player.duration = 99;

          const afterSet = [
            String(player.defaultMuted),
            String(player.muted),
            String(player.getAttribute('muted')),
            String(player.currentTime),
            String(player.volume),
            String(Number.isNaN(player.duration)),
            String(Object.hasOwn(player, 'duration'))
          ].join(':');

          Object.defineProperty(player, 'defaultMuted', {
            value: 'shadow-muted',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'currentTime', {
            value: 'shadow-time',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'volume', {
            value: 'shadow-volume',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'duration', {
            value: 'shadow-duration',
            writable: true,
            enumerable: true,
            configurable: true
          });

          const shadowed = [
            String(player.defaultMuted),
            String(player.currentTime),
            String(player.volume),
            String(player.duration),
            Object.keys(player).sort().join(',')
          ].join(':');

          delete player.defaultMuted;
          delete player.currentTime;
          delete player.volume;
          delete player.duration;

          const restored = [
            String(player.defaultMuted),
            String(player.muted),
            String(player.currentTime),
            String(player.volume),
            String(Number.isNaN(player.duration)),
            String(Object.hasOwn(player, 'defaultMuted')),
            String(Object.hasOwn(player, 'currentTime')),
            String(Object.hasOwn(player, 'volume')),
            String(Object.hasOwn(player, 'duration'))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterSet,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "true:[object TextTrackList]:false:0:1:true|true:true:true:12.5:0.25:true:false|shadow-muted:shadow-time:shadow-volume:shadow-duration:currentTime,defaultMuted,duration,volume|true:true:12.5:0.25:true:false:false:false:false",
    )?;
    Ok(())
}

#[test]
fn video_text_track_list_reflective_surface_and_object_copy_work() -> Result<()> {
    let html = r#"
        <video id='player'>
          <track id='captions-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
          <track id='captions-ja' kind='subtitles' srclang='ja' src='/tracks/ja.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const tracks = player.textTracks;
          const sameTracks = player.textTracks;
          const zeroDesc = Object.getOwnPropertyDescriptor(tracks, '0');
          const lengthDesc = Object.getOwnPropertyDescriptor(tracks, 'length');
          const assigned = Object.assign({}, tracks);
          const spread = { ...tracks };

          const before = [
            String(sameTracks === tracks),
            Object.keys(tracks).join(','),
            Object.getOwnPropertyNames(tracks).join(','),
            Reflect.ownKeys(tracks).join(','),
            String(Object.hasOwn(tracks, '1')),
            zeroDesc.value.id,
            lengthDesc.value,
            assigned['0'].id,
            assigned['1'].id,
            Object.keys(assigned).join(','),
            spread['0'].id,
            spread['1'].id,
            Object.keys(spread).join(',')
          ].join(':');

          Object.defineProperty(tracks, 'marker', {
            value: 'own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(tracks, '0', {
            value: 'shadow-track',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(tracks, 'length', {
            get() {
              return 99;
            },
            configurable: true
          });

          const shadowed = [
            tracks.marker,
            String(tracks[0]),
            String(tracks.length),
            Object.keys(tracks).join(',')
          ].join(':');

          delete sameTracks.marker;
          delete sameTracks[0];
          delete sameTracks.length;

          const restored = [
            sameTracks[0].id,
            String(sameTracks.length),
            Object.keys(sameTracks).join(','),
            String(Object.hasOwn(sameTracks, 'marker')),
            String(Object.hasOwn(sameTracks, '0')),
            String(Object.hasOwn(sameTracks, 'length'))
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
        "true:0,1:0,1,length:0,1,length:true:captions-en:2:captions-en:captions-ja:0,1:captions-en:captions-ja:0,1|own:shadow-track:99:0,1,marker|captions-en:2:0,1:false:true:true",
    )?;
    Ok(())
}

#[test]
fn video_playback_rate_properties_shadow_define_property_delete_and_restore_work() -> Result<()> {
    let html = r#"
        <video id='player'></video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');

          const before = [
            String(player.playbackRate),
            String(player.defaultPlaybackRate),
            String(Object.getOwnPropertyDescriptor(player, 'playbackRate') === undefined),
            String(Object.getOwnPropertyDescriptor(player, 'defaultPlaybackRate') === undefined)
          ].join(':');

          player.playbackRate = 1.5;
          player.defaultPlaybackRate = 0.75;

          const afterSet = [
            String(player.playbackRate),
            String(player.defaultPlaybackRate),
            String(Object.hasOwn(player, 'playbackRate')),
            String(Object.hasOwn(player, 'defaultPlaybackRate'))
          ].join(':');

          Object.defineProperty(player, 'playbackRate', {
            value: 'shadow-rate',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'defaultPlaybackRate', {
            value: 'shadow-default-rate',
            writable: true,
            enumerable: true,
            configurable: true
          });

          player.playbackRate = 'set-rate';
          player.defaultPlaybackRate = 'set-default-rate';

          const shadowed = [
            String(player.playbackRate),
            String(player.defaultPlaybackRate),
            Object.keys(player).sort().join(',')
          ].join(':');

          delete player.playbackRate;
          delete player.defaultPlaybackRate;

          const restored = [
            String(player.playbackRate),
            String(player.defaultPlaybackRate),
            String(Object.hasOwn(player, 'playbackRate')),
            String(Object.hasOwn(player, 'defaultPlaybackRate'))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterSet,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "1:1:true:true|1.5:0.75:false:false|set-rate:set-default-rate:defaultPlaybackRate,playbackRate|1.5:0.75:false:false",
    )?;
    Ok(())
}

#[test]
fn video_time_ranges_branding_and_media_shadow_delete_parity_work() -> Result<()> {
    let html = r#"
        <video id='player' src='/movie.mp4'></video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 12.5;
          const buffered = player.buffered;
          const seekable = player.seekable;
          const played = player.played;

          const before = [
            String(buffered === player.buffered),
            String(seekable === player.seekable),
            String(played === player.played),
            String(Object.getPrototypeOf(buffered) === TimeRanges.prototype),
            String(Object.getPrototypeOf(seekable) === TimeRanges.prototype),
            String(Object.getPrototypeOf(played) === TimeRanges.prototype),
            Object.prototype.toString.call(buffered),
            String(buffered.length),
            String(buffered.start(0)),
            String(buffered.end(0)),
            String(seekable.length),
            String(seekable.end(0)),
            String(played.length),
            String(played.end(0))
          ].join(':');

          Object.defineProperty(player, 'buffered', {
            value: 'shadow-buffered',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(player, 'played', {
            get() {
              return 'shadow-played';
            },
            enumerable: true,
            configurable: true
          });

          const shadowed = [
            String(player.buffered),
            String(player.seekable === seekable),
            String(player.played),
            Object.keys(player).sort().join(',')
          ].join(':');

          delete player.buffered;
          delete player.played;
          player.currentTime = 7;

          const restored = [
            String(player.buffered === buffered),
            String(player.played === played),
            String(player.played.length),
            String(player.played.end(0)),
            String(Object.hasOwn(player, 'buffered')),
            String(Object.hasOwn(player, 'played'))
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
        "true:true:true:true:true:true:[object TimeRanges]:1:0:12.5:1:12.5:1:12.5|shadow-buffered:true:shadow-played:buffered,played|true:true:1:7:false:false",
    )?;
    Ok(())
}

#[test]
fn video_time_ranges_reflective_surface_and_object_copy_work() -> Result<()> {
    let html = r#"
        <video id='player' src='/movie.mp4'></video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 3;
          const buffered = player.buffered;
          const sameBuffered = player.buffered;
          const assigned = Object.assign({}, buffered);
          const spread = { ...buffered };

          const before = [
            String(sameBuffered === buffered),
            String(Object.keys(buffered).length),
            String(Object.getOwnPropertyNames(buffered).length),
            String(Reflect.ownKeys(buffered).length),
            String(Object.getOwnPropertyDescriptor(buffered, 'length') === undefined),
            String('length' in buffered),
            String(Object.hasOwn(buffered, 'length')),
            String(Object.keys(assigned).length),
            String(Object.keys(spread).length)
          ].join(':');

          Object.defineProperty(buffered, 'length', {
            value: 99,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(buffered, 'marker', {
            value: 'own',
            enumerable: true,
            configurable: true
          });

          const ownAssigned = Object.assign({}, buffered);
          const ownSpread = { ...buffered };

          const shadowed = [
            String(buffered.length),
            buffered.marker,
            Object.keys(buffered).sort().join(','),
            String(ownAssigned.length),
            ownAssigned.marker,
            Object.keys(ownAssigned).sort().join(','),
            String(ownSpread.length),
            ownSpread.marker,
            Object.keys(ownSpread).sort().join(',')
          ].join(':');

          delete sameBuffered.length;
          delete sameBuffered.marker;

          const restored = [
            String(sameBuffered.length),
            Object.keys(sameBuffered).join(','),
            String(Object.hasOwn(sameBuffered, 'length')),
            String(Object.hasOwn(sameBuffered, 'marker')),
            String(sameBuffered.start(0)),
            String(sameBuffered.end(0))
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
        "true:0:0:0:true:true:false:0:0|99:own:length,marker:99:own:length,marker:99:own:length,marker|1::false:false:0:3",
    )?;
    Ok(())
}

#[test]
fn video_time_ranges_live_wrappers_survive_src_mutation_and_extracted_calls_work() -> Result<()> {
    let html = r#"
        <video id='player' src='/movie.mp4'></video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 4;
          const buffered = player.buffered;
          const seekable = player.seekable;
          const played = player.played;
          const proto = TimeRanges.prototype;
          const lengthGetter = Object.getOwnPropertyDescriptor(proto, 'length').get;
          const end = proto.end;
          let emptyError = '';

          const initial = [
            String(buffered === player.buffered),
            String(seekable === player.seekable),
            String(played === player.played),
            String(lengthGetter.call(buffered)),
            String(end.call(played, 0))
          ].join(':');

          player.removeAttribute('src');

          try {
            end.call(buffered, 0);
          } catch (error) {
            emptyError = String(error);
          }

          const emptied = [
            String(player.buffered === buffered),
            String(player.seekable === seekable),
            String(player.played === played),
            String(lengthGetter.call(buffered)),
            String(lengthGetter.call(seekable)),
            String(lengthGetter.call(played)),
            String(emptyError.includes('TimeRanges.end index out of range'))
          ].join(':');

          player.setAttribute('src', '/movie-2.mp4');
          player.currentTime = 9;

          const restored = [
            String(player.buffered === buffered),
            String(player.seekable === seekable),
            String(player.played === played),
            String(lengthGetter.call(buffered)),
            String(end.call(buffered, 0)),
            String(end.call(seekable, 0)),
            String(end.call(played, 0))
          ].join(':');

          document.getElementById('result').textContent = [
            initial,
            emptied,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "true:true:true:1:4|true:true:true:0:0:0:true|true:true:true:1:9:9:9",
    )?;
    Ok(())
}

#[test]
fn video_time_ranges_and_text_tracks_cached_wrappers_update_together_work() -> Result<()> {
    let html = r#"
        <video id='player' src='/movie.mp4'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 2;
          const ranges = player.buffered;
          const tracks = player.textTracks;

          const before = [
            String(ranges === player.buffered),
            String(tracks === player.textTracks),
            String(tracks.length),
            tracks[0].id,
            String(ranges.length),
            String(ranges.end(0))
          ].join(':');

          player.insertAdjacentHTML('beforeend', '<track id="track-ja" kind="subtitles" srclang="ja" src="/tracks/ja.vtt">');
          player.currentTime = 6;

          const mutated = [
            String(ranges === player.buffered),
            String(tracks === player.textTracks),
            String(tracks.length),
            tracks[1].id,
            String(ranges.length),
            String(ranges.end(0))
          ].join(':');

          player.removeAttribute('src');

          const emptied = [
            String(ranges === player.buffered),
            String(tracks === player.textTracks),
            String(tracks.length),
            String(ranges.length)
          ].join(':');

          player.setAttribute('src', '/movie-2.mp4');
          player.currentTime = 1.25;

          const restored = [
            String(ranges === player.buffered),
            String(tracks === player.textTracks),
            String(tracks.length),
            tracks[0].id,
            tracks[1].id,
            String(ranges.length),
            String(ranges.end(0))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            mutated,
            emptied,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "true:true:1:track-en:1:2|true:true:2:track-ja:1:6|true:true:2:0|true:true:2:track-en:track-ja:1:1.25",
    )?;
    Ok(())
}

#[test]
fn video_media_methods_update_cached_time_ranges_and_receiver_parity_work() -> Result<()> {
    let html = r#"
        <video id='player' src='/movie.mp4'></video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 4;
          const buffered = player.buffered;
          const seekable = player.seekable;
          const played = player.played;
          const play = player.play;
          const pause = player['pause'];
          const load = player.load;
          let receiverError = '';
          let extraArgError = '';

          try {
            play.call({});
          } catch (error) {
            receiverError = String(error);
          }

          try {
            load.call(player, 1);
          } catch (error) {
            extraArgError = String(error);
          }

          const before = [
            String(player.paused),
            String(buffered === player.buffered),
            String(seekable === player.seekable),
            String(played === player.played),
            String(played.end(0))
          ].join(':');

          const playResult = play.call(player);

          const afterPlay = [
            Object.prototype.toString.call(playResult),
            String(player.paused),
            String(buffered === player.buffered),
            String(played === player.played),
            String(played.end(0))
          ].join(':');

          player.currentTime = 6;
          pause.call(player);

          const afterPause = [
            String(player.paused),
            String(buffered.end(0)),
            String(seekable.end(0)),
            String(played.end(0))
          ].join(':');

          player.removeAttribute('src');
          load.call(player);

          const afterLoad = [
            String(player.paused),
            String(player.currentTime),
            String(buffered === player.buffered),
            String(seekable === player.seekable),
            String(played === player.played),
            String(buffered.length),
            String(seekable.length),
            String(played.length)
          ].join(':');

          player.setAttribute('src', '/movie-2.mp4');
          player.currentTime = 1.5;
          play.call(player);

          const restored = [
            String(player.paused),
            String(buffered === player.buffered),
            String(seekable === player.seekable),
            String(played === player.played),
            String(buffered.end(0)),
            String(seekable.end(0)),
            String(played.end(0)),
            String(receiverError.includes('HTMLMediaElement method called on incompatible receiver')),
            String(extraArgError.includes('load takes no arguments'))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterPlay,
            afterPause,
            afterLoad,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "true:true:true:true:4|[object Promise]:false:true:true:4|true:6:6:6|true:0:true:true:true:0:0:0|false:true:true:true:1.5:1.5:1.5:true:true",
    )?;
    Ok(())
}

#[test]
fn video_media_method_reflective_surface_shadow_and_restore_work() -> Result<()> {
    let html = r#"
        <video id='player' src='/movie.mp4'></video>
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
            String(pauseDesc.value === player.pause),
            String(loadDesc.value === player.load),
            playDesc.value.name,
            String(playDesc.value.length),
            String(playDesc.enumerable === false),
            String(Object.keys(player).includes('play')),
            String(Object.getOwnPropertyNames(player).includes('pause')),
            String(Reflect.ownKeys(player).includes('load')),
            String(Object.hasOwn(player, 'play'))
          ].join(':');

          Object.defineProperty(player, 'play', {
            value: 'shadow-play',
            enumerable: true,
            configurable: true
          });

          const shadowed = [
            String(player.play),
            Object.keys(player).join(','),
            String(Object.getOwnPropertyDescriptor(player, 'play').value === 'shadow-play')
          ].join(':');

          delete player.play;

          const restored = [
            typeof player.play,
            String(player.play === playDesc.value),
            String(Object.keys(player).includes('play')),
            String(Object.getOwnPropertyDescriptor(player, 'play').enumerable === false),
            String(Object.hasOwn(player, 'play'))
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
        "function:true:true:true:true:play:0:true:false:true:true:true|shadow-play:play:true|function:true:false:true:true",
    )?;
    Ok(())
}

#[test]
fn video_cached_media_wrappers_keep_expando_and_prototype_across_load_and_src_changes_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/movie.mp4'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 3;

          const tracks = player.textTracks;
          const ranges = player.buffered;
          const tracksProto = Object.create(TextTrackList.prototype);
          const rangesProto = Object.create(TimeRanges.prototype);

          tracksProto.kindTag = function() { return 'tracks-proto'; };
          rangesProto.kindTag = function() { return 'ranges-proto'; };

          Object.setPrototypeOf(tracks, tracksProto);
          Object.setPrototypeOf(ranges, rangesProto);

          Object.defineProperty(tracks, 'marker', {
            value: 'track-own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(ranges, 'marker', {
            value: 'range-own',
            enumerable: true,
            configurable: true
          });

          const before = [
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            String(tracks.length),
            String(ranges.end(0))
          ].join(':');

          player.insertAdjacentHTML('beforeend', '<track id="track-ja" kind="subtitles" srclang="ja" src="/tracks/ja.vtt">');
          player.load();

          const afterLoad = [
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            tracks.marker,
            ranges.marker,
            String(tracks.length),
            String(ranges.length),
            String(ranges.end(0))
          ].join(':');

          player.setAttribute('src', '/movie-2.mp4');
          player.currentTime = 1.25;

          const afterSrc = [
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            tracks.marker,
            ranges.marker,
            String(tracks.length),
            tracks[1].id,
            String(ranges.length),
            String(ranges.end(0)),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterLoad,
            afterSrc
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "true:true:tracks-proto:ranges-proto:track-own:range-own:1:3|true:true:tracks-proto:ranges-proto:track-own:range-own:2:1:0|true:true:tracks-proto:ranges-proto:track-own:range-own:2:track-ja:1:1.25:track-own:range-own",
    )?;
    Ok(())
}

#[test]
fn video_source_selection_network_state_and_can_play_type_edge_cases_work() -> Result<()> {
    let html = r#"
        <video id='player'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <source id='backup' src='/video/backup.mp4' type='video/mp4'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');

          const before = [
            player.canPlayType(' video/mp4 ; codecs="avc1.42E01E" '),
            String(player.canPlayType('audio') === ''),
            String(player.canPlayType('text/plain') === ''),
            String(player.canPlayType('/mp4') === ''),
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          primary.remove();
          player.load();

          const afterPrimaryRemoval = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.src = '/video/direct.mp4';

          const afterDirectSrc = [
            player.currentSrc,
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          player.removeAttribute('src');
          backup.remove();
          player.load();

          const emptied = [
            String(player.currentSrc === ''),
            String(player.networkState),
            String(player.readyState)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterPrimaryRemoval,
            afterDirectSrc,
            emptied
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "maybe:true:true:true:https://app.local/video/primary.webm:1:0|https://app.local/video/backup.mp4:1:0|https://app.local/video/direct.mp4:1:0|true:0:0",
    )?;
    Ok(())
}

#[test]
fn video_media_collections_reflective_surface_stays_live_across_source_and_track_mutations_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/movie.mp4'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 2;

          const tracks = player.textTracks;
          const ranges = player.buffered;

          Object.defineProperty(tracks, 'marker', {
            value: 'tracks-own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(ranges, 'marker', {
            value: 'ranges-own',
            enumerable: true,
            configurable: true
          });

          const before = [
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            Object.keys(tracks).join(','),
            Object.keys(ranges).join(','),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            String(tracks.length),
            tracks[0].id,
            String(ranges.length),
            String(ranges.end(0))
          ].join(':');

          player.removeAttribute('src');
          player.load();
          player.insertAdjacentHTML('beforeend', '<track id="track-ja" kind="subtitles" srclang="ja" src="/tracks/ja.vtt">');

          const emptied = [
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            Object.keys(tracks).join(','),
            Object.keys(ranges).join(','),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            String(tracks.length),
            tracks[1].id,
            String(ranges.length)
          ].join(':');

          player.setAttribute('src', '/movie-2.mp4');
          player.currentTime = 4.5;

          const restored = [
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            Object.keys(tracks).join(','),
            Object.keys(ranges).join(','),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            String(tracks.length),
            tracks[0].id,
            tracks[1].id,
            String(ranges.length),
            String(ranges.end(0))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            emptied,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "true:true:0,marker:marker:tracks-own:ranges-own:1:track-en:1:2|true:true:0,1,marker:marker:tracks-own:ranges-own:2:track-ja:0|true:true:0,1,marker:marker:tracks-own:ranges-own:2:track-en:track-ja:1:4.5",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_extracted_calls_and_iterators_stay_live_after_child_mutations_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/movie.mp4'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 2;

          const tracks = player.textTracks;
          const ranges = player.buffered;
          const item = tracks.item;
          const values = tracks.values;
          const iterator = tracks[Symbol.iterator];
          const rangeStart = ranges.start;
          const rangeEnd = ranges.end;
          const rangeLength = Object.getOwnPropertyDescriptor(TimeRanges.prototype, 'length').get;

          const beforeValues = values.call(tracks);
          const beforeIter = iterator.call(tracks);
          const before = [
            item.call(tracks, 0).id,
            beforeValues.next().value.id,
            beforeIter.next().value.id,
            String(rangeLength.call(ranges)),
            String(rangeStart.call(ranges, 0)),
            String(rangeEnd.call(ranges, 0))
          ].join(':');

          player.insertAdjacentHTML('beforeend', '<track id="track-ja" kind="subtitles" srclang="ja" src="/tracks/ja.vtt">');
          player.currentTime = 5.5;

          const afterAddValues = values.call(tracks);
          const afterAdd = [
            item.call(tracks, 1).id,
            afterAddValues.next().value.id,
            afterAddValues.next().value.id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(rangeLength.call(ranges)),
            String(rangeEnd.call(ranges, 0))
          ].join(':');

          player.removeAttribute('src');
          player.load();
          document.getElementById('track-en').remove();

          const afterRemoveValues = values.call(tracks);
          const afterRemove = [
            item.call(tracks, 0).id,
            afterRemoveValues.next().value.id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(rangeLength.call(ranges)),
            String(Object.getPrototypeOf(tracks) === TextTrackList.prototype),
            String(Object.getPrototypeOf(ranges) === TimeRanges.prototype)
          ].join(':');

          player.setAttribute('src', '/movie-2.mp4');
          player.currentTime = 1.25;

          const restoredValues = values.call(tracks);
          const restored = [
            item.call(tracks, 0).id,
            restoredValues.next().value.id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(rangeLength.call(ranges)),
            String(rangeStart.call(ranges, 0)),
            String(rangeEnd.call(ranges, 0))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterAdd,
            afterRemove,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "track-en:track-en:track-en:1:0:2|track-ja:track-en:track-ja:track-en,track-ja:1:5.5|track-ja:track-ja:track-ja:0:true:true|track-ja:track-ja:track-ja:1:0:1.25",
    )?;
    Ok(())
}

#[test]
fn video_media_wrappers_keep_reflective_surface_across_source_attribute_mutations_work()
-> Result<()> {
    let html = r#"
        <video id='player'>
          <source id='primary' src='/video/primary.webm' type='video/webm' media='all'>
          <source id='backup' src='/video/backup.mp4' type='video/mp4' media='all'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');
          player.currentTime = 3;

          const tracks = player.textTracks;
          const ranges = player.buffered;
          const tracksProto = Object.create(TextTrackList.prototype);
          const rangesProto = Object.create(TimeRanges.prototype);
          tracksProto.kindTag = function() { return 'tracks-proto'; };
          rangesProto.kindTag = function() { return 'ranges-proto'; };
          Object.setPrototypeOf(tracks, tracksProto);
          Object.setPrototypeOf(ranges, rangesProto);

          Object.defineProperty(tracks, 'marker', {
            value: 'tracks-own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(ranges, 'marker', {
            value: 'ranges-own',
            enumerable: true,
            configurable: true
          });

          const before = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            String(Object.getOwnPropertyDescriptor(tracks, 'marker').enumerable),
            String(Object.getOwnPropertyDescriptor(ranges, 'marker').enumerable),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            String(ranges.end(0))
          ].join(':');

          primary.type = 'text/plain';

          const afterUnsupportedType = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            tracks.marker,
            ranges.marker,
            String(ranges.length),
            String(ranges.end(0))
          ].join(':');

          backup.media = 'not all';
          player.load();

          const afterNoMatch = [
            String(player.currentSrc === ''),
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            tracks.marker,
            ranges.marker,
            String(ranges.length)
          ].join(':');

          backup.media = 'all';
          primary.type = 'video/webm';
          primary.removeAttribute('src');
          primary.setAttribute('srcset', '/video/from-srcset.webm 1x');
          player.currentTime = 1.25;

          const restored = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            String(Reflect.ownKeys(tracks).includes('marker')),
            String(Reflect.ownKeys(ranges).includes('marker')),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            String(ranges.length),
            String(ranges.end(0))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterUnsupportedType,
            afterNoMatch,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/video/primary.webm:true:true:tracks-proto:ranges-proto:true:true:tracks-own:ranges-own:3|https://app.local/video/backup.mp4:true:true:tracks-proto:ranges-proto:tracks-own:ranges-own:1:3|true:true:true:tracks-proto:ranges-proto:tracks-own:ranges-own:0|https://app.local/video/from-srcset.webm:true:true:tracks-proto:ranges-proto:true:true:tracks-own:ranges-own:1:1.25",
    )?;
    Ok(())
}

#[test]
fn video_cached_media_wrappers_keep_object_copy_surface_across_direct_src_precedence_flips_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <source id='backup' src='/video/backup.mp4' type='video/mp4'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');
          player.currentTime = 2;

          const tracks = player.textTracks;
          const ranges = player.buffered;

          Object.defineProperty(tracks, 'marker', {
            value: 'tracks-own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(ranges, 'marker', {
            value: 'ranges-own',
            enumerable: true,
            configurable: true
          });

          const beforeAssignedTracks = Object.assign({}, tracks);
          const beforeSpreadRanges = { ...ranges };
          const before = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            beforeAssignedTracks.marker,
            String(Object.keys(beforeAssignedTracks).includes('0')),
            beforeSpreadRanges.marker,
            String(Object.keys(beforeSpreadRanges).join(',')),
            String(ranges.end(0))
          ].join(':');

          primary.src = '/video/updated.webm';
          player.removeAttribute('src');

          const afterRemovingDirectAssigned = Object.assign({}, tracks);
          const afterRemovingDirectSpread = { ...ranges };
          const afterRemovingDirect = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            afterRemovingDirectAssigned.marker,
            String(Object.keys(afterRemovingDirectAssigned).includes('0')),
            afterRemovingDirectSpread.marker,
            String(Object.keys(afterRemovingDirectSpread).join(',')),
            String(ranges.end(0))
          ].join(':');

          player.src = '/video/direct-2.mp4';
          player.currentTime = 4.5;

          const afterRestoringDirectAssigned = Object.assign({}, tracks);
          const afterRestoringDirectSpread = { ...ranges };
          const afterRestoringDirect = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            afterRestoringDirectAssigned.marker,
            String(Object.keys(afterRestoringDirectAssigned).includes('0')),
            afterRestoringDirectSpread.marker,
            String(Object.keys(afterRestoringDirectSpread).join(',')),
            String(ranges.end(0))
          ].join(':');

          player.removeAttribute('src');
          primary.removeAttribute('src');
          backup.removeAttribute('src');
          backup.setAttribute('srcset', '/video/fallback-from-srcset.mp4 1x');

          const afterBackupFallbackAssigned = Object.assign({}, tracks);
          const afterBackupFallbackSpread = { ...ranges };
          const afterBackupFallback = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            afterBackupFallbackAssigned.marker,
            String(Object.keys(afterBackupFallbackAssigned).includes('0')),
            afterBackupFallbackSpread.marker,
            String(Object.keys(afterBackupFallbackSpread).join(',')),
            String(ranges.end(0))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterRestoringDirect,
            afterBackupFallback
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/video/direct.mp4:true:true:tracks-own:true:ranges-own:marker:2|https://app.local/video/updated.webm:true:true:tracks-own:true:ranges-own:marker:2|https://app.local/video/direct-2.mp4:true:true:tracks-own:true:ranges-own:marker:4.5|https://app.local/video/fallback-from-srcset.mp4:true:true:tracks-own:true:ranges-own:marker:4.5",
    )?;
    Ok(())
}

#[test]
fn video_cached_media_wrapper_descriptors_survive_precedence_churn_and_load_work() -> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <source id='backup' src='/video/backup.mp4' type='video/mp4'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');
          player.currentTime = 2;

          const tracks = player.textTracks;
          const ranges = player.buffered;
          const tracksProto = Object.create(TextTrackList.prototype);
          const rangesProto = Object.create(TimeRanges.prototype);
          tracksProto.kindTag = function() { return 'tracks-proto'; };
          rangesProto.kindTag = function() { return 'ranges-proto'; };
          Object.setPrototypeOf(tracks, tracksProto);
          Object.setPrototypeOf(ranges, rangesProto);

          Object.defineProperty(tracks, 'marker', {
            value: 'tracks-own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(ranges, 'marker', {
            value: 'ranges-own',
            enumerable: true,
            configurable: true
          });

          const before = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            String(Object.getOwnPropertyDescriptor(tracks, 'marker').enumerable),
            String(Object.getOwnPropertyDescriptor(ranges, 'marker').enumerable),
            String(Object.hasOwn(tracks, 'length')),
            String('length' in tracks),
            String(Object.hasOwn(ranges, 'length')),
            String('length' in ranges),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker
          ].join(':');

          player.removeAttribute('src');
          primary.src = '/video/nested-updated.webm';
          player.load();
          player.currentTime = 3.5;

          const afterNested = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            String(Object.getOwnPropertyDescriptor(tracks, 'marker').enumerable),
            String(Object.getOwnPropertyDescriptor(ranges, 'marker').enumerable),
            String(Object.hasOwn(tracks, 'length')),
            String('length' in tracks),
            String(Object.hasOwn(ranges, 'length')),
            String('length' in ranges),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            String(ranges.end(0))
          ].join(':');

          player.src = '/video/direct-2.mp4';
          player.load();
          player.currentTime = 1.25;

          const afterDirect = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            String(Reflect.ownKeys(tracks).includes('marker')),
            String(Reflect.ownKeys(ranges).includes('marker')),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            String(ranges.end(0))
          ].join(':');

          player.removeAttribute('src');
          primary.removeAttribute('src');
          backup.removeAttribute('src');
          backup.setAttribute('srcset', '/video/fallback-from-srcset.mp4 1x');
          player.load();
          player.currentTime = 4.75;
          delete tracks.marker;
          delete ranges.marker;

          const restored = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            tracks.kindTag(),
            ranges.kindTag(),
            String(Object.hasOwn(tracks, 'marker')),
            String(Object.hasOwn(ranges, 'marker')),
            String('marker' in tracks),
            String('marker' in ranges),
            String(Object.keys(Object.assign({}, tracks)).length),
            String(Object.keys({ ...ranges }).length),
            String(ranges.end(0))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterNested,
            afterDirect,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/video/direct.mp4:true:true:tracks-proto:ranges-proto:true:true:true:true:false:true:tracks-own:ranges-own|https://app.local/video/nested-updated.webm:true:true:tracks-proto:ranges-proto:true:true:true:true:false:true:tracks-own:ranges-own:3.5|https://app.local/video/direct-2.mp4:true:true:tracks-proto:ranges-proto:true:true:tracks-own:ranges-own:1.25|https://app.local/video/fallback-from-srcset.mp4:true:true:tracks-proto:ranges-proto:false:false:false:false:1:0:4.75",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_iterators_and_property_paths_stay_live_across_fallback_churn_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='primary' src='/video/primary.webm' type='video/webm' media='all'>
          <source id='backup' srcset='/video/backup-from-srcset.mp4 1x' type='video/mp4' media='all'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');
          player.currentTime = 2;

          const tracks = player.textTracks;
          const ranges = player.buffered;
          const holder = Object.create({ tracks, ranges });
          const item = holder.tracks['item'];
          const values = holder.tracks['values'];
          const iterator = holder.tracks[Symbol.iterator];
          const start = holder.ranges['start'];
          const end = Object.getPrototypeOf(holder.ranges)['end'];
          const lengthGetter = Object.getOwnPropertyDescriptor(TimeRanges.prototype, 'length').get;

          const beforeValues = values.call(holder.tracks);
          const before = [
            player.currentSrc,
            item.call(holder.tracks, 0).id,
            beforeValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(start.call(holder.ranges, 0)),
            String(end.call(holder.ranges, 0))
          ].join(':');

          player.removeAttribute('src');
          player.insertAdjacentHTML('beforeend', '<track id="track-ja" kind="subtitles" srclang="ja" src="/tracks/ja.vtt">');
          player.currentTime = 5.5;

          const nestedValues = values.call(holder.tracks);
          const afterNested = [
            player.currentSrc,
            item.call(holder.tracks, 1).id,
            nestedValues.next().value.id,
            nestedValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(end.call(holder.ranges, 0))
          ].join(':');

          primary.type = 'text/plain';

          const backupValues = values.call(holder.tracks);
          const afterBackupFallback = [
            player.currentSrc,
            item.call(holder.tracks, 0).id,
            backupValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(end.call(holder.ranges, 0))
          ].join(':');

          backup.removeAttribute('srcset');
          player.load();

          const emptiedValues = values.call(holder.tracks);
          const emptied = [
            String(player.currentSrc === ''),
            item.call(holder.tracks, 1).id,
            emptiedValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(Object.getPrototypeOf(holder.tracks) === TextTrackList.prototype),
            String(Object.getPrototypeOf(holder.ranges) === TimeRanges.prototype)
          ].join(':');

          player.src = '/video/direct-2.mp4';
          player.currentTime = 1.25;

          const restoredValues = values.call(holder.tracks);
          const restored = [
            player.currentSrc,
            item.call(holder.tracks, 0).id,
            restoredValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(start.call(holder.ranges, 0)),
            String(end.call(holder.ranges, 0))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterNested,
            afterBackupFallback,
            emptied,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/video/direct.mp4:track-en:track-en:track-en:1:0:2|https://app.local/video/primary.webm:track-ja:track-en:track-ja:track-en,track-ja:1:5.5|https://app.local/video/backup-from-srcset.mp4:track-en:track-en:track-en,track-ja:1:5.5|true:track-ja:track-en:track-en,track-ja:0:true:true|https://app.local/video/direct-2.mp4:track-en:track-en:track-en,track-ja:1:0:1.25",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_alias_paths_stay_live_across_source_normalization_churn_work() -> Result<()>
{
    let html = r#"
        <video id='player' src=' ./video/direct.mp4 '>
          <source id='primary' src='   ' type='video/webm'>
          <source id='backup' srcset=' ./video/from-srcset.mp4 1x, ./video/ignored.mp4 2x ' type='video/mp4'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');
          player.currentTime = 2;

          const aliases = {
            media: player,
            wrappers: [player.textTracks, player.buffered]
          };
          const item = aliases.wrappers[0]['item'];
          const values = aliases.wrappers[0]['values'];
          const iterator = aliases.wrappers[0][Symbol.iterator];
          const start = aliases.wrappers[1]['start'];
          const end = aliases.wrappers[1]['end'];
          const lengthGetter = Object.getOwnPropertyDescriptor(TimeRanges.prototype, 'length').get;

          const beforeValues = values.call(aliases.wrappers[0]);
          const before = [
            aliases.media.currentSrc,
            item.call(aliases.wrappers[0], 0).id,
            beforeValues.next().value.id,
            Array.from(iterator.call(aliases.wrappers[0])).map((track) => track.id).join(','),
            String(lengthGetter.call(aliases.wrappers[1])),
            String(start.call(aliases.wrappers[1], 0)),
            String(end.call(aliases.wrappers[1], 0))
          ].join(':');

          player.src = '   ';
          player.insertAdjacentHTML('beforeend', '<track id="track-ja" kind="subtitles" srclang="ja" src="/tracks/ja.vtt">');
          player.currentTime = 5.5;

          const whitespaceValues = values.call(aliases.wrappers[0]);
          const whitespaceDirect = [
            aliases.media.currentSrc,
            item.call(aliases.wrappers[0], 1).id,
            whitespaceValues.next().value.id,
            whitespaceValues.next().value.id,
            Array.from(iterator.call(aliases.wrappers[0])).map((track) => track.id).join(','),
            String(lengthGetter.call(aliases.wrappers[1])),
            String(end.call(aliases.wrappers[1], 0))
          ].join(':');

          player.removeAttribute('src');

          const srcsetValues = values.call(aliases.wrappers[0]);
          const srcsetFallback = [
            aliases.media.currentSrc,
            item.call(aliases.wrappers[0], 0).id,
            srcsetValues.next().value.id,
            Array.from(iterator.call(aliases.wrappers[0])).map((track) => track.id).join(','),
            String(lengthGetter.call(aliases.wrappers[1])),
            String(end.call(aliases.wrappers[1], 0))
          ].join(':');

          backup.srcset = '   ';
          primary.src = ' ./video/primary-restored.webm ';
          player.currentTime = 1.25;

          const restoredValues = values.call(aliases.wrappers[0]);
          const restoredPrimary = [
            aliases.media.currentSrc,
            item.call(aliases.wrappers[0], 1).id,
            restoredValues.next().value.id,
            Array.from(iterator.call(aliases.wrappers[0])).map((track) => track.id).join(','),
            String(lengthGetter.call(aliases.wrappers[1])),
            String(start.call(aliases.wrappers[1], 0)),
            String(end.call(aliases.wrappers[1], 0))
          ].join(':');

          primary.src = '   ';
          player.load();

          const emptiedValues = values.call(aliases.wrappers[0]);
          const emptied = [
            String(aliases.media.currentSrc === ''),
            item.call(aliases.wrappers[0], 1).id,
            emptiedValues.next().value.id,
            Array.from(iterator.call(aliases.wrappers[0])).map((track) => track.id).join(','),
            String(lengthGetter.call(aliases.wrappers[1])),
            String(Object.getPrototypeOf(aliases.wrappers[0]) === TextTrackList.prototype),
            String(Object.getPrototypeOf(aliases.wrappers[1]) === TimeRanges.prototype)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            whitespaceDirect,
            srcsetFallback,
            restoredPrimary,
            emptied
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/watch/video/direct.mp4:track-en:track-en:track-en:1:0:2|https://app.local/watch/index.html:track-ja:track-en:track-ja:track-en,track-ja:1:5.5|https://app.local/watch/video/from-srcset.mp4:track-en:track-en:track-en,track-ja:1:5.5|https://app.local/watch/video/primary-restored.webm:track-ja:track-en:track-en,track-ja:1:0:1.25|true:track-ja:track-en:track-en,track-ja:0:true:true",
    )?;
    Ok(())
}

#[test]
fn video_cached_media_wrappers_persist_across_source_reorder_disconnect_and_replace_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <source id='backup' src='/video/backup.mp4' type='video/mp4'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const primary = document.getElementById('primary');
          const backup = document.getElementById('backup');
          player.currentTime = 2;

          const tracks = player.textTracks;
          const ranges = player.buffered;
          const tracksProto = Object.create(TextTrackList.prototype);
          const rangesProto = Object.create(TimeRanges.prototype);
          tracksProto.kindTag = function() { return 'tracks-proto'; };
          rangesProto.kindTag = function() { return 'ranges-proto'; };
          Object.setPrototypeOf(tracks, tracksProto);
          Object.setPrototypeOf(ranges, rangesProto);

          Object.defineProperty(tracks, 'marker', {
            value: 'tracks-own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(ranges, 'marker', {
            value: 'ranges-own',
            enumerable: true,
            configurable: true
          });

          const item = tracks.item;
          const iterator = tracks[Symbol.iterator];
          const rangeEnd = ranges.end;

          const before = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(Object.getPrototypeOf(tracks) === tracksProto),
            String(Object.getPrototypeOf(ranges) === rangesProto),
            tracks.kindTag(),
            ranges.kindTag(),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            item.call(tracks, 0).id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(rangeEnd.call(ranges, 0))
          ].join(':');

          player.removeAttribute('src');
          player.appendChild(primary);
          player.currentTime = 3.5;

          const afterReorder = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(Object.getPrototypeOf(tracks) === tracksProto),
            String(Object.getPrototypeOf(ranges) === rangesProto),
            tracks.kindTag(),
            ranges.kindTag(),
            tracks.marker,
            ranges.marker,
            item.call(tracks, 0).id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(rangeEnd.call(ranges, 0))
          ].join(':');

          const detachedBackup = player.removeChild(backup);

          const afterDisconnect = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(Object.getPrototypeOf(tracks) === tracksProto),
            String(Object.getPrototypeOf(ranges) === rangesProto),
            tracks.marker,
            ranges.marker,
            item.call(tracks, 0).id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(rangeEnd.call(ranges, 0))
          ].join(':');

          detachedBackup.src = '/video/backup-return.mp4';
          player.insertBefore(detachedBackup, primary);
          player.currentTime = 4.5;

          const afterReinsert = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(Object.getPrototypeOf(tracks) === tracksProto),
            String(Object.getPrototypeOf(ranges) === rangesProto),
            tracks.marker,
            ranges.marker,
            item.call(tracks, 0).id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(rangeEnd.call(ranges, 0))
          ].join(':');

          const replacement = document.createElement('source');
          replacement.id = 'replacement';
          replacement.type = 'video/mp4';
          replacement.srcset = '/video/replacement-from-srcset.mp4 1x';
          player.replaceChild(replacement, detachedBackup);
          player.load();
          player.currentTime = 1.25;

          const afterReplace = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(Object.getPrototypeOf(tracks) === tracksProto),
            String(Object.getPrototypeOf(ranges) === rangesProto),
            String(Object.getOwnPropertyDescriptor(tracks, 'marker').enumerable),
            String(Object.getOwnPropertyDescriptor(ranges, 'marker').enumerable),
            Object.assign({}, tracks).marker,
            Object.assign({}, ranges).marker,
            item.call(tracks, 0).id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(rangeEnd.call(ranges, 0))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterReorder,
            afterDisconnect,
            afterReinsert,
            afterReplace
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/video/direct.mp4:true:true:true:true:tracks-proto:ranges-proto:tracks-own:ranges-own:track-en:track-en:2|https://app.local/video/backup.mp4:true:true:true:true:tracks-proto:ranges-proto:tracks-own:ranges-own:track-en:track-en:3.5|https://app.local/video/primary.webm:true:true:true:true:tracks-own:ranges-own:track-en:track-en:3.5|https://app.local/video/backup-return.mp4:true:true:true:true:tracks-own:ranges-own:track-en:track-en:4.5|https://app.local/video/replacement-from-srcset.mp4:true:true:true:true:true:true:tracks-own:ranges-own:track-en:track-en:1.25",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_callables_stay_live_across_source_clone_and_fragment_churn_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='base' src='/video/base.webm' type='video/webm'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const base = document.getElementById('base');
          player.currentTime = 2;

          const tracks = player.textTracks;
          const ranges = player.buffered;
          const item = tracks.item;
          const values = tracks.values;
          const iterator = tracks[Symbol.iterator];
          const start = ranges.start;
          const end = ranges.end;
          const lengthGetter = Object.getOwnPropertyDescriptor(TimeRanges.prototype, 'length').get;

          const beforeValues = values.call(tracks);
          const before = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(tracks.item === item),
            String(tracks.values === values),
            String(tracks[Symbol.iterator] === iterator),
            String(ranges.start === start),
            String(ranges.end === end),
            item.call(tracks, 0).id,
            beforeValues.next().value.id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(ranges)),
            String(start.call(ranges, 0)),
            String(end.call(ranges, 0))
          ].join(':');

          player.removeAttribute('src');

          const fragment = document.createDocumentFragment();
          const invalidClone = base.cloneNode();
          invalidClone.id = 'invalid';
          invalidClone.type = 'text/plain';
          invalidClone.src = '/video/invalid.txt';
          const preferredClone = base.cloneNode();
          preferredClone.id = 'preferred';
          preferredClone.type = 'video/mp4';
          preferredClone.src = '/video/clone-preferred.mp4';
          fragment.appendChild(invalidClone);
          fragment.appendChild(preferredClone);
          player.insertBefore(fragment, base);
          player.currentTime = 3.5;

          const afterFragmentValues = values.call(tracks);
          const afterFragmentInsert = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(tracks.item === item),
            String(tracks.values === values),
            String(tracks[Symbol.iterator] === iterator),
            String(ranges.start === start),
            String(ranges.end === end),
            item.call(tracks, 0).id,
            afterFragmentValues.next().value.id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(ranges)),
            String(end.call(ranges, 0))
          ].join(':');

          const detachedPreferred = player.removeChild(preferredClone);

          const afterDetachValues = values.call(tracks);
          const afterDetachPreferred = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(tracks.item === item),
            String(tracks.values === values),
            String(tracks[Symbol.iterator] === iterator),
            String(ranges.start === start),
            String(ranges.end === end),
            item.call(tracks, 0).id,
            afterDetachValues.next().value.id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(ranges)),
            String(end.call(ranges, 0))
          ].join(':');

          detachedPreferred.src = '/video/reused-return.mp4';
          const reinsertFragment = document.createDocumentFragment();
          const replacementClone = base.cloneNode();
          replacementClone.id = 'replacement';
          replacementClone.type = 'video/mp4';
          replacementClone.removeAttribute('src');
          replacementClone.srcset = '/video/replacement-from-srcset.mp4 1x';
          reinsertFragment.appendChild(detachedPreferred);
          reinsertFragment.appendChild(replacementClone);
          player.insertBefore(reinsertFragment, base);
          player.currentTime = 4.5;

          const afterReinsertValues = values.call(tracks);
          const afterReinsertBatch = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(tracks.item === item),
            String(tracks.values === values),
            String(tracks[Symbol.iterator] === iterator),
            String(ranges.start === start),
            String(ranges.end === end),
            item.call(tracks, 0).id,
            afterReinsertValues.next().value.id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(ranges)),
            String(end.call(ranges, 0))
          ].join(':');

          player.removeChild(detachedPreferred);
          player.load();
          player.currentTime = 1.25;

          const afterReplacementValues = values.call(tracks);
          const afterReplacementWins = [
            player.currentSrc,
            String(tracks === player.textTracks),
            String(ranges === player.buffered),
            String(tracks.item === item),
            String(tracks.values === values),
            String(tracks[Symbol.iterator] === iterator),
            String(ranges.start === start),
            String(ranges.end === end),
            item.call(tracks, 0).id,
            afterReplacementValues.next().value.id,
            Array.from(iterator.call(tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(ranges)),
            String(start.call(ranges, 0)),
            String(end.call(ranges, 0))
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterFragmentInsert,
            afterDetachPreferred,
            afterReinsertBatch,
            afterReplacementWins
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/video/direct.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:2|https://app.local/video/clone-preferred.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:3.5|https://app.local/video/base.webm:true:true:true:true:true:true:true:track-en:track-en:track-en:1:3.5|https://app.local/video/reused-return.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:4.5|https://app.local/video/replacement-from-srcset.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:1.25",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_receiver_paths_stay_live_across_source_adoption_and_reparent_work()
-> Result<()> {
    let html = r#"
        <video id='left' src='/video/left-direct.mp4'>
          <source id='shared' src='/video/shared.mp4' type='video/mp4'>
          <source id='left-base' src='/video/left-base.webm' type='video/webm'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <video id='right'>
          <source id='right-base' src='/video/right-base.webm' type='video/webm'>
        </video>
        <div id='stash'></div>
        <p id='result'></p>
        <script>
          const left = document.getElementById('left');
          const right = document.getElementById('right');
          const shared = document.getElementById('shared');
          const leftBase = document.getElementById('left-base');
          const rightBase = document.getElementById('right-base');
          const stash = document.getElementById('stash');
          left.currentTime = 2;

          const holder = Object.create({
            media: left,
            tracks: left.textTracks,
            ranges: left.buffered
          });
          const item = holder.tracks['item'];
          const values = Object.getPrototypeOf(holder).tracks.values;
          const iterator = holder.tracks[Symbol.iterator];
          const start = holder.ranges['start'];
          const end = Object.getPrototypeOf(holder).ranges.end;
          const lengthGetter = Object.getOwnPropertyDescriptor(TimeRanges.prototype, 'length').get;

          const beforeValues = values.call(holder.tracks);
          const before = [
            holder.media.currentSrc,
            right.currentSrc,
            String(holder.tracks === left.textTracks),
            String(holder.ranges === left.buffered),
            item.call(holder.tracks, 0).id,
            beforeValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(start.call(holder.ranges, 0)),
            String(end.call(holder.ranges, 0))
          ].join(':');

          left.removeAttribute('src');
          left.currentTime = 3.5;

          const afterRemovingDirectValues = values.call(holder.tracks);
          const afterRemovingDirect = [
            holder.media.currentSrc,
            right.currentSrc,
            String(holder.tracks === left.textTracks),
            String(holder.ranges === left.buffered),
            item.call(holder.tracks, 0).id,
            afterRemovingDirectValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(end.call(holder.ranges, 0))
          ].join(':');

          right.insertBefore(shared, rightBase);

          const afterMoveToRightValues = values.call(holder.tracks);
          const afterMoveToRight = [
            holder.media.currentSrc,
            right.currentSrc,
            String(holder.tracks === left.textTracks),
            String(holder.ranges === left.buffered),
            item.call(holder.tracks, 0).id,
            afterMoveToRightValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(end.call(holder.ranges, 0))
          ].join(':');

          stash.appendChild(shared);

          const afterStashValues = values.call(holder.tracks);
          const afterStash = [
            holder.media.currentSrc,
            right.currentSrc,
            String(holder.tracks === left.textTracks),
            String(holder.ranges === left.buffered),
            item.call(holder.tracks, 0).id,
            afterStashValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(end.call(holder.ranges, 0))
          ].join(':');

          shared.src = '/video/shared-return.mp4';
          left.insertBefore(shared, leftBase);
          left.currentTime = 4.5;

          const afterReturnValues = values.call(holder.tracks);
          const afterReturnLeft = [
            holder.media.currentSrc,
            right.currentSrc,
            String(holder.tracks === left.textTracks),
            String(holder.ranges === left.buffered),
            item.call(holder.tracks, 0).id,
            afterReturnValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(end.call(holder.ranges, 0))
          ].join(':');

          left.src = '/video/left-direct-2.mp4';
          left.currentTime = 1.25;

          const restoredValues = values.call(holder.tracks);
          const restoredDirect = [
            holder.media.currentSrc,
            right.currentSrc,
            String(holder.tracks === left.textTracks),
            String(holder.ranges === left.buffered),
            item.call(holder.tracks, 0).id,
            restoredValues.next().value.id,
            Array.from(iterator.call(holder.tracks)).map((track) => track.id).join(','),
            String(lengthGetter.call(holder.ranges)),
            String(start.call(holder.ranges, 0)),
            String(end.call(holder.ranges, 0))
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
        "https://app.local/video/left-direct.mp4:https://app.local/video/right-base.webm:true:true:track-en:track-en:track-en:1:0:2|https://app.local/video/shared.mp4:https://app.local/video/right-base.webm:true:true:track-en:track-en:track-en:1:3.5|https://app.local/video/left-base.webm:https://app.local/video/shared.mp4:true:true:track-en:track-en:track-en:1:3.5|https://app.local/video/left-base.webm:https://app.local/video/right-base.webm:true:true:track-en:track-en:track-en:1:3.5|https://app.local/video/shared-return.mp4:https://app.local/video/right-base.webm:true:true:track-en:track-en:track-en:1:4.5|https://app.local/video/left-direct-2.mp4:https://app.local/video/right-base.webm:true:true:track-en:track-en:track-en:1:0:1.25",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_borrowed_calls_stay_live_across_source_sibling_and_filter_churn_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='bad-type' src='/video/bad.txt' type='text/plain'>
          <source id='filtered' src='/video/filtered.mp4' type='video/mp4' media='not all'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const badType = document.getElementById('bad-type');
          const filtered = document.getElementById('filtered');
          const primary = document.getElementById('primary');
          player.currentTime = 2;

          const alias = {
            wrappers: {
              tracks: player.textTracks,
              ranges: player.buffered
            }
          };

          const item = alias.wrappers.tracks.item;
          const values = alias.wrappers.tracks.values;
          const iterator = alias.wrappers.tracks[Symbol.iterator];
          const start = alias.wrappers.ranges.start;
          const end = alias.wrappers.ranges.end;
          const lengthGetter = Object.getOwnPropertyDescriptor(TimeRanges.prototype, 'length').get;
          function borrowItem(receiver, index) {
            return Function.prototype.call.call(item, receiver, index);
          }

          function borrowValues(receiver) {
            return Function.prototype.apply.call(values, receiver, []);
          }

          function borrowIterator(receiver) {
            return Function.prototype.call.call(iterator, receiver);
          }

          function borrowStart(receiver, index) {
            return Function.prototype.call.call(start, receiver, index);
          }

          function borrowEnd(receiver, index) {
            return Function.prototype.apply.call(end, receiver, [index]);
          }

          function borrowLength(receiver) {
            return Function.prototype.call.call(lengthGetter, receiver);
          }

          const beforeValues = borrowValues(alias.wrappers.tracks);
          const before = [
            player.currentSrc,
            String(alias.wrappers.tracks === player.textTracks),
            String(alias.wrappers.ranges === player.buffered),
            borrowItem(alias.wrappers.tracks, 0).id,
            beforeValues.next().value.id,
            Array.from(borrowIterator(alias.wrappers.tracks)).map((track) => track.id).join(','),
            String(borrowLength(alias.wrappers.ranges)),
            String(borrowStart(alias.wrappers.ranges, 0)),
            String(borrowEnd(alias.wrappers.ranges, 0))
          ].join(':');

          player.removeAttribute('src');
          player.currentTime = 3.5;

          const afterRemovingDirectValues = borrowValues(alias.wrappers.tracks);
          const afterRemovingDirect = [
            player.currentSrc,
            String(alias.wrappers.tracks === player.textTracks),
            String(alias.wrappers.ranges === player.buffered),
            borrowItem(alias.wrappers.tracks, 0).id,
            afterRemovingDirectValues.next().value.id,
            Array.from(borrowIterator(alias.wrappers.tracks)).map((track) => track.id).join(','),
            String(borrowLength(alias.wrappers.ranges)),
            String(borrowEnd(alias.wrappers.ranges, 0))
          ].join(':');

          const spacer = document.createElement('div');
          spacer.id = 'spacer';
          player.insertBefore(spacer, primary);

          const afterSiblingInsertValues = borrowValues(alias.wrappers.tracks);
          const afterSiblingInsert = [
            player.currentSrc,
            String(alias.wrappers.tracks === player.textTracks),
            String(alias.wrappers.ranges === player.buffered),
            borrowItem(alias.wrappers.tracks, 0).id,
            afterSiblingInsertValues.next().value.id,
            Array.from(borrowIterator(alias.wrappers.tracks)).map((track) => track.id).join(','),
            String(borrowLength(alias.wrappers.ranges)),
            String(borrowEnd(alias.wrappers.ranges, 0))
          ].join(':');

          badType.type = 'video/mp4';
          player.currentTime = 4.5;

          const afterTypeToggleValues = borrowValues(alias.wrappers.tracks);
          const afterTypeToggle = [
            player.currentSrc,
            String(alias.wrappers.tracks === player.textTracks),
            String(alias.wrappers.ranges === player.buffered),
            borrowItem(alias.wrappers.tracks, 0).id,
            afterTypeToggleValues.next().value.id,
            Array.from(borrowIterator(alias.wrappers.tracks)).map((track) => track.id).join(','),
            String(borrowLength(alias.wrappers.ranges)),
            String(borrowEnd(alias.wrappers.ranges, 0))
          ].join(':');

          badType.src = '   ';
          filtered.removeAttribute('media');
          player.currentTime = 5.5;

          const afterMediaToggleValues = borrowValues(alias.wrappers.tracks);
          const afterMediaToggle = [
            player.currentSrc,
            String(alias.wrappers.tracks === player.textTracks),
            String(alias.wrappers.ranges === player.buffered),
            borrowItem(alias.wrappers.tracks, 0).id,
            afterMediaToggleValues.next().value.id,
            Array.from(borrowIterator(alias.wrappers.tracks)).map((track) => track.id).join(','),
            String(borrowLength(alias.wrappers.ranges)),
            String(borrowEnd(alias.wrappers.ranges, 0))
          ].join(':');

          filtered.src = '   ';
          primary.src = '   ';
          player.load();

          const afterEmptyingValues = borrowValues(alias.wrappers.tracks);
          const afterEmptyingCandidates = [
            String(player.currentSrc === ''),
            String(alias.wrappers.tracks === player.textTracks),
            String(alias.wrappers.ranges === player.buffered),
            borrowItem(alias.wrappers.tracks, 0).id,
            afterEmptyingValues.next().value.id,
            Array.from(borrowIterator(alias.wrappers.tracks)).map((track) => track.id).join(','),
            String(borrowLength(alias.wrappers.ranges)),
            String(Object.getPrototypeOf(alias.wrappers.tracks) === TextTrackList.prototype),
            String(Object.getPrototypeOf(alias.wrappers.ranges) === TimeRanges.prototype)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterSiblingInsert,
            afterTypeToggle,
            afterMediaToggle,
            afterEmptyingCandidates
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/video/direct.mp4:true:true:track-en:track-en:track-en:1:0:2|https://app.local/video/primary.webm:true:true:track-en:track-en:track-en:1:3.5|https://app.local/video/primary.webm:true:true:track-en:track-en:track-en:1:3.5|https://app.local/video/bad.txt:true:true:track-en:track-en:track-en:1:4.5|https://app.local/video/filtered.mp4:true:true:track-en:track-en:track-en:1:5.5|true:true:true:track-en:track-en:track-en:0:true:true",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_descriptor_and_copy_surface_stays_live_across_source_batch_resets_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          const track = document.getElementById('track-en');
          player.currentTime = 2;

          const trackList = player.textTracks;
          const timeRanges = player.buffered;
          const item = trackList.item;
          const values = trackList.values;
          Object.defineProperty(trackList, 'marker', {
            value: 'tracks-own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(timeRanges, 'marker', {
            value: 'ranges-own',
            enumerable: true,
            configurable: true
          });

          function snapshot() {
            return [
              player.currentSrc,
              String(trackList === player.textTracks),
              String(timeRanges === player.buffered),
              String(Object.getOwnPropertyDescriptor(trackList, 'marker').enumerable),
              String(Object.getOwnPropertyDescriptor(timeRanges, 'marker').enumerable),
              String(Reflect.ownKeys(trackList).includes('marker')),
              String(Reflect.ownKeys(timeRanges).includes('marker')),
              Object.assign({}, trackList).marker,
              ({ ...trackList }).marker,
              Object.assign({}, timeRanges).marker,
              ({ ...timeRanges }).marker,
              item.call(trackList, 0).id,
              Array.from(values.call(trackList)).map((entry) => entry.id).join(','),
              String(timeRanges.length),
              String(timeRanges.end(0))
            ].join(':');
          }

          const before = snapshot();

          player.removeAttribute('src');
          const firstBatch = document.createDocumentFragment();
          const invalid = document.createElement('source');
          invalid.type = 'text/plain';
          invalid.src = '/video/invalid.txt';
          const preferred = document.createElement('source');
          preferred.type = 'video/mp4';
          preferred.src = '/video/first-reset.mp4';
          firstBatch.appendChild(invalid);
          firstBatch.appendChild(preferred);
          player.replaceChildren(firstBatch, track);
          player.currentTime = 3.5;

          const afterFirstReset = snapshot();

          player.replaceChildren(track);
          player.load();

          const afterClearingAll = [
            String(player.currentSrc === ''),
            String(trackList === player.textTracks),
            String(timeRanges === player.buffered),
            String(Object.getOwnPropertyDescriptor(trackList, 'marker').enumerable),
            String(Object.getOwnPropertyDescriptor(timeRanges, 'marker').enumerable),
            String(Reflect.ownKeys(trackList).includes('marker')),
            String(Reflect.ownKeys(timeRanges).includes('marker')),
            Object.assign({}, trackList).marker,
            ({ ...trackList }).marker,
            Object.assign({}, timeRanges).marker,
            ({ ...timeRanges }).marker,
            item.call(trackList, 0).id,
            Array.from(values.call(trackList)).map((entry) => entry.id).join(','),
            String(timeRanges.length)
          ].join(':');

          const secondBatch = document.createDocumentFragment();
          const filtered = document.createElement('source');
          filtered.type = 'video/mp4';
          filtered.media = 'not all';
          filtered.src = '/video/filtered.mp4';
          const empty = document.createElement('source');
          empty.type = 'video/webm';
          empty.src = '   ';
          const fallback = document.createElement('source');
          fallback.type = 'video/mp4';
          fallback.srcset = '/video/second-reset-from-srcset.mp4 1x';
          secondBatch.appendChild(filtered);
          secondBatch.appendChild(empty);
          secondBatch.appendChild(fallback);
          player.replaceChildren(secondBatch, track);
          player.currentTime = 1.25;

          const afterSecondReset = snapshot();

          player.src = '/video/direct-2.mp4';

          const restoredDirect = snapshot();

          document.getElementById('result').textContent = [
            before,
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
        "https://app.local/video/direct.mp4:true:true:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:track-en:1:2|https://app.local/video/first-reset.mp4:true:true:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:track-en:1:3.5|true:true:true:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:track-en:0|https://app.local/video/second-reset-from-srcset.mp4:true:true:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:track-en:1:1.25|https://app.local/video/direct-2.mp4:true:true:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:track-en:1:1.25",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_expando_and_prototype_stays_live_across_source_string_rebuilds_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 2;

          const trackList = player.textTracks;
          const timeRanges = player.buffered;
          const trackProto = Object.create(TextTrackList.prototype);
          const rangeProto = Object.create(TimeRanges.prototype);
          trackProto.kindTag = function() { return 'tracks-proto'; };
          rangeProto.kindTag = function() { return 'ranges-proto'; };
          Object.setPrototypeOf(trackList, trackProto);
          Object.setPrototypeOf(timeRanges, rangeProto);

          Object.defineProperty(trackList, 'marker', {
            value: 'tracks-own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(timeRanges, 'marker', {
            value: 'ranges-own',
            enumerable: true,
            configurable: true
          });

          const item = trackList.item;
          const end = timeRanges.end;

          function snapshot() {
            return [
              player.currentSrc,
              String(trackList === player.textTracks),
              String(timeRanges === player.buffered),
              String(Object.getPrototypeOf(trackList) === trackProto),
              String(Object.getPrototypeOf(timeRanges) === rangeProto),
              trackList.kindTag(),
              timeRanges.kindTag(),
              String(Object.getOwnPropertyDescriptor(trackList, 'marker').enumerable),
              String(Object.getOwnPropertyDescriptor(timeRanges, 'marker').enumerable),
              String(Reflect.ownKeys(trackList).includes('marker')),
              String(Reflect.ownKeys(timeRanges).includes('marker')),
              Object.assign({}, trackList).marker,
              ({ ...trackList }).marker,
              Object.assign({}, timeRanges).marker,
              ({ ...timeRanges }).marker,
              item.call(trackList, 0).id,
              String(end.call(timeRanges, 0))
            ].join(':');
          }

          const before = snapshot();

          player.removeAttribute('src');
          player.innerHTML = '<source id="bad" src="/video/bad.txt" type="text/plain"><source id="rebuilt" src="/video/rebuilt.mp4" type="video/mp4"><track id="track-en" kind="captions" srclang="en" src="/tracks/en.vtt">';
          player.currentTime = 3.5;

          const afterInnerHtml = snapshot();

          player.insertAdjacentHTML('afterbegin', '<source id="leading" src="/video/leading.webm" type="video/webm">');
          player.currentTime = 4.5;

          const afterInsertAdjacent = snapshot();

          document.getElementById('leading').removeAttribute('src');
          document.getElementById('bad').type = 'video/mp4';

          const afterMixedToggle = snapshot();

          player.innerHTML = '<source id="filtered" src="/video/filtered.mp4" type="video/mp4" media="not all"><source id="fallback" srcset="/video/from-srcset.mp4 1x" type="video/mp4"><track id="track-en" kind="captions" srclang="en" src="/tracks/en.vtt">';
          player.load();
          player.currentTime = 1.25;

          const afterSecondInnerHtml = snapshot();

          player.src = '/video/direct-2.mp4';

          const restoredDirect = snapshot();

          document.getElementById('result').textContent = [
            before,
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
        "https://app.local/video/direct.mp4:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:2|https://app.local/video/rebuilt.mp4:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:3.5|https://app.local/video/leading.webm:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:4.5|https://app.local/video/bad.txt:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:4.5|https://app.local/video/from-srcset.mp4:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:1.25|https://app.local/video/direct-2.mp4:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:1.25",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_borrowed_object_surface_stays_live_across_mixed_rebuild_churn_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 2;

          const trackList = player.textTracks;
          const timeRanges = player.buffered;
          const trackProto = Object.create(TextTrackList.prototype);
          const rangeProto = Object.create(TimeRanges.prototype);
          trackProto.kindTag = function() { return 'tracks-proto'; };
          rangeProto.kindTag = function() { return 'ranges-proto'; };
          Object.setPrototypeOf(trackList, trackProto);
          Object.setPrototypeOf(timeRanges, rangeProto);

          Object.defineProperty(trackList, 'marker', {
            value: 'tracks-own',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(timeRanges, 'marker', {
            value: 'ranges-own',
            enumerable: true,
            configurable: true
          });

          const holder = Object.create({
            media: player,
            wrappers: {
              trackList,
              timeRanges
            }
          });
          const ownKeys = Reflect.ownKeys;
          const getDesc = Object.getOwnPropertyDescriptor;
          const assign = Object.assign;
          const item = holder.wrappers.trackList.item;
          const end = holder.wrappers.timeRanges.end;

          function snapshot() {
            return [
              holder.media.currentSrc,
              String(holder.wrappers.trackList === player.textTracks),
              String(holder.wrappers.timeRanges === player.buffered),
              String(Object.getPrototypeOf(holder.wrappers.trackList) === trackProto),
              String(Object.getPrototypeOf(holder.wrappers.timeRanges) === rangeProto),
              holder.wrappers.trackList.kindTag(),
              holder.wrappers.timeRanges.kindTag(),
              String(getDesc(holder.wrappers.trackList, 'marker').enumerable),
              String(getDesc(holder.wrappers.timeRanges, 'marker').enumerable),
              String(ownKeys(holder.wrappers.trackList).includes('marker')),
              String(ownKeys(holder.wrappers.timeRanges).includes('marker')),
              assign({}, holder.wrappers.trackList).marker,
              ({ ...holder.wrappers.trackList }).marker,
              assign({}, holder.wrappers.timeRanges).marker,
              ({ ...holder.wrappers.timeRanges }).marker,
              item.call(holder.wrappers.trackList, 0).id,
              String(end.call(holder.wrappers.timeRanges, 0))
            ].join(':');
          }

          const before = snapshot();

          player.removeAttribute('src');
          player.innerHTML = '<source id="bad" src="/video/bad.txt" type="text/plain"><source id="rebuilt" src="/video/rebuilt.mp4" type="video/mp4"><track id="track-en" kind="captions" srclang="en" src="/tracks/en.vtt">';
          player.currentTime = 3.5;

          const afterInnerHtml = snapshot();

          const domInserted = document.createElement('source');
          domInserted.id = 'dom-inserted';
          domInserted.type = 'video/webm';
          domInserted.src = '/video/dom-inserted.webm';
          player.insertBefore(domInserted, player.firstChild);
          player.currentTime = 4.5;

          const afterDomInsert = snapshot();

          player.removeChild(domInserted);
          player.insertAdjacentHTML('afterbegin', '<source id="string-leading" src="/video/string-leading.mp4" type="video/mp4">');

          const afterStringInsert = snapshot();

          document.getElementById('string-leading').removeAttribute('src');
          document.getElementById('bad').type = 'video/mp4';

          const afterToggle = snapshot();

          player.src = '/video/direct-2.mp4';

          const restoredDirect = snapshot();

          document.getElementById('result').textContent = [
            before,
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
        "https://app.local/video/direct.mp4:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:2|https://app.local/video/rebuilt.mp4:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:3.5|https://app.local/video/dom-inserted.webm:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:4.5|https://app.local/video/string-leading.mp4:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:4.5|https://app.local/video/bad.txt:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:4.5|https://app.local/video/direct-2.mp4:true:true:true:true:tracks-proto:ranges-proto:true:true:true:true:tracks-own:tracks-own:ranges-own:ranges-own:track-en:4.5",
    )?;
    Ok(())
}

#[test]
fn video_cached_media_wrapper_callables_stay_stable_across_direct_src_reset_and_load_churn_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 2;

          const tracks = player.textTracks;
          const ranges = player.buffered;
          const item = tracks.item;
          const values = tracks.values;
          const iterator = tracks[Symbol.iterator];
          const start = ranges.start;
          const end = ranges.end;
          const lengthGetter = Object.getOwnPropertyDescriptor(TimeRanges.prototype, 'length').get;

          function snapshot() {
            return [
              player.currentSrc,
              String(tracks === player.textTracks),
              String(ranges === player.buffered),
              String(item === tracks.item),
              String(values === tracks.values),
              String(iterator === tracks[Symbol.iterator]),
              String(start === ranges.start),
              String(end === ranges.end),
              item.call(tracks, 0).id,
              Array.from(values.call(tracks)).map(track => track.id).join(','),
              Array.from(iterator.call(tracks)).map(track => track.id).join(','),
              String(lengthGetter.call(ranges)),
              String(start.call(ranges, 0)),
              String(end.call(ranges, 0))
            ].join(':');
          }

          const before = snapshot();

          player.removeAttribute('src');
          player.load();
          player.currentTime = 3.5;

          const afterRemovingDirect = snapshot();

          player.src = '/video/direct-2.mp4';
          player.innerHTML = '<source id="bad" src="/video/bad.txt" type="text/plain"><source id="rebuilt" src="/video/rebuilt.mp4" type="video/mp4"><track id="track-en" kind="captions" srclang="en" src="/tracks/en.vtt">';
          player.load();
          player.currentTime = 4.5;

          const afterResetWhileDirect = snapshot();

          player.removeAttribute('src');
          player.load();
          player.currentTime = 5.5;

          const afterNestedRestore = snapshot();

          player.src = '/video/direct-3.mp4';
          player.load();
          player.currentTime = 6.5;

          const afterDirectAgain = snapshot();

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterResetWhileDirect,
            afterNestedRestore,
            afterDirectAgain
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/video/direct.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:2|https://app.local/video/primary.webm:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:3.5|https://app.local/video/direct-2.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:4.5|https://app.local/video/rebuilt.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:5.5|https://app.local/video/direct-3.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:6.5",
    )?;
    Ok(())
}

#[test]
fn video_media_wrapper_callable_alias_paths_stay_live_across_load_triggered_precedence_churn_work()
-> Result<()> {
    let html = r#"
        <video id='player' src='/video/direct.mp4'>
          <source id='bad' src='/video/bad.txt' type='text/plain'>
          <source id='primary' src='/video/primary.webm' type='video/webm'>
          <track id='track-en' kind='captions' srclang='en' src='/tracks/en.vtt'>
        </video>
        <p id='result'></p>
        <script>
          const player = document.getElementById('player');
          player.currentTime = 2;

          const alias = {
            wrappers: {
              tracks: player.textTracks,
              ranges: player.buffered
            }
          };

          const item = alias.wrappers['tracks'].item;
          const values = alias.wrappers.tracks['values'];
          const iterator = alias.wrappers['tracks'][Symbol.iterator];
          const start = alias.wrappers.ranges['start'];
          const end = alias.wrappers['ranges'].end;
          const lengthGetter = Object.getOwnPropertyDescriptor(TimeRanges.prototype, 'length').get;

          function borrowItem(receiver, index) {
            return Function.prototype.call.call(item, receiver, index);
          }

          function borrowValues(receiver) {
            return Function.prototype.apply.call(values, receiver, []);
          }

          function borrowIterator(receiver) {
            return Function.prototype.call.call(iterator, receiver);
          }

          function borrowStart(receiver, index) {
            return Function.prototype.call.call(start, receiver, index);
          }

          function borrowEnd(receiver, index) {
            return Function.prototype.apply.call(end, receiver, [index]);
          }

          function borrowLength(receiver) {
            return Function.prototype.call.call(lengthGetter, receiver);
          }

          function snapshot() {
            return [
              player.currentSrc,
              String(alias.wrappers.tracks === player.textTracks),
              String(alias.wrappers.ranges === player.buffered),
              String(item === alias.wrappers.tracks.item),
              String(values === alias.wrappers.tracks.values),
              String(iterator === alias.wrappers.tracks[Symbol.iterator]),
              String(start === alias.wrappers.ranges.start),
              String(end === alias.wrappers.ranges.end),
              borrowItem(alias.wrappers.tracks, 0).id,
              Array.from(borrowValues(alias.wrappers.tracks)).map((track) => track.id).join(','),
              Array.from(borrowIterator(alias.wrappers.tracks)).map((track) => track.id).join(','),
              String(borrowLength(alias.wrappers.ranges)),
              String(borrowStart(alias.wrappers.ranges, 0)),
              String(borrowEnd(alias.wrappers.ranges, 0))
            ].join(':');
          }

          player.load();
          const before = snapshot();

          player.removeAttribute('src');
          player.load();
          player.currentTime = 3.5;
          const afterRemovingDirect = snapshot();

          player.src = '/video/direct-2.mp4';
          player.load();
          player.currentTime = 4.5;
          const afterRestoringDirect = snapshot();

          player.innerHTML = '<source id="rebuilt-bad" src="/video/rebuilt-bad.txt" type="text/plain"><source id="rebuilt" src="/video/rebuilt.mp4" type="video/mp4"><track id="track-en" kind="captions" srclang="en" src="/tracks/en.vtt">';
          player.load();
          player.currentTime = 5.5;
          const afterNestedResetWhileDirect = snapshot();

          player.removeAttribute('src');
          player.load();
          player.currentTime = 6.5;
          const afterNestedRestore = snapshot();

          document.getElementById('rebuilt').removeAttribute('src');
          document.getElementById('rebuilt-bad').type = 'video/mp4';
          player.load();
          player.currentTime = 7.5;
          const afterReweightingNested = snapshot();

          player.src = '/video/direct-3.mp4';
          player.load();
          player.currentTime = 8.5;
          const afterDirectAgain = snapshot();

          document.getElementById('result').textContent = [
            before,
            afterRemovingDirect,
            afterRestoringDirect,
            afterNestedResetWhileDirect,
            afterNestedRestore,
            afterReweightingNested,
            afterDirectAgain
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/watch/index.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/video/direct.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:0|https://app.local/video/primary.webm:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:3.5|https://app.local/video/direct-2.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:4.5|https://app.local/video/direct-2.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:5.5|https://app.local/video/rebuilt.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:6.5|https://app.local/video/rebuilt-bad.txt:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:7.5|https://app.local/video/direct-3.mp4:true:true:true:true:true:true:true:track-en:track-en:track-en:1:0:8.5",
    )?;
    Ok(())
}
