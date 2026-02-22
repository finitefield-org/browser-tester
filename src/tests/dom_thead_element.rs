use super::*;

#[test]
fn thead_has_implicit_rowgroup_role_and_multirow_structure_work() -> Result<()> {
    let html = r#"
        <table id='students'>
          <thead id='head'>
            <tr>
              <th rowspan='2'>Student ID</th>
              <th colspan='2'>Student</th>
              <th rowspan='2'>Major</th>
              <th rowspan='2'>Credits</th>
            </tr>
            <tr>
              <th>First name</th>
              <th>Last name</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>3741255</td>
              <td>Martha</td>
              <td>Jones</td>
              <td>Computer Science</td>
              <td>240</td>
            </tr>
            <tr>
              <td>3971244</td>
              <td>Victor</td>
              <td>Nim</td>
              <td>Russian Literature</td>
              <td>220</td>
            </tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const table = document.getElementById('students');
            const head = document.getElementById('head');
            document.getElementById('result').textContent =
              head.role + ':' +
              head.tagName + ':' +
              table.querySelectorAll('thead > tr').length + ':' +
              table.querySelectorAll('thead th').length + ':' +
              table.querySelector('thead th[colspan=\"2\"]').textContent.trim() + ':' +
              table.querySelector('thead > tr:nth-child(2) > th:last-child').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "rowgroup:THEAD:2:6:Student:Last name")?;
    Ok(())
}

#[test]
fn thead_deprecated_attributes_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <table>
          <thead id='head' align='center' bgcolor='#2c5e77' char='.' charoff='2' valign='middle'>
            <tr>
              <th>Items</th>
              <th>Expenditure</th>
            </tr>
          </thead>
          <tbody>
            <tr><td>Donuts</td><td>3000</td></tr>
            <tr><td>Stationery</td><td>18000</td></tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const head = document.getElementById('head');
            const initial =
              head.role + ':' +
              head.getAttribute('align') + ':' +
              head.getAttribute('bgcolor') + ':' +
              head.getAttribute('char') + ':' +
              head.getAttribute('charoff') + ':' +
              head.getAttribute('valign');

            head.setAttribute('align', 'right');
            head.setAttribute('bgcolor', '#ffeecc');
            head.setAttribute('char', ',');
            head.setAttribute('charoff', '4');
            head.setAttribute('valign', 'top');
            const updated =
              head.getAttribute('align') + ':' +
              head.getAttribute('bgcolor') + ':' +
              head.getAttribute('char') + ':' +
              head.getAttribute('charoff') + ':' +
              head.getAttribute('valign');

            head.role = 'presentation';
            const assignedRole = head.role + ':' + head.getAttribute('role');
            head.removeAttribute('role');
            const restoredRole = head.role + ':' + (head.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + assignedRole + '|' + restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "rowgroup:center:#2c5e77:.:2:middle|right:#ffeecc:,:4:top|presentation:presentation|rowgroup:true",
    )?;
    Ok(())
}
