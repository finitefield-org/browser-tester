use super::*;

#[test]
fn meter_has_implicit_role_and_range_attributes_roundtrip_work() -> Result<()> {
    let html = r#"
        <label id='fuel-label' for='fuel'>Fuel level:</label>
        <meter id='fuel' min='0' max='100' low='33' high='66' optimum='80' value='50'>
          at 50/100
        </meter>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const fuel = document.getElementById('fuel');
            const initial =
              fuel.role + ':' +
              fuel.tagName + ':' +
              fuel.value + ':' +
              fuel.getAttribute('value') + ':' +
              fuel.getAttribute('min') + ':' +
              fuel.getAttribute('max') + ':' +
              fuel.getAttribute('low') + ':' +
              fuel.getAttribute('high') + ':' +
              fuel.getAttribute('optimum') + ':' +
              document.querySelector('label[for="fuel"]').textContent.trim();

            fuel.setAttribute('value', '84');
            fuel.setAttribute('min', '10');
            fuel.setAttribute('max', '120');
            fuel.setAttribute('low', '40');
            fuel.setAttribute('high', '90');
            fuel.setAttribute('optimum', '95');

            const assigned =
              fuel.value + ':' +
              fuel.getAttribute('value') + ':' +
              fuel.getAttribute('min') + ':' +
              fuel.getAttribute('max') + ':' +
              fuel.getAttribute('low') + ':' +
              fuel.getAttribute('high') + ':' +
              fuel.getAttribute('optimum');

            document.getElementById('result').textContent = initial + '|' + assigned;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "meter:METER:50:50:0:100:33:66:80:Fuel level:|84:84:10:120:40:90:95",
    )?;
    Ok(())
}

#[test]
fn meter_role_override_and_value_property_assignment_work() -> Result<()> {
    let html = r#"
        <p id='line'>
          Battery level:
          <meter id='battery' min='0' max='1' value='0.75'>75%</meter>
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const battery = document.getElementById('battery');
            const line = document.getElementById('line');
            const initial =
              battery.role + ':' +
              line.childElementCount + ':' +
              battery.textContent.trim() + ':' +
              battery.getAttribute('value');

            battery.role = 'progressbar';
            const assigned = battery.role + ':' + battery.getAttribute('role');
            battery.removeAttribute('role');
            const restored = battery.role + ':' + (battery.getAttribute('role') === null);

            battery.value = 0.25;
            const valueByProp = battery.value;

            battery.setAttribute('value', '0.5');
            const valueByAttr = battery.value + ':' + battery.getAttribute('value');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + valueByProp + '|' + valueByAttr;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "meter:1:75%:0.75|progressbar:progressbar|meter:true|0.25|0.5:0.5",
    )?;
    Ok(())
}
