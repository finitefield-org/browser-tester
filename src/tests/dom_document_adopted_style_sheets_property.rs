use super::*;

#[test]
fn document_adopted_style_sheets_defaults_to_empty_and_supports_push() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = document.adoptedStyleSheets.length;
            const sheet = new CSSStyleSheet();
            sheet.replaceSync('a { color: red; }');
            const pushedLength = document.adoptedStyleSheets.push(sheet);
            const insertIndex = sheet.insertRule('* { background-color: blue; }');
            const same = document.adoptedStyleSheets[0] === sheet;
            const after = document.adoptedStyleSheets.length;
            document.getElementById('result').textContent =
              [before, pushedLength, after, same, insertIndex].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "0:1:1:true:1")?;
    Ok(())
}

#[test]
fn document_adopted_style_sheets_accepts_array_assignment() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const first = new CSSStyleSheet();
            const second = new CSSStyleSheet();
            first.replaceSync('h1 { color: red; }');
            second.replaceSync('p { color: blue; }');
            document.adoptedStyleSheets = [first, second];

            const sheets = document.adoptedStyleSheets;
            document.getElementById('result').textContent = [
              sheets.length,
              sheets[0] === first,
              sheets[1] === second
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:true:true")?;
    Ok(())
}

#[test]
fn document_adopted_style_sheets_rejects_non_stylesheet_assignment() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.adoptedStyleSheets = [{}];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#run")
        .expect_err("adoptedStyleSheets should reject non-stylesheet assignment");
    match err {
        Error::ScriptRuntime(message) => {
            assert!(message.contains("NotAllowedError"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn document_adopted_style_sheets_rejects_push_of_non_stylesheet() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.adoptedStyleSheets.push({});
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#run")
        .expect_err("adoptedStyleSheets.push should reject non-stylesheet value");
    match err {
        Error::ScriptRuntime(message) => {
            assert!(message.contains("NotAllowedError"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}
