use super::*;

#[test]
fn table_has_implicit_table_role_and_structured_sections_work() -> Result<()> {
    let html = r#"
        <table id='course'>
          <caption>Front-end web developer course 2021</caption>
          <thead>
            <tr>
              <th scope='col'>Person</th>
              <th scope='col'>Most interest in</th>
              <th scope='col'>Age</th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <th scope='row'>Chris</th>
              <td>HTML tables</td>
              <td>22</td>
            </tr>
            <tr>
              <th scope='row'>Dennis</th>
              <td>Web accessibility</td>
              <td>45</td>
            </tr>
            <tr>
              <th scope='row'>Sarah</th>
              <td>JavaScript frameworks</td>
              <td>29</td>
            </tr>
          </tbody>
          <tfoot>
            <tr>
              <th scope='row' colspan='2'>Average age</th>
              <td>32</td>
            </tr>
          </tfoot>
        </table>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const table = document.getElementById('course');
            document.getElementById('result').textContent =
              table.role + ':' +
              table.tagName + ':' +
              table.querySelectorAll('thead > tr').length + ':' +
              table.querySelectorAll('tbody > tr').length + ':' +
              table.querySelectorAll('tfoot > tr').length + ':' +
              table.querySelectorAll('tbody > tr > th[scope=\"row\"]').length + ':' +
              table.querySelectorAll('tbody td').length + ':' +
              table.querySelector('caption').textContent.replace(/\s+/g, ' ').trim();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "table:TABLE:1:3:1:3:6:Front-end web developer course 2021",
    )?;
    Ok(())
}

#[test]
fn table_deprecated_attributes_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <table
          id='members'
          border='1'
          cellpadding='4'
          cellspacing='0'
          rules='rows'
          frame='box'
          summary='Member status'
          width='100%'>
          <tr>
            <th scope='col'>Name</th>
            <th scope='col'>ID</th>
          </tr>
          <tr>
            <td>Margaret Nguyen</td>
            <td>427311</td>
          </tr>
        </table>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const table = document.getElementById('members');

            const initial =
              table.role + ':' +
              table.getAttribute('border') + ':' +
              table.getAttribute('cellpadding') + ':' +
              table.getAttribute('cellspacing') + ':' +
              table.getAttribute('rules') + ':' +
              table.getAttribute('frame') + ':' +
              table.getAttribute('summary') + ':' +
              table.getAttribute('width');

            table.setAttribute('bgcolor', '#ffeecc');
            table.setAttribute('align', 'center');
            const deprecated =
              table.getAttribute('bgcolor') + ':' +
              table.getAttribute('align');

            table.role = 'grid';
            const assigned = table.role + ':' + table.getAttribute('role');
            table.removeAttribute('role');
            const restored = table.role + ':' + (table.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + deprecated + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "table:1:4:0:rows:box:Member status:100%|#ffeecc:center|grid:grid|table:true",
    )?;
    Ok(())
}

#[test]
fn table_inner_html_html_8_5_13_2_6_4_9_wraps_direct_rows_in_implied_tbody() -> Result<()> {
    let html = r#"
        <table id='scores'><caption>before</caption></table>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const table = document.getElementById('scores');
            table.innerHTML = '<tr id="a"><td>Alpha</td></tr><tr id="b"><td>Beta</td></tr>';

            document.getElementById('result').textContent = [
              document.querySelectorAll('#scores > tbody').length,
              document.querySelectorAll('#scores > tr').length,
              document.querySelectorAll('#scores > tbody > tr').length,
              Array.from(document.querySelectorAll('#scores > tbody > tr > td'))
                .map((cell) => cell.textContent)
                .join(',')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:0:2:Alpha,Beta")?;
    Ok(())
}

#[test]
fn table_insert_adjacent_html_html_13_2_6_4_9_wraps_direct_rows_in_new_tbody() -> Result<()> {
    let html = r#"
        <table id='scores'>
          <tbody>
            <tr id='existing'><td>Existing</td></tr>
          </tbody>
        </table>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const table = document.getElementById('scores');
            table.insertAdjacentHTML('beforeend', '<tr id="late"><td>Late</td></tr>');
            const late = document.getElementById('late');

            document.getElementById('result').textContent = [
              document.querySelectorAll('#scores > tbody').length,
              document.querySelectorAll('#scores > tr').length,
              late.parentElement.tagName,
              Array.from(document.querySelectorAll('#scores > tbody'))
                .map((body) => body.textContent.trim())
                .join('|')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:0:TBODY:Existing|Late")?;
    Ok(())
}
