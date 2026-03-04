use super::*;

#[test]
fn document_dom_content_loaded_basic_usage_works() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const log = [];
          log.push('start:' + document.readyState);
          document.addEventListener('DOMContentLoaded', (event) => {
            log.push('event:' + event.type + ':' + event.cancelable + ':' + document.readyState);
            document.getElementById('result').textContent = log.join('|');
          });
          log.push('end:' + document.readyState);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "start:loading|end:loading|event:DOMContentLoaded:false:interactive",
    )?;
    Ok(())
}

#[test]
fn document_dom_content_loaded_ready_state_guard_pattern_works() -> Result<()> {
    let html = r#"
        <button id='late'>late</button>
        <p id='result'></p>
        <script>
          let setupCalls = 0;
          function setup() {
            setupCalls = setupCalls + 1;
          }

          if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', setup);
          } else {
            setup();
          }

          document.addEventListener('DOMContentLoaded', () => {
            document.getElementById('result').textContent =
              'init:' + setupCalls + ':' + document.readyState;
          });

          document.getElementById('late').addEventListener('click', () => {
            let lateCalls = 0;
            function lateSetup() {
              lateCalls = lateCalls + 1;
            }

            if (document.readyState === 'loading') {
              document.addEventListener('DOMContentLoaded', lateSetup);
            } else {
              lateSetup();
            }

            document.getElementById('result').textContent =
              'late:' + setupCalls + ':' + lateCalls + ':' + document.readyState;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.assert_text("#result", "init:1:interactive")?;
    h.click("#late")?;
    h.assert_text("#result", "late:1:1:complete")?;
    Ok(())
}

#[test]
fn document_dom_content_loaded_has_no_ondomcontentloaded_handler_property() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let addCalls = 0;
          let propCalls = 0;

          document.ondomcontentloaded = () => {
            propCalls = propCalls + 1;
          };

          document.addEventListener('DOMContentLoaded', () => {
            addCalls = addCalls + 1;
            document.getElementById('result').textContent =
              'add:' + addCalls + ':prop:' + propCalls;
          });
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "add:1:prop:0")?;
    Ok(())
}

#[test]
fn document_dom_content_loaded_fires_for_location_mock_page_navigation() -> Result<()> {
    let html = r#"
        <button id='go'>go</button>
        <script>
          document.getElementById('go').addEventListener('click', () => {
            location.assign('https://app.local/next');
          });
        </script>
        "#;

    let next_mock = r#"
        <p id='result'></p>
        <script>
          document.addEventListener('DOMContentLoaded', (event) => {
            document.getElementById('result').textContent =
              event.type + ':' + event.cancelable + ':' + document.readyState;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/next", next_mock);
    h.click("#go")?;
    h.assert_text("#result", "DOMContentLoaded:false:interactive")?;
    Ok(())
}
