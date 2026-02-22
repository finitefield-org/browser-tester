use super::*;

#[test]
fn optgroup_implicit_group_role_and_attributes_roundtrip_work() -> Result<()> {
    let html = r#"
        <label for='dino-select'>Choose a dinosaur:</label>
        <select id='dino-select'>
          <optgroup id='theropods' label='Theropods'>
            <option value='trex'>Tyrannosaurus</option>
            <option value='raptor'>Velociraptor</option>
            <option value='deino'>Deinonychus</option>
          </optgroup>
          <optgroup id='sauropods' label='Sauropods' disabled>
            <option value='diplo'>Diplodocus</option>
            <option value='salta'>Saltasaurus</option>
            <option value='apato'>Apatosaurus</option>
          </optgroup>
        </select>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const theropods = document.getElementById('theropods');
            const sauropods = document.getElementById('sauropods');

            const initial =
              theropods.role + ':' +
              theropods.getAttribute('label') + ':' +
              theropods.querySelectorAll('option').length + ':' +
              sauropods.role + ':' +
              sauropods.disabled + ':' +
              sauropods.getAttribute('label');

            theropods.role = 'presentation';
            const assigned = theropods.role + ':' + theropods.getAttribute('role');
            theropods.removeAttribute('role');
            const restored = theropods.role + ':' + (theropods.getAttribute('role') === null);

            sauropods.disabled = false;
            sauropods.setAttribute('label', 'Long-necked');
            const updated =
              sauropods.disabled + ':' +
              (sauropods.getAttribute('disabled') === null) + ':' +
              sauropods.getAttribute('label');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + updated;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "group:Theropods:3:group:true:Sauropods|presentation:presentation|group:true|false:true:Long-necked",
    )?;
    Ok(())
}

#[test]
fn optgroup_optional_end_tag_before_next_optgroup_parses_as_siblings() -> Result<()> {
    let html = r#"
        <select id='cities'>
          <optgroup id='jp' label='Japan'>
            <legend id='jp-legend'>Japan cities</legend>
            <option value='tokyo'>Tokyo</option>
          <optgroup id='fr' label='France'>
            <option value='paris'>Paris</option>
          </optgroup>
        </select>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const jp = document.getElementById('jp');
            const fr = document.getElementById('fr');

            document.getElementById('result').textContent =
              document.querySelectorAll('#cities > optgroup').length + ':' +
              jp.querySelectorAll('option').length + ':' +
              fr.querySelectorAll('option').length + ':' +
              jp.querySelectorAll('optgroup').length + ':' +
              document.getElementById('jp-legend').textContent.trim() + ':' +
              jp.role + ':' + fr.role;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:1:1:0:Japan cities:group:group")?;
    Ok(())
}
