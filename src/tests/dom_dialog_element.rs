use super::*;

#[test]
fn dialog_implicit_role_and_role_assignment_roundtrip() -> Result<()> {
    let html = r#"
        <dialog id='target'>Hello</dialog>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const initial = target.role;
            target.role = 'alertdialog';
            const assigned = target.role + ':' + target.getAttribute('role');
            target.removeAttribute('role');
            const restored = target.role + ':' + (target.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + target.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "dialog|alertdialog:alertdialog|dialog:true|DIALOG",
    )?;
    Ok(())
}

#[test]
fn dialog_methods_and_closedby_returnvalue_roundtrip_work() -> Result<()> {
    let html = r#"
        <dialog id='target' closedby='none'></dialog>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const before =
              target.open + ':' + target.closedBy + ':' + target.returnValue;

            target.showModal();
            const afterShow =
              target.open + ':' + target.hasAttribute('open');

            target.returnValue = 'seed';
            target.requestClose('next');
            const afterRequest =
              target.open + ':' + target.returnValue;

            target.closedBy = 'any';
            target.close('done');
            const afterClose =
              target.open + ':' +
              target.returnValue + ':' +
              target.closedBy + ':' +
              target.getAttribute('closedby');

            document.getElementById('result').textContent =
              before + '|' + afterShow + '|' + afterRequest + '|' + afterClose;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:none:|true:true|false:next|false:done:any:any",
    )?;
    Ok(())
}

#[test]
fn dialog_beforetoggle_prevent_default_blocks_show_and_close_default_actions_work() -> Result<()> {
    let html = r#"
        <dialog id='target'></dialog>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const log = [];
            let blockOpen = true;
            let blockClose = true;

            target.addEventListener('beforetoggle', (event) => {
              log.push([
                'before',
                event.oldState,
                event.newState,
                String(event.cancelable),
                String(event.bubbles)
              ].join(':'));
              if ((blockOpen && event.newState === 'open') ||
                  (blockClose && event.newState === 'closed')) {
                log.push('prevent:' + event.newState);
                event.preventDefault();
              }
            });
            target.addEventListener('toggle', (event) => {
              log.push('toggle:' + event.newState);
            });
            target.addEventListener('close', () => {
              log.push('close');
            });

            target.showModal();
            const afterBlockedOpen = String(target.open);

            blockOpen = false;
            target.showModal();
            const afterAllowedOpen = String(target.open);

            target.close('blocked');
            const afterBlockedClose = String(target.open);

            blockClose = false;
            target.close('done');
            const afterAllowedClose = String(target.open);

            document.getElementById('result').textContent = [
              afterBlockedOpen,
              afterAllowedOpen,
              afterBlockedClose,
              afterAllowedClose,
              log.join('|')
            ].join(';');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false;true;true;false;before:closed:open:true:false|prevent:open|before:closed:open:true:false|toggle:open|before:open:closed:true:false|prevent:closed|before:open:closed:true:false|toggle:closed|close",
    )?;
    Ok(())
}
