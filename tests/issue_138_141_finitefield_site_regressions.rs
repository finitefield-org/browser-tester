use browser_tester::{Harness, KeyboardEventInit};

#[test]
fn issue_138_generic_format_two_args_is_not_hijacked_as_intl_relative_time() -> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const helper = {
          format(template, values) {
            return template
              .replace('{shown}', String(values.shown))
              .replace('{total}', String(values.total));
          }
        };

        function setStatus(text) {
          document.getElementById('out').textContent = text;
        }

        const shown = 3;
        const total = 8;
        setStatus(helper.format('Shown {shown}/{total}', { shown, total }));
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "Shown 3/8")?;
    Ok(())
}

#[test]
fn issue_139_function_can_reference_later_declared_const() -> browser_tester::Result<()> {
    let html = r#"
      <div id='out'></div>
      <script>
        function ensure() {
          return state.value;
        }

        const state = { value: 123 };
        document.getElementById('out').textContent = String(ensure());
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "123")?;
    Ok(())
}

#[test]
fn issue_140_nested_state_paths_are_not_treated_as_dom_element_variables() -> browser_tester::Result<()> {
    let html = r#"
      <button id='btn'>run</button>
      <p id='out'></p>
      <script>
        const state = {
          ratio: { mode: 'a' },
          measurements: [{ value: 1 }],
        };

        document.getElementById('btn').addEventListener('click', () => {
          state.ratio.mode = 'b';
          state.measurements[0].value = 2;
          document.getElementById('out').textContent = state.ratio.mode + ':' + String(state.measurements[0].value);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#btn")?;
    harness.assert_text("#out", "b:2")?;
    Ok(())
}

#[test]
fn issue_141_dispatch_keyboard_bubbles_to_delegated_listener() -> browser_tester::Result<()> {
    let html = r#"
      <div id='root'>
        <input id='field'>
      </div>
      <p id='out'></p>
      <script>
        document.getElementById('root').addEventListener('keydown', (event) => {
          document.getElementById('out').textContent = 'root:' + event.target.id + ':' + event.key;
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.dispatch_keyboard(
        "#field",
        "keydown",
        KeyboardEventInit {
            key: "Enter".to_string(),
            code: Some("Enter".to_string()),
            ..Default::default()
        },
    )?;
    harness.assert_text("#out", "root:field:Enter")?;
    Ok(())
}
