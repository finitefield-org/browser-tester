use super::*;

#[test]
fn sup_has_implicit_superscript_role_and_supports_exponent_content() -> Result<()> {
    let html = r#"
        <p>
          <var>E</var>=<var>m</var><var>c</var><sup id='exp'>2</sup>
        </p>
        <p id='ordinals'>
          <span>English: 5<sup>th</sup></span>
          <span>French: 5<sup id='fr'>ème</sup></span>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const exp = document.getElementById('exp');
            const fr = document.getElementById('fr');
            document.getElementById('result').textContent =
              exp.role + ':' +
              fr.role + ':' +
              exp.tagName + ':' +
              document.querySelectorAll('sup').length + ':' +
              exp.textContent.trim() + ':' +
              fr.textContent.trim() + ':' +
              document.getElementById('ordinals').textContent.replace(/\s+/g, ' ').trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "superscript:superscript:SUP:3:2:ème:English: 5th French: 5ème",
    )?;
    Ok(())
}

#[test]
fn sup_role_override_and_remove_restores_implicit_superscript_role() -> Result<()> {
    let html = r#"
        <p>Robert a présenté son rapport à M<sup id='label'>lle</sup> Bernard.</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const label = document.getElementById('label');
            const initial = label.role + ':' + label.textContent.trim();

            label.role = 'note';
            const assigned = label.role + ':' + label.getAttribute('role');

            label.removeAttribute('role');
            const restored = label.role + ':' + (label.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "superscript:lle|note:note|superscript:true")?;
    Ok(())
}
