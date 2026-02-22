use super::*;

#[test]
fn title_treats_nested_html_markup_as_plain_text_and_syncs_document_title() -> Result<()> {
    let html = r#"
        <html>
          <head>
            <title id='doc-title'>Grandma's <b>Heavy</b> &amp; Metal</title>
          </head>
          <body>
            <button id='run'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                const titleEl = document.getElementById('doc-title');
                const initial =
                  titleEl.tagName + ':' +
                  titleEl.role + ':' +
                  titleEl.text + ':' +
                  document.title + ':' +
                  titleEl.querySelectorAll('*').length;

                document.title = 'Updated <em>Title</em>';
                const updated =
                  titleEl.text + ':' +
                  document.title + ':' +
                  titleEl.querySelectorAll('*').length;

                document.getElementById('result').textContent =
                  initial + '|' + updated;
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "TITLE::Grandma's <b>Heavy</b> & Metal:Grandma's <b>Heavy</b> & Metal:0|Updated <em>Title</em>:Updated <em>Title</em>:0",
    )?;
    Ok(())
}

#[test]
fn title_text_and_global_title_attribute_roundtrip_with_role_override_work() -> Result<()> {
    let html = r#"
        <html>
          <head>
            <title id='doc-title'>Awesome interesting stuff</title>
          </head>
          <body>
            <button id='run'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                const titleEl = document.getElementById('doc-title');
                const initial =
                  titleEl.role + ':' +
                  titleEl.text + ':' +
                  document.title + ':' +
                  (titleEl.getAttribute('title') === null);

                titleEl.title = 'Document tooltip';
                titleEl.text = 'Menu - Blue House Chinese Food';
                const updated =
                  titleEl.title + ':' +
                  titleEl.getAttribute('title') + ':' +
                  titleEl.text + ':' +
                  document.title;

                titleEl.role = 'none';
                const assignedRole = titleEl.role + ':' + titleEl.getAttribute('role');
                titleEl.removeAttribute('role');
                const restoredRole = titleEl.role + ':' + (titleEl.getAttribute('role') === null);

                document.getElementById('result').textContent =
                  initial + '|' + updated + '|' + assignedRole + '|' + restoredRole + '|' + titleEl.tagName;
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":Awesome interesting stuff:Awesome interesting stuff:true|Document tooltip:Document tooltip:Menu - Blue House Chinese Food:Menu - Blue House Chinese Food|none:none|:true|TITLE",
    )?;
    Ok(())
}
