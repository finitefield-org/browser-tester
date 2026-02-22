use super::*;

#[test]
fn strong_has_implicit_strong_role_and_importance_usage_work() -> Result<()> {
    let html = r#"
        <p id='warning'>
          Before proceeding,
          <strong id='goggles'>make sure you put on your safety goggles</strong>.
        </p>
        <p>
          <strong id='label'>Important:</strong>
          Add plenty of butter.
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const goggles = document.getElementById('goggles');
            const label = document.getElementById('label');

            document.getElementById('result').textContent =
              goggles.role + ':' +
              label.role + ':' +
              goggles.tagName + ':' +
              document.querySelectorAll('strong').length + ':' +
              goggles.textContent.includes('safety goggles') + ':' +
              label.textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "strong:strong:STRONG:2:true:Important:")?;
    Ok(())
}

#[test]
fn strong_role_override_and_remove_restores_implicit_strong_role() -> Result<()> {
    let html = r#"
        <p>
          Rule:
          <strong id='target'>never feed him after midnight</strong>.
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const initial = target.role + ':' + target.textContent.trim();

            target.role = 'note';
            const assigned = target.role + ':' + target.getAttribute('role');

            target.removeAttribute('role');
            const restored = target.role + ':' + (target.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "strong:never feed him after midnight|note:note|strong:true",
    )?;
    Ok(())
}
