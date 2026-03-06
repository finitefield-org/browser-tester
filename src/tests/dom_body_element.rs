use super::*;

#[test]
fn html_body_element_global_and_instanceof_work() -> Result<()> {
    let html = r#"
        <body id='main'>
          <a id='link' href='/docs'>docs</a>
          <p id='result'></p>
          <script>
            const body = document.body;
            const link = document.getElementById('link');
            document.getElementById('result').textContent = [
              typeof HTMLBodyElement,
              window.HTMLBodyElement === HTMLBodyElement,
              body instanceof HTMLBodyElement,
              body instanceof HTMLElement,
              link instanceof HTMLBodyElement
            ].join(':');
          </script>
        </body>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "function:true:true:true:false")?;
    Ok(())
}

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
    h.dispatch("window", "load")?;
    h.dispatch("window", "resize")?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "generic|none:none|generic:true|onload-set|onresize-set|yes|yes",
    )?;
    Ok(())
}

#[test]
fn body_window_event_handler_aliases_forward_to_window_handlers() -> Result<()> {
    let html = r#"
        <body id='main'>
          <button id='run'>run</button>
          <p id='result'></p>
          <script>
            const body = document.body;
            let score = 0;
            body.onhashchange = () => { score += 1; };
            const initialMirror = window.onhashchange === body.onhashchange;
            window.onhashchange = () => { score += 10; };
            const overwrittenMirror = window.onhashchange === body.onhashchange;
            body.onbeforeunload = (event) => {
              score += 100;
              event.returnValue = 'leave?';
            };
            const beforeUnloadMirror = window.onbeforeunload === body.onbeforeunload;
            body.setAttribute(
              'data-flags',
              [initialMirror, overwrittenMirror, beforeUnloadMirror].join(':')
            );
            document.getElementById('result').textContent =
              body.getAttribute('data-flags') + '|' + score;
            document.getElementById('run').addEventListener('click', () => {
              document.getElementById('result').textContent =
                body.getAttribute('data-flags') + '|' +
                String(window.onhashchange === body.onhashchange) + ':' +
                String(window.onbeforeunload === body.onbeforeunload) + '|' +
                String(window.onbeforeunload !== null) + '|' +
                String(score);
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html(html)?;
    h.assert_text("#result", "true:true:true|0")?;
    h.dispatch("window", "hashchange")?;
    h.dispatch("window", "beforeunload")?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true|true:true|true|110")?;
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
