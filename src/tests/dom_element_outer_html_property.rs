use super::*;

#[test]
fn element_outer_html_getter_serializes_element_and_descendants() -> Result<()> {
    let html = r#"
        <div id='example'>
          <p>Content</p>
          <p>Further Elaborated</p>
        </div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const example = document.getElementById('example');
            const serialized = example.outerHTML;
            document.getElementById('result').textContent = [
              serialized.includes('<div id="example">'),
              serialized.includes('<p>Content</p>'),
              serialized.includes('<p>Further Elaborated</p>')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true")?;
    Ok(())
}

#[test]
fn element_outer_html_set_replaces_node_but_old_reference_stays_same_object() -> Result<()> {
    let html = r#"
        <div id='host'><p id='target'>Original</p></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const p = document.getElementById('target');
            const before = p.tagName;

            p.outerHTML = '<div id="target">Replaced</div>';

            const after = p.tagName;
            const live = document.getElementById('target').tagName;
            const oldGone = document.querySelectorAll('p#target').length;
            document.getElementById('result').textContent =
              [before, after, live, oldGone].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "P:P:DIV:0")?;
    Ok(())
}

#[test]
fn element_outer_html_set_null_removes_target_element() -> Result<()> {
    let html = r#"
        <div id='host'><span id='x'>X</span><b id='y'>Y</b></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const x = document.getElementById('x');
            x.outerHTML = null;
            const host = document.getElementById('host');
            document.getElementById('result').textContent =
              host.textContent + ':' +
              document.querySelectorAll('#x').length + ':' +
              document.querySelectorAll('#y').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "Y:0:1")?;
    Ok(())
}

#[test]
fn element_outer_html_set_on_detached_element_is_noop() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const div = document.createElement('div');
            div.id = 'detached';
            div.textContent = 'A';
            div.outerHTML = '<section id="detached">B</section>';
            document.getElementById('result').textContent =
              div.outerHTML + ':' + div.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "<div id=\"detached\">A</div>:A")?;
    Ok(())
}

#[test]
fn element_outer_html_set_on_document_child_throws() -> Result<()> {
    let html = r#"
        <html><body><button id='run'>run</button></body></html>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.documentElement.outerHTML = '<html><body>X</body></html>';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("NoModificationAllowedError"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn element_outer_html_set_sanitizes_scripts_and_dangerous_attrs() -> Result<()> {
    let html = r#"
        <div id='box'>old</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.outerHTML =
              '<div id="box">' +
              '<script id="evil">document.getElementById("result").textContent = "pwned";</script>' +
              '<a id="link" href="javascript:alert(1)" onclick="alert(1)">safe</a>' +
              '</div>';
            const next = document.getElementById('box');
            const link = document.getElementById('link');
            document.getElementById('result').textContent =
              document.querySelectorAll('#evil').length + ':' +
              link.hasAttribute('onclick') + ':' +
              link.hasAttribute('href') + ':' +
              next.outerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "0:false:false:<div id=\"box\"><a id=\"link\">safe</a></div>",
    )?;
    Ok(())
}
