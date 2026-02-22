use super::*;

#[test]
fn tbody_has_implicit_rowgroup_role_and_multiple_bodies_work() -> Result<()> {
    let html = r#"
        <table id='students'>
          <thead>
            <tr>
              <th>Student ID</th>
              <th>Name</th>
              <th>Credits</th>
            </tr>
          </thead>
          <tbody id='cs'>
            <tr><th colspan='3'>Computer Science</th></tr>
            <tr><td>3741255</td><td>Jones, Martha</td><td>240</td></tr>
            <tr><td>4077830</td><td>Pierce, Benjamin</td><td>200</td></tr>
          </tbody>
          <tbody id='literature'>
            <tr><th colspan='3'>Russian Literature</th></tr>
            <tr><td>3971244</td><td>Nim, Victor</td><td>220</td></tr>
          </tbody>
          <tbody id='astro'>
            <tr><th colspan='3'>Astrophysics</th></tr>
            <tr><td>4100332</td><td>Petrov, Alexandra</td><td>260</td></tr>
            <tr><td>8892377</td><td>Toyota, Hiroko</td><td>240</td></tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const table = document.getElementById('students');
            const firstBody = document.getElementById('cs');
            const titles = Array.from(table.querySelectorAll('tbody > tr:first-child > th'))
              .map((cell) => cell.textContent.trim())
              .join(',');

            document.getElementById('result').textContent =
              firstBody.role + ':' +
              firstBody.tagName + ':' +
              table.querySelectorAll('table > tbody').length + ':' +
              table.querySelectorAll('tbody > tr').length + ':' +
              table.querySelectorAll('tbody > tr > th[colspan]').length + ':' +
              titles;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "rowgroup:TBODY:3:8:3:Computer Science,Russian Literature,Astrophysics",
    )?;
    Ok(())
}

#[test]
fn tbody_deprecated_attributes_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <table>
          <thead>
            <tr><th>Student ID</th><th>Name</th><th>Major</th><th>Credits</th></tr>
          </thead>
          <tbody id='body' align='right' bgcolor='#e4f0f5' char='.' charoff='2' valign='middle'>
            <tr><td>3741255</td><td>Jones, Martha</td><td>Computer Science</td><td>240</td></tr>
            <tr><td>3971244</td><td>Nim, Victor</td><td>Russian Literature</td><td>220</td></tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const body = document.getElementById('body');
            const initial =
              body.role + ':' +
              body.getAttribute('align') + ':' +
              body.getAttribute('bgcolor') + ':' +
              body.getAttribute('char') + ':' +
              body.getAttribute('charoff') + ':' +
              body.getAttribute('valign');

            body.setAttribute('align', 'center');
            body.setAttribute('bgcolor', '#ffeecc');
            body.setAttribute('char', ',');
            body.setAttribute('charoff', '4');
            body.setAttribute('valign', 'top');
            const updated =
              body.getAttribute('align') + ':' +
              body.getAttribute('bgcolor') + ':' +
              body.getAttribute('char') + ':' +
              body.getAttribute('charoff') + ':' +
              body.getAttribute('valign');

            body.role = 'presentation';
            const assignedRole = body.role + ':' + body.getAttribute('role');
            body.removeAttribute('role');
            const restoredRole = body.role + ':' + (body.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + assignedRole + '|' + restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "rowgroup:right:#e4f0f5:.:2:middle|center:#ffeecc:,:4:top|presentation:presentation|rowgroup:true",
    )?;
    Ok(())
}
