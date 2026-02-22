use super::*;

#[test]
fn col_span_defaults_to_one_and_reflects_attributes() -> Result<()> {
    let html = r#"
        <table>
          <colgroup>
            <col id='first'>
            <col id='weekdays' span='5' class='weekdays'>
          </colgroup>
          <tbody>
            <tr><td>a</td><td>b</td></tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const first = document.getElementById('first');
            const weekdays = document.getElementById('weekdays');
            document.getElementById('result').textContent =
              first.span + ':' +
              weekdays.span + ':' +
              weekdays.getAttribute('span') + ':' +
              document.querySelectorAll('colgroup > col').length + ':' +
              weekdays.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:5:5:2:COL")?;
    Ok(())
}

#[test]
fn col_span_assignment_normalizes_invalid_values_and_role_roundtrip() -> Result<()> {
    let html = r#"
        <table>
          <colgroup>
            <col id='target' span='3'>
          </colgroup>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');

            const initial = target.span;
            target.span = 5;
            const assigned = target.span + ':' + target.getAttribute('span');

            target.setAttribute('span', '0');
            const zero = target.span;
            target.setAttribute('span', '-4');
            const negative = target.span;
            target.setAttribute('span', 'oops');
            const invalid = target.span;

            const initialRole = target.role;
            target.role = 'note';
            const assignedRole = target.role + ':' + target.getAttribute('role');
            target.removeAttribute('role');
            const restoredRole = target.role + ':' + (target.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' +
              assigned + '|' +
              zero + '|' +
              negative + '|' +
              invalid + '|' +
              initialRole + '|' +
              assignedRole + '|' +
              restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3|5:5|1|1|1||note:note|:true")?;
    Ok(())
}

#[test]
fn col_direct_dom_query_span_assignment_works() -> Result<()> {
    let html = r#"
        <table>
          <colgroup>
            <col id='target'>
          </colgroup>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('target').span = 4;
            document.getElementById('result').textContent =
              document.getElementById('target').span + ':' +
              document.getElementById('target').getAttribute('span');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "4:4")?;
    Ok(())
}
