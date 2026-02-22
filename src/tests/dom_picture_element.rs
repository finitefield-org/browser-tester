use super::*;

#[test]
fn picture_selects_best_source_for_img_src_using_type_media_and_srcset() -> Result<()> {
    let html = r#"
        <picture id='art'>
          <source id='bad-type' srcset='/img/photo.avif' type='image/unsupported'>
          <source id='wide' srcset='/img/wide.webp 1x, /img/wide@2x.webp 2x' media='(width >= 700px)' type='image/webp'>
          <source id='narrow' srcset='/img/narrow.webp' media='(width < 700px)' type='image/webp'>
          <img id='fallback' src='/img/fallback.jpg' alt='Scenery'>
        </picture>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            window.innerWidth = 1024;
            window.innerHeight = 768;

            const picture = document.getElementById('art');
            const img = document.getElementById('fallback');
            const first =
              img.src + ':' +
              img.getAttribute('src') + ':' +
              picture.role + ':' +
              document.querySelectorAll('#art > source').length;

            window.innerWidth = 640;
            const second = img.src;

            document.getElementById('wide').setAttribute('type', 'image/unsupported');
            const third = img.src;

            document.getElementById('narrow').setAttribute('media', '(width >= 700px)');
            const fourth = img.src;

            document.getElementById('result').textContent =
              first + '|' + second + '|' + third + '|' + fourth;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/base/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/img/wide.webp:/img/fallback.jpg::3|https://app.local/img/narrow.webp|https://app.local/img/narrow.webp|https://app.local/img/fallback.jpg",
    )?;
    Ok(())
}

#[test]
fn picture_has_no_implicit_role_and_falls_back_to_img_when_no_source_matches() -> Result<()> {
    let html = r#"
        <picture id='icon'>
          <source id='legacy' src='/img/legacy.png' type='image/unsupported' media='(orientation: portrait)'>
          <img id='img' src='/img/default.png' alt='Logo'>
        </picture>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const picture = document.getElementById('icon');
            const img = document.getElementById('img');

            const before = img.src + ':' + picture.role;

            document.getElementById('legacy').setAttribute('type', 'image/png');
            const afterType = img.src;

            document.getElementById('legacy').removeAttribute('media');
            const afterMediaRemoved = img.src;

            picture.role = 'group';
            const assigned = picture.role + ':' + picture.getAttribute('role');
            picture.removeAttribute('role');
            const restored = picture.role + ':' + (picture.getAttribute('role') === null);

            document.getElementById('result').textContent =
              before + '|' + afterType + '|' + afterMediaRemoved + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/base/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/img/default.png:|https://app.local/img/default.png|https://app.local/img/legacy.png|group:group|:true",
    )?;
    Ok(())
}
