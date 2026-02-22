use super::*;

#[test]
fn datalist_input_binding_keeps_suggestions_and_allows_arbitrary_input_value() -> Result<()> {
    let html = r#"
        <label for='ice-cream-choice'>Choose a flavor:</label>
        <input list='ice-cream-flavors' id='ice-cream-choice' name='ice-cream-choice' />

        <datalist id='ice-cream-flavors'>
          <option value='Chocolate'></option>
          <option value='Coconut'></option>
          <option value='Mint'></option>
          <option value='Strawberry'></option>
          <option value='Vanilla'></option>
        </datalist>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const input = document.getElementById('ice-cream-choice');
            const flavors = document.getElementById('ice-cream-flavors');
            const options = document.querySelectorAll('#ice-cream-flavors option');

            input.value = 'Mint';
            const fromSuggestions = input.value;
            input.value = 'Pistachio';
            const arbitrary = input.value;

            document.getElementById('result').textContent =
              flavors.role + ':' +
              input.getAttribute('list') + ':' +
              options.length + ':' +
              options[0].value + ',' +
              options[1].value + ',' +
              options[2].value + ',' +
              options[3].value + ',' +
              options[4].value + ':' +
              fromSuggestions + ':' +
              arbitrary;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "listbox:ice-cream-flavors:5:Chocolate,Coconut,Mint,Strawberry,Vanilla:Mint:Pistachio",
    )?;
    Ok(())
}

#[test]
fn datalist_role_attribute_override_and_remove_restore_implicit_listbox() -> Result<()> {
    let html = r#"
        <datalist id='choices'>
          <option value='A'></option>
          <option value='B'></option>
        </datalist>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const choices = document.getElementById('choices');
            const initial = choices.role;
            choices.role = 'list';
            const assigned = choices.role + ':' + choices.getAttribute('role');
            choices.removeAttribute('role');
            const restored = choices.role + ':' + (choices.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "listbox|list:list|listbox:true")?;
    Ok(())
}
