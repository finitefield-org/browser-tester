use super::*;

#[test]
fn code_implicit_role_and_inline_fragment_work() -> Result<()> {
    let html = r#"
        <p>
          The <code id='method'>push()</code> method adds one or more elements.
        </p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const method = document.getElementById('method');
            document.getElementById('result').textContent =
              method.role + ':' +
              method.tagName + ':' +
              method.textContent + ':' +
              document.querySelectorAll('p code').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "code:CODE:push():1")?;
    Ok(())
}

#[test]
fn code_inside_pre_keeps_multiline_text_and_role_roundtrip() -> Result<()> {
    let html = r#"
        <pre id='block'><code id='snippet'>line1
line2</code></pre>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const snippet = document.getElementById('snippet');
            const initial = snippet.role;
            snippet.role = 'note';
            const assigned = snippet.role + ':' + snippet.getAttribute('role');
            snippet.removeAttribute('role');
            const restored = snippet.role + ':' + (snippet.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' +
              assigned + '|' +
              restored + '|' +
              snippet.textContent + '|' +
              document.querySelectorAll('pre code').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "code|note:note|code:true|line1\nline2|1")?;
    Ok(())
}
