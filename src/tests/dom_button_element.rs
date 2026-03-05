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

#[test]
fn button_interface_properties_reflect_associated_targets_and_form_overrides() -> Result<()> {
    let html = r#"
        <form id='owner'>
          <label id='button-label' for='primary'>Primary</label>
        </form>
        <div id='dialog'></div>
        <div id='panel'></div>
        <div id='tip'></div>
        <button
          id='primary'
          type='submit'
          form='owner'
          name='save'
          value='seed'
          command='show-modal'
          commandfor='dialog'
          formaction='/submit/override'
          formenctype='text/plain'
          formmethod='get'
          formnovalidate
          formtarget='preview'
          interestfor='panel'
          popovertarget='tip'
          popovertargetaction='toggle'
        >Save</button>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const button = document.getElementById('primary');
            const labels = button.labels;
            const first = [
              button.command,
              button.commandForElement.id,
              button.disabled,
              button.form.id,
              button.formAction.indexOf('/submit/override') >= 0,
              button.formEnctype,
              button.formMethod,
              button.formNoValidate,
              button.formTarget,
              button.interestForElement.id,
              labels.length,
              labels.item(0).id,
              button.name,
              button.popoverTargetAction,
              button.popoverTargetElement.id,
              button.type,
              button.value
            ].join(',');

            button.command = 'request-close';
            button.commandForElement = document.getElementById('panel');
            button.formAction = '/submit/final';
            button.formEnctype = 'application/json';
            button.formMethod = 'post';
            button.formNoValidate = false;
            button.formTarget = '_blank';
            button.interestForElement = document.getElementById('dialog');
            button.popoverTargetAction = 'show';
            button.popoverTargetElement = document.getElementById('dialog');
            button.type = 'invalid';
            button.value = 'changed';
            button.disabled = true;

            const second = [
              button.command,
              button.getAttribute('commandfor'),
              button.formAction.indexOf('/submit/final') >= 0,
              button.getAttribute('formenctype'),
              button.getAttribute('formmethod'),
              button.formNoValidate,
              button.getAttribute('formtarget'),
              button.getAttribute('interestfor'),
              button.getAttribute('popovertargetaction'),
              button.getAttribute('popovertarget'),
              button.type,
              button.value,
              button.disabled
            ].join(',');

            button.commandForElement = null;
            button.interestForElement = null;
            button.popoverTargetElement = null;
            const third = [
              button.commandForElement === null,
              button.interestForElement === null,
              button.popoverTargetElement === null,
              button.getAttribute('commandfor') === null,
              button.getAttribute('interestfor') === null,
              button.getAttribute('popovertarget') === null
            ].join(',');

            document.getElementById('result').textContent = first + '|' + second + '|' + third;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "show-modal,dialog,false,owner,true,text/plain,get,true,preview,panel,1,button-label,save,toggle,tip,submit,seed|request-close,panel,true,application/json,post,false,_blank,dialog,show,dialog,submit,changed,true|true,true,true,true,true,true",
    )?;
    Ok(())
}

#[test]
fn button_validity_and_custom_validity_follow_button_constraints() -> Result<()> {
    let html = r#"
        <button id='submitter' type='submit'>submit</button>
        <button id='plain' type='button'>plain</button>
        <button id='reset' type='reset'>reset</button>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const submitter = document.getElementById('submitter');
            submitter.setCustomValidity('Need approval');
            const first = [
              submitter.willValidate,
              submitter.checkValidity(),
              submitter.reportValidity(),
              submitter.validity.customError,
              submitter.validationMessage
            ].join(',');

            submitter.type = 'button';
            const second = [
              submitter.willValidate,
              submitter.checkValidity(),
              submitter.validity.customError,
              submitter.validationMessage === ''
            ].join(',');

            submitter.type = 'submit';
            submitter.disabled = true;
            const third = [
              submitter.willValidate,
              submitter.checkValidity(),
              submitter.validity.valid,
              submitter.validationMessage === ''
            ].join(',');

            const plain = document.getElementById('plain');
            plain.setCustomValidity('x');
            const fourth = [
              plain.willValidate,
              plain.checkValidity(),
              plain.validity.customError,
              plain.validationMessage === ''
            ].join(',');

            const reset = document.getElementById('reset');
            reset.setCustomValidity('x');
            const fifth = [
              reset.willValidate,
              reset.checkValidity(),
              reset.validity.customError,
              reset.validationMessage === ''
            ].join(',');

            document.getElementById('result').textContent =
              first + '|' + second + '|' + third + '|' + fourth + '|' + fifth;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true,false,false,true,Need approval|false,true,false,true|false,true,true,true|false,true,false,true|false,true,false,true",
    )?;
    Ok(())
}
