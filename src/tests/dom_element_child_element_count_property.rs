use super::*;

#[test]
fn element_child_element_count_counts_only_element_children_and_updates_live() -> Result<()> {
    let html = r#"
        <div id='sidebar'><p>One</p><span>Two</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sidebar = document.getElementById('sidebar');
            const initial = sidebar.childElementCount;

            const text = document.createTextNode('text-only');
            sidebar.appendChild(text);
            const afterText = sidebar.childElementCount;

            const strong = document.createElement('strong');
            sidebar.appendChild(strong);
            const afterElement = sidebar.childElementCount;

            sidebar.removeChild(strong);
            const afterRemove = sidebar.childElementCount;

            document.getElementById('result').textContent = [
              initial,
              afterText,
              afterElement,
              afterRemove
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:2:3:2")?;
    Ok(())
}

#[test]
fn element_child_element_count_supports_basic_conditional_usage() -> Result<()> {
    let html = r#"
        <div id='sidebar'><a href='#'>Link</a></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sidebar = document.getElementById('sidebar');
            let output = 'empty';
            if (sidebar.childElementCount > 0) {
              output = 'has-children';
            }
            document.getElementById('result').textContent = output + ':' + sidebar.childElementCount;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "has-children:1")?;
    Ok(())
}

#[test]
fn element_child_element_count_rejects_assignment() -> Result<()> {
    let html = r#"
        <div id='sidebar'><a href='#'>Link</a></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sidebar = document.getElementById('sidebar');
            sidebar.childElementCount = 99;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("childElementCount is read-only"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
