use super::*;

#[test]
fn element_set_html_unsafe_without_sanitizer_keeps_script_and_event_attrs() -> Result<()> {
    let html = r#"
        <div id='target'>old</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const returned = target.setHTMLUnsafe(
              "<p id='p' onclick='alert(1)'>P</p><script id='s'>window.__x = 1</" + "script>"
            );
            const p = target.querySelector('#p');
            const s = target.querySelector('#s');
            document.getElementById('result').textContent = [
              returned === undefined,
              p !== null,
              p !== null ? p.getAttribute('onclick') : 'none',
              s !== null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:alert(1):true")?;
    Ok(())
}

#[test]
fn element_set_html_unsafe_with_default_sanitizer_strips_xss_unsafe_markup() -> Result<()> {
    let html = r#"
        <div id='target'>old</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const returned = target.setHTMLUnsafe(
              "<p id='p' onclick='alert(1)'>P</p><script id='s'>window.__x = 1</" + "script>",
              { sanitizer: "default" }
            );
            const p = target.querySelector('#p');
            document.getElementById('result').textContent = [
              returned === undefined,
              p !== null,
              p !== null ? p.getAttribute('onclick') === null : false,
              target.querySelector('#s') === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:true")?;
    Ok(())
}

#[test]
fn element_set_html_unsafe_with_config_remove_elements_filters_matching_tags() -> Result<()> {
    let html = r#"
        <div id='target'>old</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const returned = target.setHTMLUnsafe(
              "<p id='p'>P</p><script id='s'>X</" + "script><button id='b'>B</button>",
              { sanitizer: { removeElements: ["script", "button"] } }
            );
            document.getElementById('result').textContent = [
              returned === undefined,
              target.querySelector('#p') !== null,
              target.querySelector('#s') === null,
              target.querySelector('#b') === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:true")?;
    Ok(())
}

#[test]
fn element_set_html_unsafe_creates_declarative_shadow_root() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const returned = host.setHTMLUnsafe(
              "<template shadowrootmode='open'><span id='inside'>I</span></template><i id='outside'>O</i>"
            );
            const root = host.shadowRoot;
            document.getElementById('result').textContent = [
              returned === undefined,
              root !== null,
              root !== null ? root.querySelector('#inside').textContent : 'none',
              host.querySelector('template') === null,
              host.querySelector('#outside') !== null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:I:true:true")?;
    Ok(())
}

#[test]
fn element_set_html_unsafe_second_declarative_template_becomes_template_in_shadow_root()
-> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            host.setHTMLUnsafe(
              "<template shadowrootmode='open'><span id='first'>F</span></template>" +
              "<template shadowrootmode='closed'><span id='second'>S</span></template>"
            );
            const root = host.shadowRoot;
            const moved = root ? root.querySelector("template[shadowrootmode='closed']") : null;
            document.getElementById('result').textContent = [
              root !== null,
              root !== null ? root.querySelector('#first') !== null : false,
              moved !== null,
              host.querySelector('template') === null
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true:true")?;
    Ok(())
}

#[test]
fn set_html_unsafe_table_html_8_5_2_13_2_6_4_9_wraps_direct_rows_in_implied_tbody() -> Result<()> {
    let html = r#"
        <table id='scores'><caption>before</caption></table>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const table = document.getElementById('scores');
            table.setHTMLUnsafe('<tr id="a"><td>Alpha</td></tr><tr id="b"><td>Beta</td></tr>');
            document.getElementById('result').textContent = [
              document.querySelectorAll('#scores > tbody').length,
              document.querySelectorAll('#scores > tr').length,
              document.querySelectorAll('#scores > tbody > tr').length,
              Array.from(document.querySelectorAll('#scores > tbody > tr > td'))
                .map((cell) => cell.textContent)
                .join(',')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:0:2:Alpha,Beta")?;
    Ok(())
}

#[test]
fn element_set_html_unsafe_rejects_invalid_sanitizer_string_and_invalid_config() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            let badString = false;
            let badConfig = false;
            try {
              target.setHTMLUnsafe('<p>x</p>', { sanitizer: 'unsafe' });
            } catch (e) {
              badString = String(e).includes('TypeError');
            }
            try {
              target.setHTMLUnsafe('<p>x</p>', {
                sanitizer: { elements: ['p'], removeElements: ['script'] }
              });
            } catch (e) {
              badConfig = String(e).includes('TypeError');
            }
            document.getElementById('result').textContent = [badString, badConfig].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true")?;
    Ok(())
}

#[test]
fn element_set_html_unsafe_rejects_wrong_argument_count() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('target').setHTMLUnsafe();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("setHTMLUnsafe requires one or two arguments"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}
