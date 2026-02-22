use super::*;

#[test]
fn main_has_implicit_role_and_can_be_skip_navigation_target() -> Result<()> {
    let html = r#"
        <header>Gecko facts</header>
        <a id='skip' href='#main-content'>Skip to main content</a>

        <main id='main-content'>
          <p>
            Geckos are a group of usually small, usually nocturnal lizards.
          </p>
          <article>
            <h2>Adhesive toe pads</h2>
            <p>Many species can climb walls and windows.</p>
          </article>
        </main>

        <button id='run' type='button'>run</button>
        <p id='result'></p>

        <script>
          document.getElementById('run').addEventListener('click', () => {
            const main = document.getElementById('main-content');
            const initial =
              main.role + ':' +
              main.tagName + ':' +
              main.querySelectorAll('article').length + ':' +
              document.getElementById('skip').textContent.trim();

            main.setAttribute('hidden', '');
            const hiddenState = main.hidden + ':' + main.hasAttribute('hidden');

            main.hidden = false;
            const shownState = main.hidden + ':' + (main.getAttribute('hidden') === null);

            document.getElementById('result').textContent =
              initial + '|' + hiddenState + '|' + shownState;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "main:MAIN:1:Skip to main content|true:true|false:true",
    )?;
    Ok(())
}

#[test]
fn main_scopes_header_footer_roles_and_role_override_roundtrips() -> Result<()> {
    let html = r#"
        <main id='primary'>
          <header id='section-header'>
            <h1>Apples</h1>
          </header>

          <p>The apple is the pomaceous fruit of the apple tree.</p>

          <footer id='section-footer'>
            <small>Page notes</small>
          </footer>
        </main>

        <button id='run' type='button'>run</button>
        <p id='result'></p>

        <script>
          document.getElementById('run').addEventListener('click', () => {
            const main = document.getElementById('primary');
            const sectionHeader = document.getElementById('section-header');
            const sectionFooter = document.getElementById('section-footer');

            const initial =
              main.role + ':' +
              sectionHeader.role + ':' +
              sectionFooter.role + ':' +
              sectionHeader.querySelector('h1').textContent.trim();

            main.role = 'presentation';
            const assigned = main.role + ':' + main.getAttribute('role');

            main.removeAttribute('role');
            const restored = main.role + ':' + (main.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "main:generic:generic:Apples|presentation:presentation|main:true",
    )?;
    Ok(())
}
