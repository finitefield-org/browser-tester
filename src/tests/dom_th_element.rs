use super::*;

#[test]
fn th_implicit_roles_and_header_attributes_work() -> Result<()> {
    let html = r#"
        <table id='stars'>
          <thead>
            <tr>
              <th id='h-col' scope='col'>Player</th>
              <th id='h-colgroup' scope='colgroup' colspan='2'>Pronunciation</th>
            </tr>
            <tr>
              <th id='h-fallback-col'>IPA</th>
              <th id='h-associated' scope='col' headers='h-colgroup'>Respelling</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <th id='h-row' scope='row' abbr='TR'>TR-7</th>
              <td>7</td>
              <td>4,569</td>
            </tr>
            <tr>
              <th id='h-rowgroup' scope='rowgroup' rowspan='2'>Group A</th>
              <td>8</td>
              <td>5,000</td>
            </tr>
            <tr>
              <td>9</td>
              <td>6,000</td>
            </tr>
            <tr>
              <th id='h-fallback-row'>No scope row header</th>
              <td>10</td>
              <td>7,000</td>
            </tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const col = document.getElementById('h-col');
            const colgroup = document.getElementById('h-colgroup');
            const row = document.getElementById('h-row');
            const rowgroup = document.getElementById('h-rowgroup');
            const fallbackCol = document.getElementById('h-fallback-col');
            const fallbackRow = document.getElementById('h-fallback-row');
            const associated = document.getElementById('h-associated');

            document.getElementById('result').textContent =
              col.role + ':' +
              colgroup.role + ':' +
              row.role + ':' +
              rowgroup.role + ':' +
              fallbackCol.role + ':' +
              fallbackRow.role + ':' +
              row.getAttribute('abbr') + ':' +
              associated.getAttribute('headers') + ':' +
              colgroup.getAttribute('colspan') + ':' +
              rowgroup.getAttribute('rowspan');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "columnheader:columnheader:rowheader:rowheader:columnheader:rowheader:TR:h-colgroup:2:2",
    )?;
    Ok(())
}

#[test]
fn th_scope_deprecated_attributes_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <table>
          <tbody>
            <tr>
              <th
                id='head'
                scope='row'
                align='right'
                axis='stats'
                bgcolor='#e4f0f5'
                char='.'
                charoff='2'
                height='24'
                valign='middle'
                width='80'>Credits</th>
              <td>240</td>
            </tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const head = document.getElementById('head');
            const initial =
              head.role + ':' +
              head.getAttribute('scope') + ':' +
              head.getAttribute('align') + ':' +
              head.getAttribute('axis') + ':' +
              head.getAttribute('bgcolor') + ':' +
              head.getAttribute('char') + ':' +
              head.getAttribute('charoff') + ':' +
              head.getAttribute('height') + ':' +
              head.getAttribute('valign') + ':' +
              head.getAttribute('width');

            head.setAttribute('scope', 'col');
            const colRole = head.role;
            head.removeAttribute('scope');
            const fallbackRole = head.role;

            head.setAttribute('align', 'center');
            head.setAttribute('axis', 'totals');
            head.setAttribute('bgcolor', '#ffeecc');
            head.setAttribute('char', ',');
            head.setAttribute('charoff', '4');
            head.setAttribute('height', '30');
            head.setAttribute('valign', 'top');
            head.setAttribute('width', '96');
            const updated =
              head.getAttribute('align') + ':' +
              head.getAttribute('axis') + ':' +
              head.getAttribute('bgcolor') + ':' +
              head.getAttribute('char') + ':' +
              head.getAttribute('charoff') + ':' +
              head.getAttribute('height') + ':' +
              head.getAttribute('valign') + ':' +
              head.getAttribute('width');

            head.role = 'note';
            const assignedRole = head.role + ':' + head.getAttribute('role');
            head.removeAttribute('role');
            const restoredRole = head.role + ':' + (head.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + colRole + ':' + fallbackRole + '|' + updated + '|' + assignedRole + '|' + restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "rowheader:row:right:stats:#e4f0f5:.:2:24:middle:80|columnheader:rowheader|center:totals:#ffeecc:,:4:30:top:96|note:note|rowheader:true",
    )?;
    Ok(())
}
