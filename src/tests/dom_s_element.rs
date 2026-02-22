use super::*;

#[test]
fn s_has_implicit_deletion_role_and_struck_text_remains_queryable() -> Result<()> {
    let html = r#"
        <p><s id='notice'>There will be tickets at the box office tonight.</s></p>
        <p id='status'>SOLD OUT!</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const notice = document.getElementById('notice');
            const status = document.getElementById('status');
            document.getElementById('result').textContent =
              notice.role + ':' +
              notice.tagName + ':' +
              notice.textContent.includes('box office') + ':' +
              document.querySelectorAll('s').length + ':' +
              status.textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "deletion:S:true:1:SOLD OUT!")?;
    Ok(())
}

#[test]
fn s_role_override_and_remove_restore_implicit_deletion_role() -> Result<()> {
    let html = r#"
        <p>Today's special: <s id='item'>Salmon</s> SOLD OUT</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const item = document.getElementById('item');
            const initial = item.role;
            item.role = 'note';
            const assigned = item.role + ':' + item.getAttribute('role');
            item.removeAttribute('role');
            const restored = item.role + ':' + (item.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + item.textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "deletion|note:note|deletion:true|Salmon")?;
    Ok(())
}
