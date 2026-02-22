use super::*;

#[test]
fn hr_has_implicit_separator_role_and_void_element_shape() -> Result<()> {
    let html = r#"
        <p>First paragraph</p>
        <hr id='divider'>
        <p>Second paragraph</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const hr = document.getElementById('divider');
            const initial =
              hr.role + ':' +
              hr.tagName + ':' +
              document.querySelectorAll('hr').length + ':' +
              hr.children.length;

            hr.role = 'none';
            const assigned = hr.role + ':' + hr.getAttribute('role');
            hr.removeAttribute('role');
            const restored = hr.role + ':' + (hr.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "separator:HR:1:0|none:none|separator:true")?;
    Ok(())
}

#[test]
fn hr_deprecated_attributes_roundtrip_work() -> Result<()> {
    let html = r#"
        <hr id='divider' />
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const hr = document.getElementById('divider');
            hr.align = 'right';
            hr.setAttribute('color', '#ff0000');
            hr.toggleAttribute('noshade', true);
            hr.setAttribute('size', '6');
            hr.setAttribute('width', '80%');

            const first =
              hr.align + ':' +
              hr.getAttribute('align') + ':' +
              hr.getAttribute('color') + ':' +
              hr.hasAttribute('noshade') + ':' +
              hr.getAttribute('size') + ':' +
              hr.getAttribute('width');

            hr.removeAttribute('noshade');
            hr.setAttribute('width', '240');
            const second =
              hr.hasAttribute('noshade') + ':' + hr.getAttribute('width');

            document.getElementById('result').textContent = first + '|' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "right:right:#ff0000:true:6:80%|false:240")?;
    Ok(())
}
