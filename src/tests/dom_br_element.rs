use super::*;

#[test]
fn br_inserts_line_breaks_into_text_content_and_inner_text() -> Result<()> {
    let html = r#"
        <p id='poem'>One<br>Two<br />Three</p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const poem = document.getElementById('poem');
            document.getElementById('result').textContent =
              poem.textContent + '|' +
              poem.innerText + '|' +
              document.querySelectorAll('br').length + ':' +
              poem.children.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "One\nTwo\nThree|One\nTwo\nThree|2:2")?;
    Ok(())
}

#[test]
fn br_clear_property_reflects_deprecated_clear_attribute() -> Result<()> {
    let html = r#"
        <p>line1<br id='line' clear='left'>line2</p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const line = document.getElementById('line');
            const before = line.clear + ':' + line.getAttribute('clear');
            line.clear = 'all';
            const after = line.clear + ':' + line.getAttribute('clear');
            line.removeAttribute('clear');
            const removed = line.clear + ':' + (line.getAttribute('clear') === null);
            document.getElementById('result').textContent =
              before + '|' + after + '|' + removed;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "left:left|all:all|:true")?;
    Ok(())
}

#[test]
fn br_has_no_implicit_role_and_role_assignment_can_roundtrip() -> Result<()> {
    let html = r#"
        <br id='line'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const line = document.getElementById('line');
            const initial = line.role;
            line.role = 'presentation';
            const assigned = line.role + ':' + line.getAttribute('role');
            line.removeAttribute('role');
            const restored = line.role + ':' + (line.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored + '|' + line.tagName;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "|presentation:presentation|:true|BR")?;
    Ok(())
}
