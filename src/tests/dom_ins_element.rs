use super::*;

#[test]
fn ins_cite_and_datetime_reflect_via_properties_and_attributes() -> Result<()> {
    let html = r#"
        <p>&ldquo;You're late!&rdquo;</p>
        <del><p>&ldquo;I apologize for the delay.&rdquo;</p></del>
        <ins id='added' cite='https://example.com/changes/10' datetime='2026-02-22T08:15'>
          <p>&ldquo;A wizard is never late.&rdquo;</p>
        </ins>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const added = document.getElementById('added');
            const initial =
              added.role + ':' +
              added.cite + ':' +
              added.dateTime + ':' +
              added.querySelector('p').textContent.trim();

            added.cite = 'https://example.com/changes/11';
            added.dateTime = '2026-02-23T09:30';
            const assigned =
              added.cite + ':' +
              added.getAttribute('cite') + ':' +
              added.dateTime + ':' +
              added.getAttribute('datetime');

            added.removeAttribute('datetime');
            const removedDatetime =
              added.dateTime + ':' + (added.getAttribute('datetime') === null);

            added.setAttribute('datetime', '2026-02-24');
            const attrAssigned =
              added.dateTime + ':' + added.getAttribute('datetime');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + removedDatetime + '|' + attrAssigned;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "insertion:https://example.com/changes/10:2026-02-22T08:15:“A wizard is never late.”|https://example.com/changes/11:https://example.com/changes/11:2026-02-23T09:30:2026-02-23T09:30|:true|2026-02-24:2026-02-24",
    )?;
    Ok(())
}

#[test]
fn ins_role_attribute_override_and_remove_restore_implicit_insertion() -> Result<()> {
    let html = r#"
        <p>
          <del>thinking</del>
          <ins id='added'>running it</ins>
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const added = document.getElementById('added');
            const initial = added.role;
            added.role = 'note';
            const assigned = added.role + ':' + added.getAttribute('role');
            added.removeAttribute('role');
            const restored = added.role + ':' + (added.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + added.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "insertion|note:note|insertion:true|INS")?;
    Ok(())
}
