use super::*;

#[test]
fn caption_implicit_role_and_table_position_work() -> Result<()> {
    let html = r#"
        <table id='facts'>
          <caption id='cap'>He-Man and Skeletor facts</caption>
          <tbody>
            <tr>
              <th scope='col'>He-Man</th>
              <th scope='col'>Skeletor</th>
            </tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const cap = document.getElementById('cap');
            document.getElementById('result').textContent =
              cap.role + ':' +
              cap.tagName + ':' +
              cap.textContent.trim() + ':' +
              document.querySelectorAll('table > caption').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "caption:CAPTION:He-Man and Skeletor facts:1")?;
    Ok(())
}

#[test]
fn caption_align_property_reflects_attribute_and_roundtrips_role() -> Result<()> {
    let html = r#"
        <table>
          <caption id='cap' align='top'>User login email addresses</caption>
          <tbody>
            <tr><td>user1@example.com</td></tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const cap = document.getElementById('cap');

            const initialAlign = cap.align + ':' + cap.getAttribute('align');
            cap.align = 'bottom';
            const assignedAlign = cap.align + ':' + cap.getAttribute('align');
            cap.removeAttribute('align');
            const restoredAlign = cap.align + ':' + (cap.getAttribute('align') === null);

            const initialRole = cap.role;
            cap.role = 'note';
            const assignedRole = cap.role + ':' + cap.getAttribute('role');
            cap.removeAttribute('role');
            const restoredRole = cap.role + ':' + (cap.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initialAlign + '|' +
              assignedAlign + '|' +
              restoredAlign + '|' +
              initialRole + '|' +
              assignedRole + '|' +
              restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "top:top|bottom:bottom|:true|caption|note:note|caption:true",
    )?;
    Ok(())
}

#[test]
fn caption_direct_dom_query_align_assignment_works() -> Result<()> {
    let html = r#"
        <table>
          <caption id='cap'>Facts</caption>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('cap').align = 'left';
            document.getElementById('result').textContent =
              document.getElementById('cap').align + ':' +
              document.getElementById('cap').getAttribute('align');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "left:left")?;
    Ok(())
}
