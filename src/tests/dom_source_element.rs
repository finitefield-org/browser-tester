use super::*;

#[test]
fn source_in_picture_reflects_attributes_and_drives_picture_selection() -> Result<()> {
    let html = r#"
        <picture id='art'>
          <source
            id='hero'
            srcset='/img/hero.webp 1x, /img/hero@2x.webp 2x'
            sizes='(width >= 900px) 900px, 100vw'
            media='(width >= 700px)'
            type='image/webp'
            width='1200'
            height='700'>
          <img id='fallback' src='/img/fallback.jpg' alt='Artwork'>
        </picture>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            window.innerWidth = 960;

            const hero = document.getElementById('hero');
            const fallback = document.getElementById('fallback');

            const first =
              hero.role + ':' +
              hero.tagName + ':' +
              hero.src + ':' +
              hero.type + ':' +
              hero.media + ':' +
              hero.sizes.includes('100vw') + ':' +
              hero.srcset.includes('hero@2x.webp') + ':' +
              hero.width + 'x' + hero.height + ':' +
              fallback.src;

            hero.media = '(width >= 1200px)';
            const second = fallback.src;

            hero.media = '(width >= 700px)';
            hero.srcset = '/img/next.webp 1x';
            hero.sizes = '50vw';
            hero.width = 800;
            hero.height = 500;
            hero.src = '/img/explicit.webp';

            const third =
              hero.getAttribute('srcset') + ':' +
              hero.sizes + ':' +
              hero.getAttribute('media') + ':' +
              hero.width + 'x' + hero.height + ':' +
              hero.getAttribute('width') + ':' +
              hero.getAttribute('height') + ':' +
              hero.src + ':' +
              fallback.src;

            document.getElementById('result').textContent =
              first + '|' + second + '|' + third;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/base/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":SOURCE::image/webp:(width >= 700px):true:true:1200x700:https://app.local/img/hero.webp|https://app.local/img/fallback.jpg|/img/next.webp 1x:50vw:(width >= 700px):800x500:800:500:https://app.local/img/explicit.webp:https://app.local/img/next.webp",
    )?;
    Ok(())
}

#[test]
fn source_in_video_exposes_src_type_media_and_role_roundtrip() -> Result<()> {
    let html = r#"
        <video id='player' controls>
          <source id='primary' src='/video/flower.webm' type='video/webm' media='(width >= 800px)'>
          <source id='backup' src='/video/flower.mp4' type='video/mp4'>
          fallback text
        </video>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const primary = document.getElementById('primary');
            const backup = document.getElementById('backup');
            const player = document.getElementById('player');

            const initial =
              primary.role + ':' +
              primary.src + ':' +
              primary.type + ':' +
              primary.media + ':' +
              primary.childElementCount + ':' +
              backup.src + ':' +
              player.src;

            primary.type = 'video/quicktime';
            primary.src = '/video/new.mov';

            const updated =
              primary.getAttribute('type') + ':' +
              primary.src + ':' +
              primary.getAttribute('src') + ':' +
              player.src;

            primary.role = 'none';
            const assigned = primary.role + ':' + primary.getAttribute('role');
            primary.removeAttribute('role');
            const restored = primary.role + ':' + (primary.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://media.local/watch/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":https://media.local/video/flower.webm:video/webm:(width >= 800px):0:https://media.local/video/flower.mp4:https://media.local/video/flower.webm|video/quicktime:https://media.local/video/new.mov:/video/new.mov:https://media.local/video/new.mov|none:none|:true",
    )?;
    Ok(())
}

