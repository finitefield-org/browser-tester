use super::*;

#[test]
fn label_explicit_and_implicit_associations_toggle_expected_controls() -> Result<()> {
    let html = r#"
        <input id='cheese' type='checkbox' name='cheese'>
        <label id='cheese-a' for='cheese'>I like cheese.</label>
        <label id='cheese-b' for='cheese'>Cheese please.</label>

        <label id='peas-label'>
          I like peas.
          <input id='peas' type='checkbox' name='peas'>
        </label>

        <input id='secret' type='hidden' value='token'>
        <label id='hidden-label' for='secret'>Hidden control</label>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const cheeseA = document.getElementById('cheese-a');
            const cheeseB = document.getElementById('cheese-b');
            const peasLabel = document.getElementById('peas-label');
            const hiddenLabel = document.getElementById('hidden-label');
            const cheese = document.getElementById('cheese');
            const peas = document.getElementById('peas');
            const secret = document.getElementById('secret');

            const initial =
              cheeseA.role + ':' +
              cheeseA.htmlFor + ':' +
              cheeseB.htmlFor + ':' +
              document.querySelectorAll("label[for='cheese']").length;

            hiddenLabel.click();
            const hiddenStep = cheese.checked + ':' + secret.value;

            cheeseB.click();
            cheeseA.click();
            peasLabel.click();
            const toggled = cheese.checked + ':' + peas.checked;

            cheeseA.htmlFor = 'peas';
            const retarget = cheeseA.htmlFor + ':' + cheeseA.getAttribute('for');

            cheeseA.click();
            const afterRetarget = cheese.checked + ':' + peas.checked;

            document.getElementById('result').textContent =
              initial + '|' + hiddenStep + '|' + toggled + '|' + retarget + '|' + afterRetarget;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":cheese:cheese:2|false:token|false:true|peas:peas|false:false",
    )?;
    Ok(())
}

#[test]
fn label_role_override_and_htmlfor_reflection_roundtrip_work() -> Result<()> {
    let html = r#"
        <label id='username-label' for='username'>Enter your username:</label>
        <input id='username' name='username' type='text'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const label = document.getElementById('username-label');
            const initial = label.role + ':' + label.htmlFor + ':' + label.tagName;

            label.role = 'note';
            const assigned = label.role + ':' + label.getAttribute('role');

            label.removeAttribute('role');
            const restored = label.role + ':' + (label.getAttribute('role') === null);

            label.removeAttribute('for');
            const removedFor = label.htmlFor + ':' + (label.getAttribute('for') === null);

            label.htmlFor = 'username';
            const reassigned = label.htmlFor + ':' + label.getAttribute('for');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + removedFor + '|' + reassigned;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":username:LABEL|note:note|:true|:true|username:username",
    )?;
    Ok(())
}

#[test]
fn label_descendant_click_focuses_and_activates_associated_controls_work() -> Result<()> {
    let html = r#"
        <input id='check' type='checkbox'>
        <label id='check-label' for='check'><span id='check-span'>Check</span></label>

        <select id='pick'>
          <option value='a'>A</option>
          <option value='b'>B</option>
        </select>
        <label id='pick-label' for='pick'><span id='pick-span'>Pick</span></label>

        <input id='upload' type='file'>
        <label id='upload-label' for='upload'><span id='upload-span'>Upload</span></label>

        <button id='save' type='button'>Save</button>
        <label id='save-label' for='save'><span id='save-span'>Save label</span></label>

        <input id='disabled' type='checkbox' disabled>
        <label id='disabled-label' for='disabled'><span id='disabled-span'>Disabled</span></label>

        <p id='result'></p>
        <script>
          const events = [];
          document.getElementById('pick').addEventListener('click', () => {
            events.push('pick:' + String(document.activeElement === document.getElementById('pick')));
          });
          document.getElementById('upload').addEventListener('click', () => {
            events.push('upload:' + String(document.activeElement === document.getElementById('upload')));
          });
          document.getElementById('save').addEventListener('click', () => {
            events.push('save:' + String(document.activeElement === document.getElementById('save')));
          });
          document.getElementById('disabled').addEventListener('click', () => {
            events.push('disabled');
          });

          document.getElementById('check-span').click();
          const a = [
            String(document.getElementById('check').checked),
            String(document.activeElement === document.getElementById('check'))
          ].join(':');

          document.getElementById('pick-span').click();
          const b = String(document.activeElement === document.getElementById('pick'));

          document.getElementById('upload-span').click();
          const c = String(document.activeElement === document.getElementById('upload'));

          document.getElementById('save-span').click();
          const d = String(document.activeElement === document.getElementById('save'));

          document.getElementById('disabled-span').click();
          const e = [
            String(document.getElementById('disabled').checked),
            String(document.activeElement === document.getElementById('save'))
          ].join(':');

          document.getElementById('result').textContent = [
            a,
            b,
            c,
            d,
            e,
            events.join(',')
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true:true|true|true|true|false:true|pick:true,upload:true,save:true",
    )?;
    Ok(())
}

#[test]
fn label_nested_interactive_descendant_does_not_retarget_associated_control_work() -> Result<()> {
    let html = r#"
        <label id='wrapper'>
          <button id='inner' type='button'><span id='inner-span'>Inner</span></button>
          <input id='check' type='checkbox'>
        </label>
        <p id='result'></p>
        <script>
          let buttonClicks = 0;
          document.getElementById('inner').addEventListener('click', () => {
            buttonClicks += 1;
          });
          document.getElementById('inner-span').click();
          document.getElementById('result').textContent = [
            String(buttonClicks),
            String(document.getElementById('check').checked),
            String(document.activeElement === document.getElementById('check')),
            String(document.activeElement === document.getElementById('inner'))
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:false:false:false")?;
    Ok(())
}
