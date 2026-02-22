use super::*;

#[test]
fn colgroup_span_defaults_to_one_and_reflects_attributes() -> Result<()> {
    let html = r#"
        <table>
          <caption>Personal weekly activities</caption>
          <colgroup id='weekdays'></colgroup>
          <colgroup id='weekend' span='2' class='weekend'></colgroup>
          <tbody>
            <tr><td>Mon</td><td>Tue</td><td>Sat</td></tr>
          </tbody>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const weekdays = document.getElementById('weekdays');
            const weekend = document.getElementById('weekend');
            document.getElementById('result').textContent =
              weekdays.span + ':' +
              weekend.span + ':' +
              weekend.getAttribute('span') + ':' +
              document.querySelectorAll('table > colgroup').length + ':' +
              document.querySelectorAll('table > caption + colgroup').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:2:2:2:1")?;
    Ok(())
}

#[test]
fn colgroup_span_assignment_normalizes_invalid_values_and_role_roundtrip() -> Result<()> {
    let html = r#"
        <table>
          <colgroup id='group' span='5'></colgroup>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const group = document.getElementById('group');

            const initial = group.span;
            group.span = 3;
            const assigned = group.span + ':' + group.getAttribute('span');

            group.setAttribute('span', '0');
            const zero = group.span;
            group.setAttribute('span', '-4');
            const negative = group.span;
            group.setAttribute('span', 'oops');
            const invalid = group.span;

            const initialRole = group.role;
            group.role = 'note';
            const assignedRole = group.role + ':' + group.getAttribute('role');
            group.removeAttribute('role');
            const restoredRole = group.role + ':' + (group.getAttribute('role') === null);

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
    h.assert_text("#result", "5|3:3|1|1|1||note:note|:true")?;
    Ok(())
}

#[test]
fn colgroup_direct_dom_query_span_assignment_works() -> Result<()> {
    let html = r#"
        <table>
          <colgroup id='group'></colgroup>
        </table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('group').span = 4;
            document.getElementById('result').textContent =
              document.getElementById('group').span + ':' +
              document.getElementById('group').getAttribute('span');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "4:4")?;
    Ok(())
}
