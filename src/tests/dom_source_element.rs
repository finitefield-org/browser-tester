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
