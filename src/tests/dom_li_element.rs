use super::*;

#[test]
fn li_implicit_role_and_value_property_behavior_work() -> Result<()> {
    let html = r#"
        <ol id='ordered' type='I'>
          <li id='third' value='3' type='I'>third item</li>
          <li id='fourth'>fourth item</li>
        </ol>

        <ul id='unordered'>
          <li id='armstrong'>Neil Armstrong</li>
        </ul>

        <menu id='commands'>
          <li id='save'>Save</li>
        </menu>

        <div>
          <li id='orphan'>Orphan item</li>
        </div>

        <button id='run' type='button'>run</button>
        <p id='result'></p>

        <script>
          document.getElementById('run').addEventListener('click', () => {
            const third = document.getElementById('third');
            const fourth = document.getElementById('fourth');
            const armstrong = document.getElementById('armstrong');
            const save = document.getElementById('save');
            const orphan = document.getElementById('orphan');

            const initialRoles =
              third.role + ':' + armstrong.role + ':' + save.role + ':' + orphan.role;

            const initialValues =
              third.value + ':' + third.getAttribute('value') + ':' + fourth.value;

            fourth.value = 9;
            const afterPropertySet =
              fourth.value + ':' + fourth.getAttribute('value');

            third.setAttribute('value', '7');
            const afterAttributeSet =
              third.value + ':' + third.getAttribute('value') + ':' +
              third.type + ':' + third.getAttribute('type');

            document.getElementById('result').textContent =
              initialRoles + '|' + initialValues + '|' + afterPropertySet + '|' + afterAttributeSet;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "listitem:listitem:listitem:|3:3:0|9:9|7:7:I:I")?;
    Ok(())
}

#[test]
fn li_role_override_and_restore_implicit_listitem() -> Result<()> {
    let html = r#"
        <ul>
          <li id='item'>first item</li>
        </ul>
        <button id='run' type='button'>run</button>
        <p id='result'></p>

        <script>
          document.getElementById('run').addEventListener('click', () => {
            const item = document.getElementById('item');
            const initial = item.role;

            item.role = 'menuitem';
            const assigned = item.role + ':' + item.getAttribute('role');

            item.removeAttribute('role');
            const restored = item.role + ':' + (item.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "listitem|menuitem:menuitem|listitem:true")?;
    Ok(())
}

#[test]
fn li_optional_end_tag_parsing_works_for_ul_and_ol() -> Result<()> {
    let html = r#"
        <p>Apollo astronauts:</p>

        <ul id='crew'>
          <li id='neil'>Neil Armstrong
          <li id='alan'>Alan Bean
          <li id='peter'>Peter Conrad
        </ul>

        <ol id='ordered'>
          <li id='third' value='3'>third item
          <li id='fourth'>fourth item
        </ol>

        <button id='run' type='button'>run</button>
        <p id='result'></p>

        <script>
          document.getElementById('run').addEventListener('click', () => {
            const crewItems = document.querySelectorAll('#crew > li');
            const orderedItems = document.querySelectorAll('#ordered > li');

            document.getElementById('result').textContent =
              crewItems.length + ':' + orderedItems.length + ':' +
              crewItems[0].textContent.trim() + ',' +
              crewItems[1].textContent.trim() + ',' +
              crewItems[2].textContent.trim() + '|' +
              orderedItems[0].textContent.trim() + ',' +
              orderedItems[1].textContent.trim() + ':' +
              orderedItems[0].value + ':' + orderedItems[1].value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "3:2:Neil Armstrong,Alan Bean,Peter Conrad|third item,fourth item:3:0",
    )?;
    Ok(())
}
