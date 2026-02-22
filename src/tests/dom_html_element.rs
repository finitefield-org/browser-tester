use super::*;

#[test]
fn html_has_implicit_document_role_and_root_structure() -> Result<()> {
    let html = r#"
        <!doctype html>
        <html id='root' lang='en'>
          <head>
            <title>Document title</title>
          </head>
          <body>
            <button id='run' type='button'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                const root = document.documentElement;
                const initial =
                  root.role + ':' +
                  root.tagName + ':' +
                  root.getAttribute('lang') + ':' +
                  document.querySelectorAll('html').length + ':' +
                  document.head.tagName + ':' +
                  document.body.tagName;

                root.role = 'none';
                const assigned = root.role + ':' + root.getAttribute('role');
                root.removeAttribute('role');
                const restored = root.role + ':' + (root.getAttribute('role') === null);

                document.getElementById('result').textContent =
                  initial + '|' + assigned + '|' + restored;
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "document:HTML:en:1:HEAD:BODY|none:none|document:true",
    )?;
    Ok(())
}

#[test]
fn html_xmlns_version_and_lang_attributes_roundtrip() -> Result<()> {
    let html = r#"
        <html id='root' xmlns='http://www.w3.org/1999/xhtml'>
          <head>
            <title>Document title</title>
          </head>
          <body>
            <button id='run' type='button'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                const root = document.getElementById('root');
                const before =
                  root.getAttribute('xmlns') + ':' +
                  (root.getAttribute('version') === null) + ':' +
                  (root.getAttribute('lang') || '');

                root.setAttribute('version', 'HTML5');
                root.setAttribute('xmlns', 'http://www.w3.org/1999/xhtml');
                root.setAttribute('lang', 'ja');

                const after =
                  root.getAttribute('version') + ':' +
                  root.getAttribute('xmlns') + ':' +
                  root.getAttribute('lang') + ':' +
                  root.getAttribute('lang');

                document.getElementById('result').textContent = before + '|' + after;
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "http://www.w3.org/1999/xhtml:true:|HTML5:http://www.w3.org/1999/xhtml:ja:ja",
    )?;
    Ok(())
}
