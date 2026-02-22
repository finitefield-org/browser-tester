use super::*;

#[test]
fn del_cite_and_datetime_reflect_via_properties_and_attributes() -> Result<()> {
    let html = r#"
        <blockquote>
          There is <del id='removed' cite='https://example.com/changes/1' datetime='2026-02-21T09:30'>nothing</del>
          <ins>no code</ins> either good or bad.
        </blockquote>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const removed = document.getElementById('removed');
            const initial =
              removed.role + ':' +
              removed.cite + ':' +
              removed.dateTime + ':' +
              removed.textContent;

            removed.cite = 'https://example.com/changes/2';
            removed.dateTime = '2026-02-22T10:45';
            const assigned =
              removed.cite + ':' +
              removed.getAttribute('cite') + ':' +
              removed.dateTime + ':' +
              removed.getAttribute('datetime');

            removed.removeAttribute('datetime');
            const removedDatetime =
              removed.dateTime + ':' + (removed.getAttribute('datetime') === null);

            removed.setAttribute('datetime', '2026-02-23');
            const attrAssigned =
              removed.dateTime + ':' + removed.getAttribute('datetime');

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + removedDatetime + '|' + attrAssigned;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "deletion:https://example.com/changes/1:2026-02-21T09:30:nothing|https://example.com/changes/2:https://example.com/changes/2:2026-02-22T10:45:2026-02-22T10:45|:true|2026-02-23:2026-02-23",
    )?;
    Ok(())
}

#[test]
fn del_role_attribute_override_and_remove_restore_implicit_deletion() -> Result<()> {
    let html = r#"
        <p>
          <del id='removed'>thinking</del>
          <ins>running it</ins>
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const removed = document.getElementById('removed');
            const initial = removed.role;
            removed.role = 'note';
            const assigned = removed.role + ':' + removed.getAttribute('role');
            removed.removeAttribute('role');
            const restored = removed.role + ':' + (removed.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + removed.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "deletion|note:note|deletion:true|DEL")?;
    Ok(())
}
