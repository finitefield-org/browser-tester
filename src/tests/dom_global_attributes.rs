use super::*;

#[test]
fn global_attributes_reflect_on_custom_elements() -> Result<()> {
    let html = r#"
        <foo id='box'></foo>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');

            box.accessKey = 'k';
            box.autocapitalize = 'words';
            box.autocorrect = 'off';
            box.contentEditable = 'plaintext-only';
            box.draggable = true;
            box.enterKeyHint = 'search';
            box.inert = true;
            box.inputMode = 'numeric';
            box.nonce = 'abc123';
            box.popover = 'manual';
            box.spellcheck = false;
            box.tabIndex = 7;
            box.translate = false;

            box.className = 'a b';
            box.lang = 'ja';
            box.dir = 'rtl';
            box.role = 'note';
            box.slot = 'hint';
            box.title = 'tip';
            box.dataset.state = 'ready';

            document.getElementById('result').textContent =
              box.accessKey + ':' +
              box.getAttribute('accesskey') + ':' +
              box.autocapitalize + ':' +
              box.autocorrect + ':' +
              box.contentEditable + ':' +
              box.draggable + ':' +
              box.inert + ':' +
              box.enterKeyHint + ':' +
              box.inputMode + ':' +
              box.nonce + ':' +
              box.popover + ':' +
              box.spellcheck + ':' +
              box.tabIndex + ':' +
              box.translate + ':' +
              box.dataset.state + ':' +
              box.role + ':' +
              box.slot + ':' +
              box.title + ':' +
              box.lang + ':' +
              box.dir + ':' +
              box.className;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "k:k:words:off:plaintext-only:true:true:search:numeric:abc123:manual:false:7:false:ready:note:hint:tip:ja:rtl:a b",
    )?;
    Ok(())
}

#[test]
fn hidden_global_attribute_works_on_non_standard_elements() -> Result<()> {
    let html = r#"
        <foo id='panel' hidden>content</foo>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const panel = document.getElementById('panel');
            const initial =
              panel.hidden + ':' +
              panel.checkVisibility() + ':' +
              panel.hasAttribute('hidden');

            panel.hidden = false;
            const shown =
              panel.hidden + ':' +
              panel.checkVisibility() + ':' +
              panel.hasAttribute('hidden');

            panel.hidden = true;
            const hiddenAgain =
              panel.hidden + ':' +
              panel.checkVisibility() + ':' +
              panel.hasAttribute('hidden');

            document.getElementById('result').textContent =
              initial + '|' + shown + '|' + hiddenAgain;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:false:true|false:true:false|true:false:true",
    )?;
    Ok(())
}

#[test]
fn global_event_handler_properties_work_on_custom_elements() -> Result<()> {
    let html = r#"
        <foo id='box'></foo>
        <button id='snapshot' type='button'>snapshot</button>
        <button id='clear' type='button'>clear</button>
        <p id='result'></p>
        <script>
          const box = document.getElementById('box');
          let log = '';

          box.onclick = () => {
            log += 'C';
          };
          box.onwheel = () => {
            log += 'W';
          };

          document.getElementById('snapshot').addEventListener('click', () => {
            document.getElementById('result').textContent =
              log + ':' +
              (box.onclick ? 'set' : 'unset') + ':' +
              (box.onwheel ? 'set' : 'unset');
          });

          document.getElementById('clear').addEventListener('click', () => {
            box.onclick = null;
            box.onwheel = null;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("#box", "click")?;
    h.dispatch("#box", "wheel")?;
    h.click("#snapshot")?;
    h.assert_text("#result", "CW:set:set")?;

    h.click("#clear")?;
    h.dispatch("#box", "click")?;
    h.dispatch("#box", "wheel")?;
    h.click("#snapshot")?;
    h.assert_text("#result", "CW:unset:unset")?;
    Ok(())
}
