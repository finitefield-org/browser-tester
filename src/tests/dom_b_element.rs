use super::*;

#[test]
fn b_implicit_generic_role_and_phrasing_usage_work() -> Result<()> {
    let html = r#"
        <p id='courses'>
          The two most popular science courses are
          <b id='chem' class='term'>chemistry</b> and
          <b id='phys' class='term'>physics</b>.
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const chem = document.getElementById('chem');
            const phys = document.getElementById('phys');
            document.getElementById('result').textContent =
              chem.role + ':' +
              phys.role + ':' +
              document.querySelectorAll('b.term').length + ':' +
              phys.className + ':' +
              chem.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic:generic:2:term:B")?;
    Ok(())
}

#[test]
fn b_role_attribute_overrides_and_remove_restores_implicit_role() -> Result<()> {
    let html = r#"
        <p>
          <b id='keyword'>HTML</b> document keyword.
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const keyword = document.getElementById('keyword');
            const initial = keyword.role;
            keyword.role = 'strong';
            const assigned = keyword.role + ':' + keyword.getAttribute('role');
            keyword.removeAttribute('role');
            const restored = keyword.role + ':' + (keyword.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic|strong:strong|generic:true")?;
    Ok(())
}