#[test]
fn source_reflective_own_property_surface_and_object_copy_work() -> Result<()> {
    let html = r#"
        <picture>
          <source
            id='hero'
            src='/img/explicit.webp'
            srcset='/img/hero.webp 1x, /img/hero@2x.webp 2x'
            sizes='50vw'>
          <img src='/img/fallback.jpg' alt='Artwork'>
        </picture>
        <p id='result'></p>
        <script>
          const hero = document.getElementById('hero');
          const beforeAssigned = Object.assign({}, hero);
          const beforeSpread = { ...hero };

          const before = [
            hero.src,
            hero.srcset,
            hero.sizes,
            String(Object.hasOwn(hero, 'src')),
            String(Object.hasOwn(hero, 'srcset')),
            String(Object.hasOwn(hero, 'sizes')),
            String(Object.getOwnPropertyDescriptor(hero, 'src') === undefined),
            String(Object.getOwnPropertyNames(hero).includes('srcset')),
            String(Reflect.ownKeys(hero).includes('sizes')),
            String('src' in beforeAssigned),
            String('srcset' in beforeSpread)
          ].join(':');

          Object.defineProperty(hero, 'src', {
            value: 'shadow-src',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(hero, 'srcset', {
            value: 'shadow-srcset',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(hero, 'sizes', {
            value: 'shadow-sizes',
            writable: true,
            enumerable: true,
            configurable: true
          });
          hero.extra = 'expando';

          const shadowAssigned = Object.assign({}, hero);
          const shadowSpread = { ...hero };

          const shadowed = [
            hero.src,
            hero.srcset,
            hero.sizes,
            String(Object.keys(hero).sort().join(',') === 'extra,sizes,src,srcset'),
            shadowAssigned.src,
            shadowAssigned.srcset,
            shadowAssigned.sizes,
            shadowAssigned.extra,
            shadowSpread.src,
            shadowSpread.srcset,
            shadowSpread.sizes,
            shadowSpread.extra
          ].join(':');

          delete hero.src;
          delete hero.srcset;
          delete hero.sizes;

          const restoredAssigned = Object.assign({}, hero);
          const restoredSpread = { ...hero };

          const restored = [
            hero.src,
            hero.srcset,
            hero.sizes,
            String(Object.hasOwn(hero, 'src')),
            String(Object.hasOwn(hero, 'srcset')),
            String(Object.hasOwn(hero, 'sizes')),
            restoredAssigned.extra,
            String('src' in restoredAssigned),
            String('srcset' in restoredAssigned),
            String('sizes' in restoredAssigned),
            restoredSpread.extra,
            String('src' in restoredSpread),
            String('srcset' in restoredSpread),
            String('sizes' in restoredSpread)
          ].join(':');

          document.getElementById('result').textContent = [
            before,
            shadowed,
            restored
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/base/page.html", html)?;
    h.assert_text(
        "#result",
        "https://app.local/img/explicit.webp:/img/hero.webp 1x, /img/hero@2x.webp 2x:50vw:false:false:false:true:false:false:false:false|shadow-src:shadow-srcset:shadow-sizes:true:shadow-src:shadow-srcset:shadow-sizes:expando:shadow-src:shadow-srcset:shadow-sizes:expando|https://app.local/img/explicit.webp:/img/hero.webp 1x, /img/hero@2x.webp 2x:50vw:false:false:false:expando:false:false:false:expando:false:false:false",
    )?;
    Ok(())
}

#[test]
fn source_type_and_media_shadow_define_property_delete_and_generic_parity_work() -> Result<()> {
    let html = r#"
        <picture>
          <source
            id='hero'
            src='/img/hero.webp'
            type='image/webp'
            media='(width >= 700px)'>
          <img src='/img/fallback.jpg' alt='Artwork'>
        </picture>
        <p id='result'></p>
        <script>
          const hero = document.getElementById('hero');

          const before = [
            hero.type,
            hero.media,
            hero.getAttribute('type'),
            hero.getAttribute('media')
          ].join(':');

          Object.defineProperty(hero, 'type', {
            value: 'shadow-type',
            writable: true,
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(hero, 'media', {
            value: 'shadow-media',
            writable: true,
            enumerable: true,
            configurable: true
          });

          hero.type = 'set-type';
          hero.media = 'set-media';

          const shadowed = [
            hero.type,
            hero.media,
            hero.getAttribute('type'),
            hero.getAttribute('media'),
            String(Object.keys(hero).sort().join(',') === 'media,type')
          ].join(':');

          delete hero.type;
          delete hero.media;

          const restored = [
            hero.type,
            hero.media,
            String(Object.hasOwn(hero, 'type')),
            String(Object.hasOwn(hero, 'media'))
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
        "image/webp:(width >= 700px):image/webp:(width >= 700px)|set-type:set-media:image/webp:(width >= 700px):true|image/webp:(width >= 700px):false:false",
    )?;
    Ok(())
}
