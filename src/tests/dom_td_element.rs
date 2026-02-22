use super::*;

#[test]
fn td_has_implicit_cell_role_and_structural_attributes_work() -> Result<()> {
    let html = r#"
        <table id='stats'>
          <thead>
            <tr>
              <th id='player' scope='col'>Player</th>
              <th id='score' scope='col'>Score</th>
              <th id='bonus' scope='col'>Bonus</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <th id='r1' scope='row'>TR-7</th>
              <td id='a' headers='r1 score' colspan='2'>4569</td>
            </tr>
            <tr>
              <th id='r2' scope='row'>Mia Oolong</th>
              <td id='b' headers='r2 score'>6219</td>
              <td headers='r2 bonus' rowspan='2'>9</td>
            </tr>
            <tr>
              <th id='r3' scope='row'>Khiresh Odo</th>
              <td headers='r3 score'>7223</td>
            </tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const a = document.getElementById('a');
            const b = document.getElementById('b');
            document.getElementById('result').textContent =
              a.role + ':' +
              a.tagName + ':' +
              a.getAttribute('headers') + ':' +
              a.getAttribute('colspan') + ':' +
              document.querySelectorAll('tbody td').length + ':' +
              document.querySelectorAll('tbody td[rowspan]').length + ':' +
              b.getAttribute('headers');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "cell:TD:r1 score:2:4:1:r2 score")?;
    Ok(())
}

#[test]
fn td_role_resolves_gridcell_with_grid_ancestor_and_roundtrips_deprecated_attributes() -> Result<()>
{
    let html = r#"
        <div role='grid'>
          <table>
            <tbody>
              <tr>
                <td
                  id='cell'
                  align='right'
                  axis='stats'
                  bgcolor='#e4f0f5'
                  char='.'
                  charoff='2'
                  height='24'
                  scope='row'
                  valign='middle'
                  width='80'>7,223</td>
              </tr>
            </tbody>
          </table>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const cell = document.getElementById('cell');
            const initial =
              cell.role + ':' +
              cell.getAttribute('align') + ':' +
              cell.getAttribute('axis') + ':' +
              cell.getAttribute('bgcolor') + ':' +
              cell.getAttribute('char') + ':' +
              cell.getAttribute('charoff') + ':' +
              cell.getAttribute('height') + ':' +
              cell.getAttribute('scope') + ':' +
              cell.getAttribute('valign') + ':' +
              cell.getAttribute('width');

            cell.setAttribute('align', 'center');
            cell.setAttribute('axis', 'totals');
            cell.setAttribute('bgcolor', '#ffeecc');
            cell.setAttribute('char', ',');
            cell.setAttribute('charoff', '4');
            cell.setAttribute('height', '30');
            cell.setAttribute('scope', 'col');
            cell.setAttribute('valign', 'top');
            cell.setAttribute('width', '96');
            const updated =
              cell.getAttribute('align') + ':' +
              cell.getAttribute('axis') + ':' +
              cell.getAttribute('bgcolor') + ':' +
              cell.getAttribute('char') + ':' +
              cell.getAttribute('charoff') + ':' +
              cell.getAttribute('height') + ':' +
              cell.getAttribute('scope') + ':' +
              cell.getAttribute('valign') + ':' +
              cell.getAttribute('width');

            cell.role = 'note';
            const assignedRole = cell.role + ':' + cell.getAttribute('role');
            cell.removeAttribute('role');
            const restoredRole = cell.role + ':' + (cell.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + assignedRole + '|' + restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "gridcell:right:stats:#e4f0f5:.:2:24:row:middle:80|center:totals:#ffeecc:,:4:30:col:top:96|note:note|gridcell:true",
    )?;
    Ok(())
}
