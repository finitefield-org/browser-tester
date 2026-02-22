use super::*;

#[test]
fn em_implicit_emphasis_role_and_nested_emphasis_work() -> Result<()> {
    let html = r#"
        <p id='line'>
          This is <em id='outer'>not <em id='inner'>really</em></em> a drill!
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const outer = document.getElementById('outer');
            const inner = document.getElementById('inner');
            document.getElementById('result').textContent =
              outer.role + ':' +
              inner.role + ':' +
              document.querySelectorAll('#line em').length + '|' +
              outer.textContent.trim() + '|' +
              inner.textContent.trim() + '|' +
              outer.tagName + ':' + inner.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "emphasis:emphasis:2|not really|really|EM:EM")?;
    Ok(())
}

#[test]
fn em_role_override_and_restore_roundtrip_work() -> Result<()> {
    let html = r#"
        <p>We <em id='target'>had</em> to do something about it.</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const initial = target.role + ':' + target.textContent;
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
    h.assert_text("#result", "emphasis:had|note:note|emphasis:true")?;
    Ok(())
}
