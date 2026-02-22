use super::*;

#[test]
fn footer_has_contentinfo_role_when_scoped_to_page_body() -> Result<()> {
    let html = r#"
        <h3>FIFA World Cup top goalscorers</h3>
        <ol>
          <li>Miroslav Klose, 16</li>
          <li>Ronaldo Nazario, 15</li>
          <li>Gerd Muller, 14</li>
        </ol>
        <footer id='site-footer'>
          <small>Copyright 2023 Football History Archives.</small>
        </footer>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const footer = document.getElementById('site-footer');
            const initial =
              footer.role + ':' +
              footer.tagName + ':' +
              footer.querySelector('small').textContent.trim();

            footer.role = 'none';
            const assigned = footer.role + ':' + footer.getAttribute('role');
            footer.removeAttribute('role');
            const restored = footer.role + ':' + (footer.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "contentinfo:FOOTER:Copyright 2023 Football History Archives.|none:none|contentinfo:true",
    )?;
    Ok(())
}

#[test]
fn footer_is_generic_inside_sectioning_or_landmark_ancestors() -> Result<()> {
    let html = r#"
        <article id='post'>
          <h1>How to be a wizard</h1>
          <footer id='article-footer'>
            <p>Copyright 2018 Gandalf</p>
          </footer>
        </article>
        <div id='region' role='region'>
          <footer id='region-footer'>
            <p>Related links</p>
          </footer>
        </div>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const articleFooter = document.getElementById('article-footer');
            const regionFooter = document.getElementById('region-footer');
            document.getElementById('result').textContent =
              articleFooter.role + ':' +
              regionFooter.role + ':' +
              articleFooter.querySelector('p').textContent.trim() + ':' +
              regionFooter.querySelector('p').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "generic:generic:Copyright 2018 Gandalf:Related links",
    )?;
    Ok(())
}
