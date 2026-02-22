use super::*;

#[test]
fn body_implicit_generic_role_and_event_handler_attributes_work() -> Result<()> {
    let html = r#"
        <html>
          <head><title>body test</title></head>
          <body id='main'>
            <button id='run'>run</button>
            <p id='result'></p>
            <script>
              document.body.onload = () => {
                document.body.setAttribute('data-loaded', 'yes');
              };
              document.body.onresize = () => {
                document.body.setAttribute('data-resized', 'yes');
              };

              document.getElementById('run').addEventListener('click', () => {
                const body = document.body;
                const initialRole = body.role;
                body.role = 'none';
                const assignedRole = body.role + ':' + body.getAttribute('role');
                body.removeAttribute('role');
                const restoredRole = body.role + ':' + (body.getAttribute('role') === null);

                document.getElementById('result').textContent =
                  initialRole + '|' +
                  assignedRole + '|' +
                  restoredRole + '|' +
                  (body.onload ? 'onload-set' : 'onload-empty') + '|' +
                  (body.onresize ? 'onresize-set' : 'onresize-empty') + '|' +
                  (body.getAttribute('data-loaded') || '') + '|' +
                  (body.getAttribute('data-resized') || '');
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.dispatch("body", "load")?;
    h.dispatch("body", "resize")?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "generic|none:none|generic:true|onload-set|onresize-set|yes|yes",
    )?;
    Ok(())
}

#[test]
fn body_deprecated_attribute_properties_reflect() -> Result<()> {
    let html = r#"
        <body id='main'>
          <button id='run'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              document.body.aLink = '#f00';
              document.body.background = '/bg.png';
              document.body.bgColor = '#ffffff';
              document.body.bottomMargin = '10';
              document.body.leftMargin = '20';
              document.body.link = '#00f';
              document.body.rightMargin = '30';
              document.body.text = '#111';
              document.body.topMargin = '40';
              document.body.vLink = '#551a8b';

              const body = document.body;
              body.bgColor = '#eeeeee';
              body.topMargin = '44';

              document.getElementById('result').textContent =
                document.body.aLink + ':' +
                document.body.background + ':' +
                document.body.bgColor + ':' +
                document.body.bottomMargin + ':' +
                document.body.leftMargin + ':' +
                document.body.link + ':' +
                document.body.rightMargin + ':' +
                document.body.text + ':' +
                document.body.topMargin + ':' +
                document.body.vLink + '|' +
                document.body.getAttribute('alink') + ':' +
                document.body.getAttribute('background') + ':' +
                document.body.getAttribute('bgcolor') + ':' +
                document.body.getAttribute('bottommargin') + ':' +
                document.body.getAttribute('leftmargin') + ':' +
                document.body.getAttribute('link') + ':' +
                document.body.getAttribute('rightmargin') + ':' +
                document.body.getAttribute('text') + ':' +
                document.body.getAttribute('topmargin') + ':' +
                document.body.getAttribute('vlink');
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "#f00:/bg.png:#eeeeee:10:20:#00f:30:#111:44:#551a8b|#f00:/bg.png:#eeeeee:10:20:#00f:30:#111:44:#551a8b",
    )?;
    Ok(())
}

#[test]
fn document_keeps_single_body_element_when_multiple_bodies_exist() -> Result<()> {
    let html = r#"
        <html>
          <head><title>multi body</title></head>
          <body id='first'>
            <button id='run'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('run').addEventListener('click', () => {
                document.getElementById('result').textContent =
                  document.querySelectorAll('body').length + ':' +
                  document.body.id + ':' +
                  (document.body.querySelector('#b') ? 'yes' : 'no');
              });
            </script>
          </body>
          <body id='second'>
            <p id='b'>B</p>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:first:yes")?;
    Ok(())
}
