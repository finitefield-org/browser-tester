use super::*;

#[test]
fn section_implicit_role_depends_on_accessible_name() -> Result<()> {
    let html = r#"
        <h1>Choosing an Apple</h1>

        <section id='intro'>
          <h2>Introduction</h2>
          <p>This document provides a guide for choosing an Apple.</p>
        </section>

        <section id='criteria' aria-label='Criteria'>
          <h2>Criteria</h2>
          <p>Size, color, firmness, sweetness, and tartness.</p>
        </section>

        <h2 id='named-heading'>Filter controls</h2>
        <section id='named' aria-labelledby='named-heading'>
          <button type='button'>Apply filter</button>
        </section>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const intro = document.getElementById('intro');
            const criteria = document.getElementById('criteria');
            const named = document.getElementById('named');

            document.getElementById('result').textContent =
              intro.role + ':' +
              criteria.role + ':' +
              named.role + ':' +
              intro.querySelector('h2').textContent.trim() + ':' +
              document.querySelectorAll('section').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic:region:region:Introduction:3")?;
    Ok(())
}

#[test]
fn section_role_override_and_name_source_roundtrip_work() -> Result<()> {
    let html = r#"
        <section id='panel' aria-label='Cars filters'>
          <h3 id='panel-heading'>Filter results</h3>
          <label for='brand'>Brand</label>
          <input id='brand' type='search' value='Mazda' />
        </section>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const panel = document.getElementById('panel');

            const initial = panel.role + ':' + panel.getAttribute('aria-label');

            panel.role = 'presentation';
            const assigned = panel.role + ':' + panel.getAttribute('role');

            panel.removeAttribute('role');
            const restored = panel.role + ':' + (panel.getAttribute('role') === null);

            panel.removeAttribute('aria-label');
            const unnamed = panel.role + ':' + (panel.getAttribute('aria-label') === null);

            panel.setAttribute('aria-labelledby', 'panel-heading');
            const labelled = panel.role + ':' + panel.getAttribute('aria-labelledby');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + unnamed + '|' + labelled;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "region:Cars filters|presentation:presentation|region:true|generic:true|region:panel-heading",
    )?;
    Ok(())
}
