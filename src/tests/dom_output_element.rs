use super::*;

#[test]
fn output_role_value_and_form_elements_name_lookup_work() -> Result<()> {
    let html = r#"
        <form id='calc'>
          <input id='a' name='a' type='number' value='10'>
          <input id='b' name='b' type='number' value='20'>
        </form>
        <output id='sum' name='result' form='calc' for='a b'>30</output>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('calc');
            const out = document.getElementById('sum');
            const byName = form.elements['result'];
            const byId = form.elements['sum'];
            const byIndex = form.elements[2];
            const formData = new FormData(form);

            const initial =
              out.role + ':' +
              out.tagName + ':' +
              out.value + ':' +
              out.textContent + ':' +
              out.getAttribute('for') + ':' +
              form.elements.length + ':' +
              (byName === out) + ':' +
              (byId === out) + ':' +
              (byIndex === out) + ':' +
              formData.has('result');

            out.value = '99';
            const afterValueSet = out.value + ':' + out.textContent;

            out.textContent = '77';
            const afterTextSet = out.value + ':' + out.textContent;

            form.reset();
            const afterReset = out.value + ':' + out.textContent;

            document.getElementById('result').textContent =
              initial + '|' + afterValueSet + '|' + afterTextSet + '|' + afterReset;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "status:OUTPUT:30:30:a b:3:true:true:true:false|99:99|77:77|30:30",
    )?;
    Ok(())
}

#[test]
fn output_form_owner_override_controls_elements_membership_and_reset_scope() -> Result<()> {
    let html = r#"
        <form id='alpha'>
          <input id='alpha-in' name='alpha-in' value='A'>
          <output id='shared-output' name='shared' form='beta'>seed</output>
        </form>

        <form id='beta'>
          <input id='beta-in' name='beta-in' value='B'>
        </form>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const alpha = document.getElementById('alpha');
            const beta = document.getElementById('beta');
            const out = document.getElementById('shared-output');

            const ownership =
              alpha.elements.length + ':' +
              beta.elements.length + ':' +
              (beta.elements['shared'] === out);

            out.value = 'changed';
            alpha.reset();
            const afterAlphaReset = out.value;

            beta.reset();
            const afterBetaReset = out.value;

            document.getElementById('result').textContent =
              ownership + '|' + afterAlphaReset + ':' + afterBetaReset;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:2:true|changed:seed")?;
    Ok(())
}
