use super::*;

#[test]
fn header_has_banner_role_when_scoped_to_page_body() -> Result<()> {
    let html = r#"
        <header id='site-header'>
          <a class='logo' href='#'>Cute Puppies Express!</a>
        </header>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const header = document.getElementById('site-header');
            const initial =
              header.role + ':' +
              header.tagName + ':' +
              header.querySelector('.logo').textContent.trim();

            header.role = 'none';
            const assigned = header.role + ':' + header.getAttribute('role');
            header.removeAttribute('role');
            const restored = header.role + ':' + (header.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "banner:HEADER:Cute Puppies Express!|none:none|banner:true",
    )?;
    Ok(())
}

#[test]
fn header_is_generic_inside_sectioning_or_landmark_ancestors() -> Result<()> {
    let html = r#"
        <article id='post'>
          <header id='article-header'>
            <h1>Beagles</h1>
            <time>08.12.2014</time>
          </header>
          <p>I love beagles.</p>
        </article>
        <div id='region' role='region'>
          <header id='region-header'>
            <h2>Related content</h2>
          </header>
        </div>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const articleHeader = document.getElementById('article-header');
            const regionHeader = document.getElementById('region-header');
            document.getElementById('result').textContent =
              articleHeader.role + ':' +
              regionHeader.role + ':' +
              articleHeader.querySelector('h1').textContent.trim() + ':' +
              regionHeader.querySelector('h2').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic:generic:Beagles:Related content")?;
    Ok(())
}
