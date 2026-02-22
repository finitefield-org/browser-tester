use super::*;

#[test]
fn bdi_implicit_generic_role_and_default_dir_auto_work() -> Result<()> {
    let html = r#"
        <div id='wrapper' dir='rtl'>
          <p><bdi id='name' class='name'>الرجل القوي إيان</bdi>: 4th place</p>
          <span id='plain'>plain</span>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const name = document.getElementById('name');
            const plain = document.getElementById('plain');
            document.getElementById('result').textContent =
              name.role + ':' +
              name.dir + ':' +
              (name.getAttribute('dir') === null) + ':' +
              plain.dir + ':' +
              document.querySelectorAll('bdi.name').length + ':' +
              name.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "generic:auto:true::1:BDI")?;
    Ok(())
}

#[test]
fn bdi_dir_and_role_assignments_override_and_restore_defaults() -> Result<()> {
    let html = r#"
        <p><bdi id='item'>Evil Steven</bdi> - 1st place</p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const item = document.getElementById('item');

            const initialDir = item.dir;
            item.dir = 'ltr';
            const assignedDir = item.dir + ':' + item.getAttribute('dir');
            item.removeAttribute('dir');
            const restoredDir = item.dir + ':' + (item.getAttribute('dir') === null);

            const initialRole = item.role;
            item.role = 'note';
            const assignedRole = item.role + ':' + item.getAttribute('role');
            item.removeAttribute('role');
            const restoredRole = item.role + ':' + (item.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initialDir + '|' +
              assignedDir + '|' +
              restoredDir + '|' +
              initialRole + '|' +
              assignedRole + '|' +
              restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "auto|ltr:ltr|auto:true|generic|note:note|generic:true",
    )?;
    Ok(())
}
