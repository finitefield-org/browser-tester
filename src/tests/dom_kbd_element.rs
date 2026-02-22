use super::*;

#[test]
fn kbd_nested_input_and_echo_patterns_render_semantics() -> Result<()> {
    let html = r#"
        <p>
          To create a new document, press
          <kbd id='combo'><kbd id='inner-ctrl'>Ctrl</kbd>+<kbd id='inner-n'>N</kbd></kbd>.
        </p>
        <blockquote>
          <samp><kbd id='echoed'>custom-git add file.cpp</kbd></samp>
        </blockquote>
        <p>
          Choose
          <kbd id='menu'><kbd><samp>File</samp></kbd>-><kbd><samp>New</samp></kbd></kbd>.
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const innerCtrl = document.getElementById('inner-ctrl');
            const combo = document.getElementById('combo');
            const echoed = document.getElementById('echoed');
            const menu = document.getElementById('menu');
            document.getElementById('result').textContent =
              innerCtrl.role + ':' +
              combo.role + ':' +
              echoed.role + '|' +
              combo.textContent.replace(/\s+/g, '').trim() + '|' +
              menu.textContent.replace(/\s+/g, '').trim() + '|' +
              document.querySelectorAll('kbd').length + ':' +
              document.querySelectorAll('samp kbd').length + ':' +
              document.querySelectorAll('kbd samp').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "::|Ctrl+N|File->New|7:1:2")?;
    Ok(())
}

#[test]
fn kbd_role_attribute_overrides_and_remove_restores_empty_implicit_role() -> Result<()> {
    let html = r#"
        <p>Use the command <kbd id='cmd'>help my-command</kbd>.</p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const cmd = document.getElementById('cmd');
            const initial = cmd.role + ':' + cmd.tagName + ':' + cmd.textContent;
            cmd.role = 'note';
            const assigned = cmd.role + ':' + cmd.getAttribute('role');
            cmd.removeAttribute('role');
            const restored = cmd.role + ':' + (cmd.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":KBD:help my-command|note:note|:true")?;
    Ok(())
}
