use super::*;

#[test]
fn fieldset_implicit_group_role_and_core_attributes_work() -> Result<()> {
    let html = r#"
        <form id='target-form'></form>
        <fieldset id='group' form='target-form' name='monster-choice'>
          <legend id='caption'>Choose your favorite monster</legend>

          <input type='radio' id='kraken' name='monster' value='K'>
          <label for='kraken'>Kraken</label><br>

          <input type='radio' id='sasquatch' name='monster' value='S'>
          <label for='sasquatch'>Sasquatch</label><br>

          <input type='radio' id='mothman' name='monster' value='M'>
          <label for='mothman'>Mothman</label>
        </fieldset>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const group = document.getElementById('group');
            const initial =
              group.role + ':' +
              group.tagName + ':' +
              group.name + ':' +
              group.getAttribute('form') + ':' +
              group.querySelectorAll('input[type=radio]').length + ':' +
              document.getElementById('caption').textContent.trim();

            group.role = 'radiogroup';
            const assigned = group.role + ':' + group.getAttribute('role');
            group.removeAttribute('role');
            const restored = group.role + ':' + (group.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "group:FIELDSET:monster-choice:target-form:3:Choose your favorite monster|radiogroup:radiogroup|group:true",
    )?;
    Ok(())
}

#[test]
fn disabled_fieldset_disables_descendants_except_first_legend_contents() -> Result<()> {
    let html = r#"
        <form>
          <fieldset id='group' disabled>
            <legend id='caption'>
              Profile
              <input id='inside-legend' type='text' value='legend-ok'>
            </legend>
            <input id='outside' type='text' value='outside'>
          </fieldset>
        </form>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          const group = document.getElementById('group');
          const inside = document.getElementById('inside-legend');
          const outside = document.getElementById('outside');

          document.getElementById('run').addEventListener('click', () => {
            inside.focus();
            const insideBefore = document.activeElement === inside;

            outside.focus();
            const outsideBefore = document.activeElement === outside;

            group.disabled = false;
            outside.focus();
            const outsideAfterEnable = document.activeElement === outside;

            group.disabled = true;
            inside.focus();
            const insideAfterReDisable = document.activeElement === inside;

            document.getElementById('result').textContent =
              insideBefore + ':' +
              outsideBefore + ':' +
              outsideAfterEnable + ':' +
              insideAfterReDisable + ':' +
              inside.disabled + ':' +
              outside.disabled + ':' +
              group.disabled;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false:true:true:false:false:true")?;
    Ok(())
}
