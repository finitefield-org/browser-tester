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
