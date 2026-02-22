use super::*;

#[test]
fn menu_has_implicit_list_role_and_menu_items_behave_like_unordered_list() -> Result<()> {
    let html = r#"
        <div class='news'>
          <a id='headline' href='/article'>NASA's Webb Delivers Deepest Infrared Image</a>
          <menu id='toolbar'>
            <li><button id='save'>Save for later</button></li>
            <li><button id='share'>Share this news</button></li>
          </menu>
        </div>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const toolbar = document.getElementById('toolbar');
            const saveItem = document.querySelector('#toolbar li:first-child');
            const shareItem = document.querySelector('#toolbar li:last-child');

            document.getElementById('result').textContent =
              toolbar.role + ':' +
              toolbar.tagName + ':' +
              toolbar.querySelectorAll('li').length + ':' +
              saveItem.role + ':' +
              shareItem.role + ':' +
              document.getElementById('save').textContent.trim() + ':' +
              document.getElementById('share').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "list:MENU:2:listitem:listitem:Save for later:Share this news",
    )?;
    Ok(())
}

#[test]
fn menu_role_override_and_compact_attribute_roundtrip_work() -> Result<()> {
    let html = r#"
        <menu id='commands' compact>
          <li>Copy</li>
          <li>Cut</li>
          <li>Paste</li>
        </menu>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const commands = document.getElementById('commands');
            const initial =
              commands.role + ':' +
              commands.getAttribute('compact') + ':' +
              commands.querySelectorAll('li').length;

            commands.role = 'toolbar';
            const assigned = commands.role + ':' + commands.getAttribute('role');

            commands.removeAttribute('role');
            const restored = commands.role + ':' + (commands.getAttribute('role') === null);

            commands.setAttribute('compact', '');
            const compactSet = commands.getAttribute('compact') === '';
            commands.removeAttribute('compact');
            const compactRemoved = commands.getAttribute('compact') === null;

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' +
              compactSet + ':' + compactRemoved;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "list:true:3|toolbar:toolbar|list:true|true:true",
    )?;
    Ok(())
}
