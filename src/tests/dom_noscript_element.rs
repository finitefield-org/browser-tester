use super::*;

#[test]
fn noscript_in_body_treats_children_as_text_when_scripting_is_enabled() -> Result<()> {
    let html = r#"
        <noscript id='fallback'>
          <a id='external' href='https://www.mozilla.org/'>External Link</a>
        </noscript>
        <p id='rocks'>Rocks!</p>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const fallback = document.getElementById('fallback');
            const initial =
              fallback.role + ':' +
              fallback.tagName + ':' +
              fallback.querySelectorAll('a').length + ':' +
              document.querySelectorAll('#external').length + ':' +
              fallback.textContent.includes('<a') + ':' +
              fallback.textContent.includes('External Link') + ':' +
              document.getElementById('rocks').textContent.trim();

            fallback.role = 'none';
            const assigned = fallback.role + ':' + fallback.getAttribute('role');
            fallback.removeAttribute('role');
            const restored = fallback.role + ':' + (fallback.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", ":NOSCRIPT:0:0:true:true:Rocks!|none:none|:true")?;
    Ok(())
}

#[test]
fn noscript_in_head_keeps_meta_link_markup_as_text_when_scripting_is_enabled() -> Result<()> {
    let html = r#"
        <head>
          <meta id='outside' name='description' content='outside'>
          <noscript id='head-fallback'>
            <meta id='inside' name='robots' content='noindex'>
            <link id='inside-link' rel='stylesheet' href='/fallback.css'>
          </noscript>
        </head>
        <body>
          <button id='run' type='button'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              const fallback = document.getElementById('head-fallback');
              document.getElementById('result').textContent =
              document.querySelectorAll('head meta').length + ':' +
              document.querySelectorAll('head link').length + ':' +
              document.querySelectorAll('#inside').length + ':' +
              document.querySelectorAll('#inside-link').length + ':' +
              fallback.textContent.includes('<meta') + ':' +
              fallback.textContent.includes('<link') + ':' +
              fallback.role;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:0:0:0:true:true:")?;
    Ok(())
}
