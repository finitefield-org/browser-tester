use super::*;

#[test]
fn ul_has_implicit_list_role_and_nested_list_structure_works() -> Result<()> {
    let html = r#"
        <ul id='ingredients'>
          <li id='milk'>Milk</li>
          <li id='cheese'>
            Cheese
            <ul id='nested-cheese'>
              <li id='blue'>Blue cheese</li>
              <li id='feta'>Feta</li>
            </ul>
          </li>
        </ul>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ingredients = document.getElementById('ingredients');
            const nested = document.getElementById('nested-cheese');
            const cheese = document.getElementById('cheese');

            document.getElementById('result').textContent =
              ingredients.role + ':' +
              ingredients.tagName + ':' +
              document.querySelectorAll('#ingredients > li').length + ':' +
              document.getElementById('milk').role + ':' +
              nested.role + ':' +
              document.querySelectorAll('#nested-cheese > li').length + ':' +
              cheese.textContent.replace(/\s+/g, ' ').trim().includes('Blue cheese') + ':' +
              document.getElementById('feta').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "list:UL:2:listitem:list:2:true:Feta")?;
    Ok(())
}

#[test]
fn ul_type_compact_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <ul id='foods' type='square' compact>
          <li>Apples</li>
          <li>Bananas</li>
          <li>Cherries</li>
        </ul>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const foods = document.getElementById('foods');
            const initial =
              foods.role + ':' +
              foods.getAttribute('type') + ':' +
              foods.getAttribute('compact') + ':' +
              foods.querySelectorAll('li').length;

            foods.role = 'menu';
            const assigned = foods.role + ':' + foods.getAttribute('role');

            foods.removeAttribute('role');
            const restored = foods.role + ':' + (foods.getAttribute('role') === null);

            foods.setAttribute('type', 'circle');
            foods.removeAttribute('compact');
            const attrs =
              foods.getAttribute('type') + ':' +
              (foods.getAttribute('compact') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + attrs;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "list:square:true:3|menu:menu|list:true|circle:true",
    )?;
    Ok(())
}
