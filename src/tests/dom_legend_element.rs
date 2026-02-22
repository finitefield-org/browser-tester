use super::*;

#[test]
fn legend_captions_fieldset_and_allows_heading_content() -> Result<()> {
    let html = r#"
        <fieldset id='monster-group'>
          <legend id='monster-legend'>
            <h3 id='legend-heading'>Choose your favorite monster</h3>
          </legend>
          <input type='radio' id='kraken' name='monster' value='K'>
          <label for='kraken'>Kraken</label>
          <input type='radio' id='sasquatch' name='monster' value='S'>
          <label for='sasquatch'>Sasquatch</label>
          <input type='radio' id='mothman' name='monster' value='M'>
          <label for='mothman'>Mothman</label>
        </fieldset>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const legend = document.getElementById('monster-legend');
            const heading = document.getElementById('legend-heading');
            const fieldset = document.getElementById('monster-group');
            document.getElementById('result').textContent =
              legend.role + ':' +
              legend.tagName + ':' +
              heading.tagName + ':' +
              heading.role + ':' +
              legend.textContent.trim() + ':' +
              fieldset.querySelectorAll('input[type=radio]').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":LEGEND:H3:heading:Choose your favorite monster:3",
    )?;
    Ok(())
}

#[test]
fn legend_role_override_restore_and_optgroup_child_usage_work() -> Result<()> {
    let html = r#"
        <select id='cities'>
          <optgroup id='jp' label='Japan'>
            <legend id='jp-legend'>Japan cities</legend>
            <option value='tokyo'>Tokyo</option>
            <option value='osaka'>Osaka</option>
          </optgroup>
        </select>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const legend = document.getElementById('jp-legend');
            const optgroup = document.getElementById('jp');
            const initial =
              legend.role + ':' +
              legend.textContent.trim() + ':' +
              optgroup.getAttribute('label') + ':' +
              optgroup.querySelectorAll('option').length;

            legend.role = 'note';
            const assigned = legend.role + ':' + legend.getAttribute('role');

            legend.removeAttribute('role');
            const restored = legend.role + ':' + (legend.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":Japan cities:Japan:2|note:note|:true")?;
    Ok(())
}
