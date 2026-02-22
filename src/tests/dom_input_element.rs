use super::*;

#[test]
fn input_implicit_roles_for_common_types_and_list_variants_work() -> Result<()> {
    let html = r#"
        <datalist id='opts'>
          <option value='one'></option>
        </datalist>
        <input id='default'>
        <input id='text' type='text'>
        <input id='text-list' type='text' list='opts'>
        <input id='search' type='search'>
        <input id='search-list' type='search' list='opts'>
        <input id='email' type='email'>
        <input id='email-list' type='email' list='opts'>
        <input id='tel' type='tel'>
        <input id='tel-list' type='tel' list='opts'>
        <input id='url' type='url'>
        <input id='url-list' type='url' list='opts'>
        <input id='number' type='number'>
        <input id='checkbox' type='checkbox'>
        <input id='radio' type='radio'>
        <input id='range' type='range'>
        <input id='button' type='button'>
        <input id='submit' type='submit'>
        <input id='reset' type='reset'>
        <input id='image' type='image' alt='go' src='/go.png'>
        <input id='hidden' type='hidden'>
        <input id='password' type='password'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ids = [
              'default', 'text', 'text-list', 'search', 'search-list',
              'email', 'email-list', 'tel', 'tel-list', 'url', 'url-list',
              'number', 'checkbox', 'radio', 'range', 'button', 'submit',
              'reset', 'image', 'hidden', 'password'
            ];
            document.getElementById('result').textContent =
              ids.map((id) => document.getElementById(id).role).join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "textbox:textbox:combobox:searchbox:combobox:textbox:combobox:textbox:combobox:textbox:combobox:spinbutton:checkbox:radio:slider:button:button:button:button::",
    )?;
    Ok(())
}

#[test]
fn input_role_reacts_to_type_list_changes_and_explicit_override_roundtrip() -> Result<()> {
    let html = r#"
        <datalist id='opts'>
          <option value='two'></option>
        </datalist>
        <input id='target' type='text'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const initial = target.role;

            target.setAttribute('list', 'opts');
            const withList = target.role;

            target.removeAttribute('list');
            const withoutList = target.role;

            target.type = 'number';
            const numberRole = target.role;

            target.type = 'checkbox';
            const checkboxRole = target.role;

            target.role = 'switch';
            const assigned = target.role + ':' + target.getAttribute('role');

            target.removeAttribute('role');
            const restored = target.role + ':' + (target.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' +
              withList + '|' +
              withoutList + '|' +
              numberRole + '|' +
              checkboxRole + '|' +
              assigned + '|' +
              restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "textbox|combobox|textbox|spinbutton|checkbox|switch:switch|checkbox:true",
    )?;
    Ok(())
}
