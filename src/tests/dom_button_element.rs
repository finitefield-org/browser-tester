use super::*;

#[test]
fn button_implicit_role_and_role_assignment_roundtrip() -> Result<()> {
    let html = r#"
        <button id='target' type='button'>Add to <b>favorites</b></button>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const initial = target.role;
            target.role = 'switch';
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
    h.assert_text("#result", "button|switch:switch|button:true|BUTTON")?;
    Ok(())
}

#[test]
fn button_type_default_invalid_and_reset_submit_behavior_work() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' value='seed'>
          <button id='missing'>missing</button>
          <button id='invalid' type='invalid'>invalid</button>
          <button id='empty' type=''>empty</button>
          <button id='plain' type='button'>plain</button>
          <button id='reset' type='reset'>reset</button>
        </form>
        <button id='report' type='button'>report</button>
        <p id='result'></p>
        <script>
          let submits = 0;
          let resets = 0;
          let plainClicks = 0;
          const form = document.getElementById('f');
          form.addEventListener('submit', (event) => {
            submits++;
            event.preventDefault();
          });
          form.addEventListener('reset', () => {
            resets++;
          });
          document.getElementById('plain').addEventListener('click', () => {
            plainClicks++;
          });
          document.getElementById('report').addEventListener('click', () => {
            const missing = document.getElementById('missing');
            const invalid = document.getElementById('invalid');
            const empty = document.getElementById('empty');
            const plain = document.getElementById('plain');
            const reset = document.getElementById('reset');
            document.getElementById('result').textContent =
              submits + ':' + resets + ':' + plainClicks + '|' +
              missing.type + ',' + invalid.type + ',' + empty.type + ',' + plain.type + ',' + reset.type + '|' +
              document.getElementById('name').value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "changed")?;
    h.click("#missing")?;
    h.click("#invalid")?;
    h.click("#empty")?;
    h.click("#plain")?;
    h.click("#reset")?;
    h.click("#report")?;
    h.assert_text("#result", "3:1:1|submit,submit,submit,button,reset|seed")?;
    Ok(())
}

#[test]
fn button_form_attribute_associates_external_form_owner() -> Result<()> {
    let html = r#"
        <form id='owner'>
          <input id='field' value='x'>
        </form>
        <button id='external-submit' type='submit' form='owner'>submit</button>
        <button id='broken-submit' type='submit' form='missing'>broken</button>
        <button id='report' type='button'>report</button>
        <p id='result'></p>
        <script>
          let submits = 0;
          document.getElementById('owner').addEventListener('submit', (event) => {
            submits++;
            event.preventDefault();
          });
          document.getElementById('report').addEventListener('click', () => {
            const external = document.getElementById('external-submit');
            document.getElementById('result').textContent =
              submits + '|' + external.getAttribute('form');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#external-submit")?;
    h.click("#broken-submit")?;
    h.click("#report")?;
    h.assert_text("#result", "1|owner")?;
    Ok(())
}

#[test]
fn button_commandfor_dialog_commands_control_open_close_flow() -> Result<()> {
    let html = r#"
        <button id='open' type='button' commandfor='dialog' command='show-modal'>open</button>
        <button id='request' type='button' commandfor='dialog' command='request-close' value='ask'>request</button>
        <button id='close' type='button' commandfor='dialog' command='close' value='done'>close</button>
        <button id='report' type='button'>report</button>
        <dialog id='dialog'></dialog>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let logs = '';
          dialog.addEventListener('cancel', (event) => {
            logs = logs + 'cancel:' + dialog.returnValue + '|';
            event.preventDefault();
          });
          dialog.addEventListener('close', () => {
            logs = logs + 'close:' + dialog.returnValue + ':' + dialog.open + '|';
          });
          document.getElementById('report').addEventListener('click', () => {
            document.getElementById('result').textContent =
              logs + 'open=' + dialog.open + '|value=' + dialog.returnValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#open")?;
    h.click("#request")?;
    h.click("#close")?;
    h.click("#report")?;
    h.assert_text(
        "#result",
        "cancel:ask|close:done:false|open=false|value=done",
    )?;
    Ok(())
}

#[test]
fn press_enter_activates_focused_button() -> Result<()> {
    let html = r#"
        <button id='target' type='button'>go</button>
        <p id='result'></p>
        <script>
          let count = 0;
          document.getElementById('target').addEventListener('click', () => {
            count++;
            document.getElementById('result').textContent = String(count);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.press_enter("#target")?;
    h.assert_text("#result", "1")?;
    Ok(())
}
