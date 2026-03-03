use super::*;

#[test]
fn element_toggle_attribute_mdn_basic_example_toggles_disabled() -> Result<()> {
    let html = r#"
        <input id='target' value='text'>
        <button id='run'>toggle</button>
        <p id='result'></p>
        <script>
          const button = document.getElementById('run');
          const input = document.getElementById('target');
          button.addEventListener('click', () => {
            const ret = input.toggleAttribute('disabled');
            document.getElementById('result').textContent = [
              String(ret),
              String(input.hasAttribute('disabled')),
              String(input.disabled)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true")?;
    h.click("#run")?;
    h.assert_text("#result", "false:false:false")?;
    Ok(())
}

#[test]
fn element_toggle_attribute_force_argument_controls_final_presence() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const add = target.toggleAttribute('hidden', true);
            const addAgain = target.toggleAttribute('hidden', true);
            const remove = target.toggleAttribute('hidden', false);
            const removeAgain = target.toggleAttribute('hidden', false);
            document.getElementById('result').textContent = [
              add,
              addAgain,
              remove,
              removeAgain,
              target.hasAttribute('hidden')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:false:false:false")?;
    Ok(())
}

#[test]
fn element_toggle_attribute_lowercases_name_on_html_elements() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            target.toggleAttribute('DATA-FLAG');
            document.getElementById('result').textContent = [
              target.hasAttribute('data-flag'),
              target.getAttribute('data-flag') === '',
              target.hasAttribute('DATA-FLAG')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true")?;
    Ok(())
}

#[test]
fn element_toggle_attribute_throws_invalid_character_error_for_bad_name() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            let invalid = false;
            try {
              target.toggleAttribute('1bad');
            } catch (e) {
              invalid = String(e).includes('InvalidCharacterError');
            }
            document.getElementById('result').textContent = String(invalid);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn element_toggle_attribute_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            target.toggleAttribute();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("toggleAttribute requires one or two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
