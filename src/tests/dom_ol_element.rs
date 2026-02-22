use super::*;

#[test]
fn ol_has_implicit_list_role_and_nested_list_structure_works() -> Result<()> {
    let html = r#"
        <ol id='recipe'>
          <li id='step1'>Mix flour, baking powder, sugar, and salt.</li>
          <li id='step2'>In another bowl, mix eggs, milk, and oil.</li>
          <li id='step3'>Stir both mixtures together.</li>
          <li id='step4'>Fill muffin tray 3/4 full.</li>
          <li id='step5'>Bake for 20 minutes.</li>
        </ol>

        <ol id='nested'>
          <li id='outer-1'>first item</li>
          <li id='outer-2'>
            second item
            <ol id='nested-inner'>
              <li id='inner-1'>second item first subitem</li>
              <li id='inner-2'>second item second subitem</li>
              <li id='inner-3'>second item third subitem</li>
            </ol>
          </li>
          <li id='outer-3'>third item</li>
        </ol>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const recipe = document.getElementById('recipe');
            const nested = document.getElementById('nested');
            const nestedInner = document.getElementById('nested-inner');

            document.getElementById('result').textContent =
              recipe.role + ':' +
              recipe.tagName + ':' +
              document.querySelectorAll('#recipe > li').length + ':' +
              document.getElementById('step2').role + ':' +
              nested.role + ':' +
              nestedInner.role + ':' +
              document.querySelectorAll('#nested > li').length + ':' +
              document.querySelectorAll('#nested-inner > li').length + ':' +
              document.getElementById('inner-2').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "list:OL:5:listitem:list:list:3:3:second item second subitem",
    )?;
    Ok(())
}

#[test]
fn ol_start_reversed_type_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <p>Finishing places of contestants not in the winners' circle:</p>

        <ol id='ranking' start='4' reversed type='I' compact>
          <li id='place4'>Speedwalk Stu</li>
          <li id='place5'>Saunterin' Sam</li>
          <li id='place6'>Slowpoke Rodriguez</li>
        </ol>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ranking = document.getElementById('ranking');

            const initial =
              ranking.role + ':' +
              ranking.getAttribute('start') + ':' +
              (ranking.getAttribute('reversed') !== null) + ':' +
              ranking.getAttribute('type') + ':' +
              ranking.getAttribute('compact') + ':' +
              ranking.querySelectorAll('li').length;

            ranking.role = 'menu';
            const assigned = ranking.role + ':' + ranking.getAttribute('role');

            ranking.removeAttribute('role');
            const restored = ranking.role + ':' + (ranking.getAttribute('role') === null);

            ranking.setAttribute('start', '7');
            ranking.removeAttribute('reversed');
            ranking.setAttribute('type', 'a');
            ranking.removeAttribute('compact');

            document.getElementById('place4').setAttribute('value', '4');
            const attrs =
              ranking.getAttribute('start') + ':' +
              (ranking.getAttribute('reversed') === null) + ':' +
              ranking.getAttribute('type') + ':' +
              (ranking.getAttribute('compact') === null);
            const liValue =
              document.getElementById('place4').value + ':' +
              document.getElementById('place4').getAttribute('value');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + attrs + '|' + liValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "list:4:true:I:true:3|menu:menu|list:true|7:true:a:true|4:4",
    )?;
    Ok(())
}
