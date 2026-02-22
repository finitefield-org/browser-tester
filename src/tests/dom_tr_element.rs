use super::*;

#[test]
fn tr_has_implicit_row_role_and_table_direct_rows_gain_implied_tbody() -> Result<()> {
    let html = r#"
        <table id='phonetic'>
          <caption>Code words</caption>
          <tr id='row-a'>
            <th scope='row'>A</th>
            <td>Alfa</td>
            <td>AL fah</td>
          </tr>
          <tr id='row-b'>
            <th scope='row'>B</th>
            <td>Bravo</td>
            <td>BRAH voh</td>
          </tr>
          <tr id='row-c'>
            <th scope='row'>C</th>
            <td>Charlie</td>
            <td>CHAR lee</td>
          </tr>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const row = document.getElementById('row-a');
            document.getElementById('result').textContent =
              row.role + ':' +
              row.tagName + ':' +
              document.querySelectorAll('#phonetic > caption').length + ':' +
              document.querySelectorAll('#phonetic > tbody').length + ':' +
              document.querySelectorAll('#phonetic > tr').length + ':' +
              document.querySelectorAll('#phonetic > tbody > tr').length + ':' +
              document.querySelectorAll('#phonetic > tbody > tr > th[scope=\"row\"]').length + ':' +
              document.querySelector('#phonetic > tbody > tr > td').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "row:TR:1:1:0:3:3:Alfa")?;
    Ok(())
}

#[test]
fn tr_deprecated_attributes_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <table>
          <tbody>
            <tr
              id='row'
              align='right'
              bgcolor='#eeeeee'
              char='.'
              charoff='2'
              valign='middle'>
              <th scope='row'>Donuts</th>
              <td>3000</td>
            </tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const row = document.getElementById('row');
            const initial =
              row.role + ':' +
              row.getAttribute('align') + ':' +
              row.getAttribute('bgcolor') + ':' +
              row.getAttribute('char') + ':' +
              row.getAttribute('charoff') + ':' +
              row.getAttribute('valign');

            row.setAttribute('align', 'center');
            row.setAttribute('bgcolor', '#ffeecc');
            row.setAttribute('char', ',');
            row.setAttribute('charoff', '4');
            row.setAttribute('valign', 'top');
            const updated =
              row.getAttribute('align') + ':' +
              row.getAttribute('bgcolor') + ':' +
              row.getAttribute('char') + ':' +
              row.getAttribute('charoff') + ':' +
              row.getAttribute('valign');

            row.role = 'presentation';
            const assignedRole = row.role + ':' + row.getAttribute('role');
            row.removeAttribute('role');
            const restoredRole = row.role + ':' + (row.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + assignedRole + '|' + restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "row:right:#eeeeee:.:2:middle|center:#ffeecc:,:4:top|presentation:presentation|row:true",
    )?;
    Ok(())
}
