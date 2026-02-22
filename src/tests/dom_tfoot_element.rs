use super::*;

#[test]
fn tfoot_has_implicit_rowgroup_role_and_summary_structure_work() -> Result<()> {
    let html = r#"
        <table id='credits'>
          <caption>Status of the club members 2021</caption>
          <thead>
            <tr>
              <th scope='col'>Student ID</th>
              <th scope='col'>Name</th>
              <th scope='col'>Major</th>
              <th scope='col'>Credits</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td>3741255</td>
              <td>Jones, Martha</td>
              <td>Computer Science</td>
              <td>240</td>
            </tr>
            <tr>
              <td>3971244</td>
              <td>Nim, Victor</td>
              <td>Russian Literature</td>
              <td>220</td>
            </tr>
            <tr>
              <td>4100332</td>
              <td>Petrov, Alexandra</td>
              <td>Astrophysics</td>
              <td>260</td>
            </tr>
          </tbody>
          <tfoot id='foot'>
            <tr>
              <th colspan='3' scope='row'>Average Credits</th>
              <td>240</td>
            </tr>
          </tfoot>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const table = document.getElementById('credits');
            const foot = document.getElementById('foot');
            document.getElementById('result').textContent =
              foot.role + ':' +
              foot.tagName + ':' +
              table.querySelectorAll('tfoot > tr').length + ':' +
              table.querySelectorAll('tfoot > tr > th[colspan=\"3\"]').length + ':' +
              table.querySelector('tfoot td').textContent.trim() + ':' +
              table.querySelector('tfoot th[scope=\"row\"]').textContent.trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "rowgroup:TFOOT:1:1:240:Average Credits")?;
    Ok(())
}

#[test]
fn tfoot_deprecated_attributes_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <table>
          <thead>
            <tr><th>Item</th><th>Cost</th></tr>
          </thead>
          <tbody>
            <tr><td>Donuts</td><td>3000</td></tr>
            <tr><td>Stationery</td><td>18000</td></tr>
          </tbody>
          <tfoot id='foot' align='right' bgcolor='#2c5e77' char='.' charoff='2' valign='middle'>
            <tr><th scope='row'>Totals</th><td>21000</td></tr>
          </tfoot>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const foot = document.getElementById('foot');
            const initial =
              foot.role + ':' +
              foot.getAttribute('align') + ':' +
              foot.getAttribute('bgcolor') + ':' +
              foot.getAttribute('char') + ':' +
              foot.getAttribute('charoff') + ':' +
              foot.getAttribute('valign');

            foot.setAttribute('align', 'center');
            foot.setAttribute('bgcolor', '#ffeecc');
            foot.setAttribute('char', ',');
            foot.setAttribute('charoff', '4');
            foot.setAttribute('valign', 'top');
            const updated =
              foot.getAttribute('align') + ':' +
              foot.getAttribute('bgcolor') + ':' +
              foot.getAttribute('char') + ':' +
              foot.getAttribute('charoff') + ':' +
              foot.getAttribute('valign');

            foot.role = 'presentation';
            const assignedRole = foot.role + ':' + foot.getAttribute('role');
            foot.removeAttribute('role');
            const restoredRole = foot.role + ':' + (foot.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + assignedRole + '|' + restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "rowgroup:right:#2c5e77:.:2:middle|center:#ffeecc:,:4:top|presentation:presentation|rowgroup:true",
    )?;
    Ok(())
}
