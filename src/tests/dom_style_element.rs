use super::*;

#[test]
fn style_has_no_implicit_role_and_keeps_css_text_in_head() -> Result<()> {
    let html = r#"
        <html>
          <head>
            <style id='base'>
              p {
                color: #26b72b;
              }
              code {
                font-weight: bold;
              }
            </style>
            <style id='alt' media='print' blocking='render' nonce='xyz' title='Print rules' type='text/css'>
              p { color: black; }
            </style>
          </head>
          <body>
            <p id='line'>This text will be green in supporting environments.</p>
            <button id='run' type='button'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                const base = document.getElementById('base');
                const alt = document.getElementById('alt');

                document.getElementById('result').textContent =
                  base.role + ':' +
                  base.tagName + ':' +
                  base.textContent.includes('color: #26b72b') + ':' +
                  base.textContent.includes('font-weight: bold') + ':' +
                  base.childElementCount + ':' +
                  alt.getAttribute('media') + ':' +
                  alt.getAttribute('blocking') + ':' +
                  alt.getAttribute('nonce') + ':' +
                  alt.getAttribute('title') + ':' +
                  alt.getAttribute('type') + ':' +
                  document.querySelectorAll('head style').length;
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":STYLE:true:true:0:print:render:xyz:Print rules:text/css:2",
    )?;
    Ok(())
}

#[test]
fn style_attribute_roundtrip_and_role_override_work() -> Result<()> {
    let html = r#"
        <head>
          <style id='dynamic' media='screen and (width <= 600px)' title='Base' type='text/css'>
            p { font-weight: bold; }
          </style>
        </head>
        <body>
          <button id='run' type='button'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              const dynamic = document.getElementById('dynamic');

              const initial =
                dynamic.role + ':' +
                dynamic.media + ':' +
                dynamic.title + ':' +
                dynamic.type + ':' +
                dynamic.textContent.includes('font-weight');

              dynamic.media = 'print';
              dynamic.title = 'Alternative';
              dynamic.type = 'text/css';
              dynamic.setAttribute('nonce', 'abc123');
              dynamic.setAttribute('blocking', 'render');

              const assigned =
                dynamic.getAttribute('media') + ':' +
                dynamic.media + ':' +
                dynamic.title + ':' +
                dynamic.getAttribute('title') + ':' +
                dynamic.type + ':' +
                dynamic.getAttribute('type') + ':' +
                dynamic.getAttribute('nonce') + ':' +
                dynamic.getAttribute('blocking');

              dynamic.role = 'none';
              const roleAssigned = dynamic.role + ':' + dynamic.getAttribute('role');
              dynamic.removeAttribute('role');
              const roleRestored = dynamic.role + ':' + (dynamic.getAttribute('role') === null);

              document.getElementById('result').textContent =
                initial + '|' + assigned + '|' + roleAssigned + '|' + roleRestored;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":screen and (width <= 600px):Base:text/css:true|print:print:Alternative:Alternative:text/css:text/css:abc123:render|none:none|:true",
    )?;
    Ok(())
}
