use super::*;

#[test]
fn submit_updates_result() -> Result<()> {
    let html = r#"
        <input id='name'>
        <input id='agree' type='checkbox'>
        <button id='submit'>Send</button>
        <p id='result'></p>
        <script>
          document.getElementById('submit').addEventListener('click', () => {
            const name = document.getElementById('name').value;
            const agree = document.getElementById('agree').checked;
            document.getElementById('result').textContent =
              agree ? `OK:${name}` : 'NG';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "Taro")?;
    h.set_checked("#agree", true)?;
    h.click("#submit")?;
    h.assert_text("#result", "OK:Taro")?;
    Ok(())
}

#[test]
fn mock_window_supports_multiple_pages() -> Result<()> {
    let mut win = MockWindow::new();
    win.open_page(
        "https://app.local/a",
        r#"
            <button id='btn'>A</button>
            <p id='result'></p>
            <script>
              document.getElementById('btn').addEventListener('click', () => {
                document.getElementById('result').textContent = 'A';
              });
            </script>
        "#,
    )?;

    win.open_page(
        "https://app.local/b",
        r#"
            <button id='btn'>B</button>
            <p id='result'></p>
            <script>
              document.getElementById('btn').addEventListener('click', () => {
                document.getElementById('result').textContent = 'B';
              });
            </script>
        "#,
    )?;

    win.switch_to("https://app.local/a")?;
    win.click("#btn")?;
    win.assert_text("#result", "A")?;

    win.switch_to("https://app.local/b")?;
    win.assert_text("#result", "")?;
    win.click("#btn")?;
    win.assert_text("#result", "B")?;

    win.switch_to("https://app.local/a")?;
    win.assert_text("#result", "A")?;
    Ok(())
}

#[test]
fn window_aliases_document_in_script_parser() -> Result<()> {
    let html = r#"
        <p id='result'>before</p>
        <script>
          window.document.getElementById('result').textContent = 'after';
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "after")?;
    Ok(())
}

#[test]
fn window_core_aliases_and_document_default_view_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent =
              (window === window.window) + ':' +
              (window === self) + ':' +
              (window === top) + ':' +
              (window === parent) + ':' +
              (window.frames === window) + ':' +
              window.length + ':' +
              window.closed + ':' +
              (window.clientInformation === window.navigator) + ':' +
              (clientInformation === navigator) + ':' +
              (window.document === document) + ':' +
              (document.defaultView === window) + ':' +
              window.origin + ':' +
              window.isSecureContext;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:true:true:true:0:false:true:true:true:true:null:false",
    )?;
    Ok(())
}

#[test]
fn window_origin_and_secure_context_follow_location_changes() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.assign('https://app.local/path');
            document.getElementById('result').textContent =
              window.origin + ':' + window.isSecureContext;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "https://app.local:true")?;
    Ok(())
}

#[test]
fn window_read_only_core_properties_are_rejected() {
    let err = Harness::from_html(
        r#"
        <script>
          window.closed = true;
        </script>
        "#,
    )
    .expect_err("window.closed should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.closed is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn html_entities_in_text_nodes_are_decoded() -> Result<()> {
    let html = "<p id='result'>&lt;A &amp; B&gt;&nbsp;&copy;</p>";
    let h = Harness::from_html(html)?;
    h.assert_text("#result", "<A & B>\u{00A0}©")?;
    Ok(())
}

#[test]
fn html_entities_in_attribute_values_are_decoded() -> Result<()> {
    let html = r#"
        <div id='result' data-value='a&amp;b&nbsp;&#x3c;'></div>
        <script>
          document.getElementById('result').textContent =
            document.getElementById('result').getAttribute('data-value');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "a&b\u{00A0}<")?;
    Ok(())
}

#[test]
fn html_entities_in_inner_html_are_decoded() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <p id='result'></p>
        <script>
          document.getElementById('host').innerHTML =
            '<span id="value">a&amp;b&nbsp;</span>';
          document.getElementById('result').textContent =
            document.getElementById('value').textContent;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "a&b\u{00A0}")?;
    Ok(())
}

#[test]
fn html_entities_without_trailing_semicolon_are_decoded() -> Result<()> {
    let html = "<p id='result'>&lt;A &amp B &gt C&copy D&thinsp;E&ensp;F&emsp;G&frac12;H</p>";

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "<A & B > C© D\u{2009}E\u{2002}F\u{2003}G½H")?;
    Ok(())
}

#[test]
fn html_entities_known_named_references_are_decoded() -> Result<()> {
    let html = "<p id='result'>&larr;&rarr;</p>";

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "←→")?;
    Ok(())
}

#[test]
fn html_entities_more_named_references_are_decoded() -> Result<()> {
    let html = "<p id='result'>&pound;&times;&divide;&laquo;&raquo;&frac13;&frac15;&frac16;&frac18;&frac23;&frac25;&frac34;&frac35;&frac38;&frac45;&frac56;&frac58;</p>";

    let h = Harness::from_html(html)?;
    h.assert_text(
            "#result",
            "\u{00A3}\u{00D7}\u{00F7}\u{00AB}\u{00BB}\u{2153}\u{2155}\u{2159}\u{215B}\u{2154}\u{2156}\u{00BE}\u{2157}\u{215C}\u{2158}\u{215A}\u{215E}",
        )?;
    Ok(())
}

#[test]
fn html_entities_unknown_reference_boundary_cases_are_preserved() -> Result<()> {
    let html = "<p id='result'>&frac12x;&frac34;&poundfoo;&pound;&frac12abc;</p>";

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "&frac12x;¾&poundfoo;£&frac12abc;")?;
    Ok(())
}

#[test]
fn html_entities_unknown_named_references_are_not_decoded() -> Result<()> {
    let html = "<p id='result'>&nopenvelope;&copy;</p>";

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "&nopenvelope;©")?;
    Ok(())
}

#[test]
fn html_entities_without_semicolon_hex_and_decimal_numeric_are_decoded() -> Result<()> {
    let html = "<p id='result'>&#38&#60&#x3C&#x3e</p>";

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "&<<>")?;
    Ok(())
}

#[test]
fn prevent_default_works_on_submit() -> Result<()> {
    let html = r#"
        <form id='f'>
          <button id='submit' type='submit'>Send</button>
        </form>
        <p id='result'></p>
        <script>
          document.getElementById('f').addEventListener('submit', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent = 'blocked';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#submit")?;
    h.assert_text("#result", "blocked")?;
    Ok(())
}

#[test]
fn form_elements_length_and_index_work() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' value='N'>
          <textarea id='bio'>B</textarea>
          <button id='ok' type='button'>OK</button>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('f');
            document.getElementById('result').textContent =
              form.elements.length + ':' +
              form.elements[0].id + ':' +
              form.elements[1].id + ':' +
              form.elements[2].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:name:bio:ok")?;
    Ok(())
}

#[test]
fn form_elements_index_supports_direct_property_access() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='a' value='X'>
          <input id='b' value='Y'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('f').elements[1].value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Y")?;
    Ok(())
}

#[test]
fn form_elements_index_supports_expression() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='a' value='X'>
          <input id='b' value='Y'>
          <input id='c' value='Z'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('f');
            const index = 1;
            const value = form.elements[index + 1].value;
            document.getElementById('result').textContent = value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Z")?;
    Ok(())
}

#[test]
fn form_elements_out_of_range_returns_runtime_error() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='a' value='X'>
        </form>
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('f').elements[5].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h.click("#btn").expect_err("out-of-range index should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("elements[5]"));
            assert!(msg.contains("returned null"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn textarea_initial_value_is_loaded_from_markup_text() -> Result<()> {
    let html = r#"
        <textarea id='bio' name='bio'>HELLO</textarea>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_value("#bio", "HELLO")?;
    Ok(())
}

#[test]
fn form_data_get_and_has_work_with_form_controls() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' name='name' value='Taro'>
          <input id='agree' name='agree' type='checkbox' checked>
          <input id='skip' name='skip' type='checkbox'>
          <input id='disabled' name='disabled' value='x' disabled>
          <button id='submit' name='submit' type='submit' value='go'>Go</button>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('f');
            const fd = new FormData(form);
            document.getElementById('result').textContent =
              fd.get('name') + ':' +
              fd.get('agree') + ':' +
              fd.has('skip') + ':' +
              fd.has('disabled') + ':' +
              fd.has('submit');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Taro:on:false:false:false")?;
    Ok(())
}

#[test]
fn form_data_uses_textarea_and_select_initial_values() -> Result<()> {
    let html = r#"
        <form id='f'>
          <textarea id='bio' name='bio'>HELLO</textarea>
          <select id='kind' name='kind'>
            <option id='k1' value='A'>Alpha</option>
            <option id='k2' selected>Beta</option>
          </select>
          <select id='city' name='city'>
            <option id='c1' value='tokyo'>Tokyo</option>
            <option id='c2' value='osaka'>Osaka</option>
          </select>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData(document.getElementById('f'));
            document.getElementById('result').textContent =
              fd.get('bio') + ':' + fd.get('kind') + ':' + fd.get('city');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "HELLO:Beta:tokyo")?;
    Ok(())
}

#[test]
fn form_data_reflects_option_selected_attribute_mutation() -> Result<()> {
    let html = r#"
        <form id='f'>
          <select id='kind' name='kind'>
            <option id='k1' selected value='A'>Alpha</option>
            <option id='k2' value='B'>Beta</option>
          </select>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('k1').removeAttribute('selected');
            document.getElementById('k2').setAttribute('selected', 'true');
            const fd = new FormData(document.getElementById('f'));
            document.getElementById('result').textContent = fd.get('kind');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "B")?;
    Ok(())
}

#[test]
fn select_value_assignment_updates_selected_option_and_form_data() -> Result<()> {
    let html = r#"
        <form id='f'>
          <select id='kind' name='kind'>
            <option id='k1' selected value='A'>Alpha</option>
            <option id='k2' value='B'>Beta</option>
          </select>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sel = document.getElementById('kind');
            sel.value = 'B';
            const fd = new FormData(document.getElementById('f'));
            document.getElementById('result').textContent =
              fd.get('kind') + ':' +
              document.getElementById('k1').hasAttribute('selected') + ':' +
              document.getElementById('k2').hasAttribute('selected') + ':' +
              sel.value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "B:false:true:B")?;
    Ok(())
}

#[test]
fn select_value_assignment_can_match_option_text_without_value_attribute() -> Result<()> {
    let html = r#"
        <form id='f'>
          <select id='kind' name='kind'>
            <option id='k1'>Alpha</option>
            <option id='k2'>Beta</option>
          </select>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sel = document.getElementById('kind');
            sel.value = 'Beta';
            const fd = new FormData(document.getElementById('f'));
            document.getElementById('result').textContent =
              fd.get('kind') + ':' +
              sel.value + ':' +
              document.getElementById('k1').hasAttribute('selected') + ':' +
              document.getElementById('k2').hasAttribute('selected');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Beta:Beta:false:true")?;
    Ok(())
}

#[test]
fn form_data_inline_constructor_call_works() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' name='name' value='Hanako'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              new FormData(document.getElementById('f')).get('name') + ':' +
              new FormData(document.getElementById('f')).has('missing') + ':' +
              new FormData(document.getElementById('f')).get('missing');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Hanako:false:")?;
    Ok(())
}

#[test]
fn form_data_get_all_length_and_append_work() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='t1' name='tag' value='A'>
          <input id='t2' name='tag' value='B'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData(document.getElementById('f'));
            fd.append('tag', 'C');
            fd.append('other', 123);
            document.getElementById('result').textContent =
              fd.get('tag') + ':' +
              fd.getAll('tag').length + ':' +
              fd.getAll('other').length + ':' +
              fd.get('other');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A:3:1:123")?;
    Ok(())
}

#[test]
fn form_data_get_all_length_inline_constructor_works() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='t1' name='tag' value='A'>
          <input id='t2' name='tag' value='B'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              new FormData(document.getElementById('f')).getAll('tag').length + ':' +
              new FormData(document.getElementById('f')).getAll('missing').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:0")?;
    Ok(())
}

#[test]
fn form_data_get_all_returns_array_values_in_order() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='t1' name='tag' value='A'>
          <input id='t2' name='tag' value='B'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData(document.getElementById('f'));
            fd.append('tag', 'C');
            const tags = fd.getAll('tag');
            const missing = fd.getAll('missing');
            document.getElementById('result').textContent =
              tags.length + ':' +
              tags[0] + ':' +
              tags[1] + ':' +
              tags[2] + ':' +
              tags.join('|') + ':' +
              missing.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:A:B:C:A|B|C:0")?;
    Ok(())
}

#[test]
fn form_data_get_all_inline_constructor_returns_array() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='t1' name='tag' value='A'>
          <input id='t2' name='tag' value='B'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const tags = new FormData(document.getElementById('f')).getAll('tag');
            document.getElementById('result').textContent =
              tags.length + ':' + tags[0] + ':' + tags[1];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:A:B")?;
    Ok(())
}

#[test]
fn form_data_method_on_non_form_data_variable_returns_runtime_error() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' name='name' value='Hanako'>
        </form>
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = document.getElementById('f');
            fd.get('name');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("non-FormData variable should fail on .get()");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("is not a FormData instance"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_data_get_all_on_non_form_data_variable_returns_runtime_error() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' name='name' value='Hanako'>
        </form>
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = document.getElementById('f');
            fd.getAll('name');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("non-FormData variable should fail on .getAll()");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("is not a FormData instance"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_data_append_on_non_form_data_variable_returns_runtime_error() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' name='name' value='Hanako'>
        </form>
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = document.getElementById('f');
            fd.append('k', 'v');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("non-FormData variable should fail on .append()");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("is not a FormData instance"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn stop_propagation_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn'>X</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            event.stopPropagation();
            document.getElementById('result').textContent = 'btn';
          });
          document.getElementById('root').addEventListener('click', () => {
            document.getElementById('result').textContent = 'root';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "btn")?;
    Ok(())
}

#[test]
fn capture_listeners_fire_in_expected_order() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='parent'>
            <button id='btn'>X</button>
          </div>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('root').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'R';
          }, true);
          document.getElementById('parent').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'P';
          }, true);
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'C';
          }, true);
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });
          document.getElementById('parent').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'p';
          });
          document.getElementById('root').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'r';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "RPCBpr")?;
    Ok(())
}

#[test]
fn remove_event_listener_respects_capture_flag() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'C';
          }, true);
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });

          document.getElementById('btn').removeEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'C';
          });
          document.getElementById('btn').removeEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          }, true);
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "CB")?;
    Ok(())
}

#[test]
fn trace_logs_capture_events_when_enabled() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {});
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enable_trace(true);
    h.click("#btn")?;

    let logs = h.take_trace_logs();
    assert!(logs.iter().any(|line| line.contains("[event] click")));
    assert!(logs.iter().any(|line| line.contains("phase=bubble")));
    assert!(h.take_trace_logs().is_empty());
    Ok(())
}

#[test]
fn trace_logs_collect_when_stderr_output_is_disabled() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {});
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enable_trace(true);
    h.set_trace_stderr(false);
    h.click("#btn")?;

    let logs = h.take_trace_logs();
    assert!(logs.iter().any(|line| line.contains("[event] click")));
    assert!(logs.iter().any(|line| line.contains("[event] done click")));
    Ok(())
}

#[test]
fn trace_categories_can_disable_timer_logs() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enable_trace(true);
    h.set_trace_stderr(false);
    h.set_trace_timers(false);
    h.click("#btn")?;

    let logs = h.take_trace_logs();
    assert!(logs.iter().any(|line| line.contains("[event] click")));
    assert!(logs.iter().all(|line| !line.contains("[timer]")));
    Ok(())
}

#[test]
fn trace_categories_can_disable_event_logs() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enable_trace(true);
    h.set_trace_stderr(false);
    h.set_trace_events(false);
    h.click("#btn")?;

    let logs = h.take_trace_logs();
    assert!(
        logs.iter()
            .any(|line| line.contains("[timer] schedule timeout id=1"))
    );
    assert!(logs.iter().all(|line| !line.contains("[event]")));
    Ok(())
}

#[test]
fn trace_logs_are_empty_when_trace_is_disabled() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {});
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert!(h.take_trace_logs().is_empty());
    Ok(())
}

#[test]
fn trace_logs_capture_timer_lifecycle_when_enabled() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enable_trace(true);
    h.click("#btn")?;

    let logs = h.take_trace_logs();
    assert!(
        logs.iter()
            .any(|line| line.contains("[timer] schedule timeout id=1"))
    );
    assert!(logs.iter().any(|line| line.contains("due_at=5")));
    assert!(logs.iter().any(|line| line.contains("delay_ms=5")));

    assert!(h.run_next_timer()?);
    let logs = h.take_trace_logs();
    assert!(logs.iter().any(|line| line.contains("[timer] run id=1")));
    assert!(logs.iter().any(|line| line.contains("now_ms=5")));
    Ok(())
}

#[test]
fn trace_logs_capture_timer_api_summaries() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 5);
            setTimeout(() => {}, 10);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enable_trace(true);
    h.set_trace_stderr(false);
    h.click("#btn")?;
    let _ = h.take_trace_logs();

    h.advance_time(5)?;
    let logs = h.take_trace_logs();
    assert!(
        logs.iter()
            .any(|line| line.contains("[timer] advance delta_ms=5 from=0 to=5 ran_due=1"))
    );

    assert_eq!(h.run_due_timers()?, 0);
    let logs = h.take_trace_logs();
    assert!(
        logs.iter()
            .any(|line| line.contains("[timer] run_due now_ms=5 ran=0"))
    );

    h.flush()?;
    let logs = h.take_trace_logs();
    assert!(
        logs.iter()
            .any(|line| line.contains("[timer] flush from=5 to=10 ran=1"))
    );
    Ok(())
}

#[test]
fn trace_log_limit_keeps_latest_entries() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enable_trace(true);
    h.set_trace_log_limit(2)?;
    h.dispatch("#btn", "alpha")?;
    h.dispatch("#btn", "beta")?;
    h.dispatch("#btn", "gamma")?;

    let logs = h.take_trace_logs();
    assert_eq!(logs.len(), 2);
    assert!(logs.iter().any(|line| line.contains("done beta")));
    assert!(logs.iter().any(|line| line.contains("done gamma")));
    assert!(logs.iter().all(|line| !line.contains("done alpha")));
    Ok(())
}

#[test]
fn set_trace_log_limit_rejects_zero() -> Result<()> {
    let html = r#"<button id='btn'>run</button>"#;
    let mut h = Harness::from_html(html)?;
    let err = h
        .set_trace_log_limit(0)
        .expect_err("zero trace log limit should be rejected");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("set_trace_log_limit requires at least 1 entry"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn trace_logs_event_done_contains_default_prevented_and_labels() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            event.preventDefault();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enable_trace(true);
    h.click("#btn")?;
    let logs = h.take_trace_logs();
    assert!(logs.iter().any(|line| line.contains("[event] click")));
    assert!(logs.iter().any(|line| line.contains("target=#btn")));
    assert!(
        logs.iter()
            .any(|line| line.contains("[event] done click")
                && line.contains("default_prevented=true"))
    );
    Ok(())
}

#[test]
fn query_selector_if_else_and_class_list_work() -> Result<()> {
    let html = r#"
        <div id='box' class='base'></div>
        <button id='btn'>toggle</button>
        <p id='result'></p>
        <script>
          document.querySelector('#btn').addEventListener('click', () => {
            if (document.querySelector('#box').classList.contains('active')) {
              document.querySelector('#result').textContent = 'active';
            } else {
              document.querySelector('#box').classList.add('active');
              document.querySelector('#result').textContent = 'activated';
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "activated")?;
    h.click("#btn")?;
    h.assert_text("#result", "active")?;
    Ok(())
}

#[test]
fn class_list_toggle_and_not_condition_work() -> Result<()> {
    let html = r#"
        <div id='badge' class='badge'></div>
        <button id='btn'>toggle</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.querySelector('#badge').classList.toggle('on');
            if (!document.querySelector('#badge').classList.contains('on')) {
              document.getElementById('result').textContent = 'off';
            } else {
              document.getElementById('result').textContent = 'on';
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "on")?;
    h.click("#btn")?;
    h.assert_text("#result", "off")?;
    Ok(())
}

#[test]
fn query_selector_all_index_and_length_work() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const second = document.querySelectorAll('.item')[1].textContent;
            document.getElementById('result').textContent =
              second + ':' + document.querySelectorAll('.item').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "B:2")?;
    Ok(())
}

#[test]
fn query_selector_all_node_list_variable_works() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const second = items[1].textContent;
            document.getElementById('result').textContent = items.length + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:B")?;
    Ok(())
}

#[test]
fn query_selector_all_index_supports_expression() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const index = 1;
            const next = items[index + 1].textContent;
            document.getElementById('result').textContent = items[index].textContent + ':' + next;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "B:C")?;
    Ok(())
}

#[test]
fn query_selector_all_list_index_after_reuse_works() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.querySelectorAll('.item');
            const picked = items[2];
            document.getElementById('result').textContent = picked.textContent + ':' + items.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "C:3")?;
    Ok(())
}

#[test]
fn get_elements_by_class_name_works() -> Result<()> {
    let html = r#"
        <ul>
          <li id='x' class='item target'>A</li>
          <li id='y' class='item'>B</li>
          <li id='z' class='target'>C</li>
          <li id='w' class='item target'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.getElementsByClassName('item target');
            document.getElementById('result').textContent = items.length + ':' + items[0].id + ':' + items[1].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:x:w")?;
    Ok(())
}

#[test]
fn get_elements_by_tag_name_works() -> Result<()> {
    let html = r#"
        <ul>
          <li id='a'>A</li>
          <li id='b'>B</li>
        </ul>
        <section id='s'>
          <li id='c'>C</li>
        </section>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const items = document.getElementsByTagName('li');
            document.getElementById('result').textContent = items.length + ':' + items[2].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:c")?;
    Ok(())
}

#[test]
fn get_elements_by_name_works() -> Result<()> {
    let html = r#"
        <input id='a' name='target' value='one'>
        <input id='b' name='other' value='other'>
        <input id='c' name='target' value='two'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fields = document.getElementsByName('target');
            document.getElementById('result').textContent = fields.length + ':' + fields[1].value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:two")?;
    Ok(())
}

#[test]
fn class_list_add_remove_multiple_arguments_work() -> Result<()> {
    let html = r#"
        <div id='box' class='base'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.classList.add('alpha', 'beta', 'gamma');
            box.classList.remove('base', 'gamma');
            document.getElementById('result').textContent = box.className;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "alpha beta")?;
    Ok(())
}

#[test]
fn member_chain_dom_targets_support_class_list_listener_and_open_property() -> Result<()> {
    let html = r#"
        <div id='dialog' class='panel hidden'></div>
        <button id='open-tool'>open</button>
        <details id='settings' open></details>
        <p id='result'></p>
        <script>
          const el = {
            dialog: document.getElementById('dialog'),
            openToolBtn: document.getElementById('open-tool'),
            settingsDetails: document.getElementById('settings'),
          };

          el.dialog.classList.remove('hidden');
          el.openToolBtn.addEventListener('click', () => {
            document.getElementById('result').textContent =
              el.dialog.className + ':' + (el.settingsDetails.open ? 'open' : 'closed');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#open-tool")?;
    h.assert_text("#result", "panel:open")?;
    Ok(())
}

#[test]
fn document_core_properties_and_collections_work() -> Result<()> {
    let html = r#"
        <html id='doc'>
          <head id='head'>
            <title>Initial</title>
          </head>
          <body id='body'>
            <form id='f'><input id='name'></form>
            <img id='logo' src='logo.png'>
            <a id='link' href='/x'>x</a>
            <button id='btn'>run</button>
            <p id='result'></p>
            <script>
              document.getElementById('btn').addEventListener('click', () => {
                const kids = document.children;
                const first = document.firstElementChild;
                const last = document.lastElementChild;
                const activeBeforeNode = document.activeElement;
                const activeBefore = activeBeforeNode ? activeBeforeNode.id : 'none';
                document.getElementById('name').focus();
                const activeAfterNode = document.activeElement;
                const activeAfter = activeAfterNode ? activeAfterNode.id : 'none';

                document.getElementById('result').textContent =
                  document.title + ':' +
                  document.characterSet + ':' +
                  document.compatMode + ':' +
                  document.contentType + ':' +
                  document.readyState + ':' +
                  document.referrer + ':' +
                  document.URL + ':' +
                  document.documentURI + ':' +
                  document.location + ':' +
                  document.location.href + ':' +
                  document.visibilityState + ':' +
                  document.hidden + ':' +
                  document.body.id + ':' +
                  document.head.id + ':' +
                  document.documentElement.id + ':' +
                  document.childElementCount + ':' +
                  kids.length + ':' +
                  first.id + ':' +
                  last.id + ':' +
                  document.forms.length + ':' +
                  document.images.length + ':' +
                  document.links.length + ':' +
                  document.scripts.length + ':' +
                  activeBefore + ':' +
                  activeAfter + ':' +
                  (document.defaultView ? 'yes' : 'no');
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Initial:UTF-8:CSS1Compat:text/html:complete::about:blank:about:blank:about:blank:about:blank:visible:false:body:head:doc:1:1:doc:doc:1:1:1:1:none:name:yes",
    )?;
    Ok(())
}

#[test]
fn document_title_assignment_and_body_chain_target_work() -> Result<()> {
    let html = r#"
        <html id='doc'>
          <head id='head'></head>
          <body id='body'>
            <button id='btn'>run</button>
            <p id='result'></p>
            <script>
              document.body.classList.add('ready');
              document.body.addEventListener('click', () => {});

              document.getElementById('btn').addEventListener('click', () => {
                document.title = 'Updated';
                const first = document.firstElementChild;
                const last = document.lastElementChild;
                document.getElementById('result').textContent =
                  document.title + ':' +
                  document.head.id + ':' +
                  document.documentElement.id + ':' +
                  document.body.className + ':' +
                  first.id + ':' +
                  last.id;
              });
            </script>
          </body>
        </html>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Updated:head:doc:ready:doc:doc")?;
    Ok(())
}

#[test]
fn location_properties_and_setters_work_from_location_document_and_window() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://developer.mozilla.org:8080/en-US/search?q=URL#search-results-close-container';
            document.location.protocol = 'http:';
            window.location.hostname = 'example.com';
            location.port = '9090';
            location.pathname = 'docs';
            location.search = 'k=v';
            location.hash = 'anchor';

            document.getElementById('result').textContent =
              location.href + '|' +
              location.protocol + '|' +
              location.host + '|' +
              location.hostname + '|' +
              location.port + '|' +
              location.pathname + '|' +
              location.search + '|' +
              location.hash + '|' +
              location.origin + '|' +
              document.location.toString() + '|' +
              window.location.toString() + '|' +
              location.ancestorOrigins.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "http://example.com:9090/docs?k=v#anchor|http:|example.com:9090|example.com|9090|/docs|?k=v|#anchor|http://example.com:9090|http://example.com:9090/docs?k=v#anchor|http://example.com:9090/docs?k=v#anchor|0",
    )?;
    Ok(())
}

#[test]
fn location_assign_replace_reload_and_navigation_logs_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.assign('https://app.local/a?x=1#h');
            location.replace('/b');
            location.reload();
            document.getElementById('result').textContent = location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "https://app.local/b")?;
    assert_eq!(h.location_reload_count(), 1);

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "about:blank".to_string(),
                to: "https://app.local/a?x=1#h".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Replace,
                from: "https://app.local/a?x=1#h".to_string(),
                to: "https://app.local/b".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Reload,
                from: "https://app.local/b".to_string(),
                to: "https://app.local/b".to_string(),
            },
        ]
    );
    assert!(h.take_location_navigations().is_empty());
    Ok(())
}

#[test]
fn location_mock_pages_load_on_navigation_and_reload() -> Result<()> {
    let html = r#"
        <button id='go'>go</button>
        <script>
          document.getElementById('go').addEventListener('click', () => {
            location.assign('https://app.local/next');
          });
        </script>
        "#;

    let first_mock = r#"
        <button id='reload'>reload</button>
        <p id='marker'>first</p>
        <script>
          document.getElementById('reload').addEventListener('click', () => {
            location.reload();
          });
        </script>
        "#;
    let second_mock = "<p id='marker'>second</p>";

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/next", first_mock);
    h.click("#go")?;
    h.assert_text("#marker", "first")?;

    h.set_location_mock_page("https://app.local/next", second_mock);
    h.click("#reload")?;
    h.assert_text("#marker", "second")?;
    assert_eq!(h.location_reload_count(), 1);

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "about:blank".to_string(),
                to: "https://app.local/next".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Reload,
                from: "https://app.local/next".to_string(),
                to: "https://app.local/next".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn hash_only_location_navigation_does_not_trigger_mock_page_swap() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'>alive</p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://app.local/path';
            location.hash = 'frag';
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("https://app.local/path#frag", "<p id='result'>swapped</p>");
    h.click("#run")?;
    h.assert_text("#result", "alive:https://app.local/path#frag")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "about:blank".to_string(),
                to: "https://app.local/path".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "https://app.local/path".to_string(),
                to: "https://app.local/path#frag".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn anchor_properties_and_to_string_work() -> Result<()> {
    let html = r#"
        <a id='link' href='/docs/page?x=1#intro'>hello</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://example.com/base/index.html?doc=1#docfrag';
            document.getElementById('link').download = 'report.txt';
            document.getElementById('link').hreflang = 'ja';
            document.getElementById('link').ping = 'https://p1.test https://p2.test';
            document.getElementById('link').referrerPolicy = 'no-referrer';
            document.getElementById('link').rel = 'noopener noreferrer';
            document.getElementById('link').target = '_blank';
            document.getElementById('link').type = 'text/plain';
            document.getElementById('link').attributionSrc = 'https://attr.test/src';
            document.getElementById('link').interestForElement = 'panel';
            document.getElementById('link').charset = 'utf-8';
            document.getElementById('link').coords = '0,0,10,10';
            document.getElementById('link').rev = 'prev';
            document.getElementById('link').shape = 'rect';

            document.getElementById('result').textContent =
              document.getElementById('link').href + '|' +
              document.getElementById('link').protocol + '|' +
              document.getElementById('link').host + '|' +
              document.getElementById('link').hostname + '|' +
              document.getElementById('link').port + '|' +
              document.getElementById('link').pathname + '|' +
              document.getElementById('link').search + '|' +
              document.getElementById('link').hash + '|' +
              document.getElementById('link').origin + '|' +
              document.getElementById('link').download + '|' +
              document.getElementById('link').hreflang + '|' +
              document.getElementById('link').ping + '|' +
              document.getElementById('link').referrerPolicy + '|' +
              document.getElementById('link').rel + '|' +
              document.getElementById('link').relList.length + '|' +
              document.getElementById('link').target + '|' +
              document.getElementById('link').type + '|' +
              document.getElementById('link').attributionSrc + '|' +
              document.getElementById('link').interestForElement + '|' +
              document.getElementById('link').charset + '|' +
              document.getElementById('link').coords + '|' +
              document.getElementById('link').rev + '|' +
              document.getElementById('link').shape + '|' +
              document.getElementById('link').toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/docs/page?x=1#intro|https:|example.com|example.com||/docs/page|?x=1|#intro|https://example.com|report.txt|ja|https://p1.test https://p2.test|no-referrer|noopener noreferrer|2|_blank|text/plain|https://attr.test/src|panel|utf-8|0,0,10,10|prev|rect|https://example.com/docs/page?x=1#intro",
    )?;
    Ok(())
}

#[test]
fn anchor_username_password_and_url_part_setters_work() -> Result<()> {
    let html = r#"
        <a id='cred' href='https://u:p@example.com:8443/p?q=1#h'>cred</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initial =
              document.getElementById('cred').username + ':' +
              document.getElementById('cred').password + ':' +
              document.getElementById('cred').host + ':' +
              document.getElementById('cred').origin;

            document.getElementById('cred').username = 'alice';
            document.getElementById('cred').password = 'secret';
            document.getElementById('cred').protocol = 'http:';
            document.getElementById('cred').hostname = 'api.example.test';
            document.getElementById('cred').port = '9090';
            document.getElementById('cred').pathname = 'docs';
            document.getElementById('cred').search = 'k=v';
            document.getElementById('cred').hash = 'frag';

            document.getElementById('result').textContent =
              initial + '|' +
              document.getElementById('cred').href + '|' +
              document.getElementById('cred').username + '|' +
              document.getElementById('cred').password + '|' +
              document.getElementById('cred').protocol + '|' +
              document.getElementById('cred').host + '|' +
              document.getElementById('cred').pathname + '|' +
              document.getElementById('cred').search + '|' +
              document.getElementById('cred').hash + '|' +
              document.getElementById('cred').origin + '|' +
              document.getElementById('cred').toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "u:p:example.com:8443:https://example.com:8443|http://alice:secret@api.example.test:9090/docs?k=v#frag|alice|secret|http:|api.example.test:9090|/docs|?k=v|#frag|http://api.example.test:9090|http://alice:secret@api.example.test:9090/docs?k=v#frag",
    )?;
    Ok(())
}

#[test]
fn anchor_text_alias_and_read_only_properties_work() -> Result<()> {
    let html = r#"
        <a id='link' href='https://example.com/start' rel='noopener'>old</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('link').text = 'Updated text';

            let originReadOnly = 'no';
            try {
              document.getElementById('link').origin = 'https://evil.example';
            } catch (e) {
              originReadOnly = 'yes';
            }

            let relListReadOnly = 'no';
            try {
              document.getElementById('link').relList = 'x';
            } catch (e) {
              relListReadOnly = 'yes';
            }

            document.getElementById('result').textContent =
              document.getElementById('link').textContent + ':' +
              document.getElementById('link').text + ':' +
              originReadOnly + ':' +
              relListReadOnly + ':' +
              document.getElementById('link').origin + ':' +
              document.getElementById('link').relList.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "Updated text:Updated text:yes:yes:https://example.com:1",
    )?;
    Ok(())
}

#[test]
fn history_properties_push_state_and_replace_state_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initialLen = history.length;
            const initialState = history.state === null ? 'null' : 'non-null';
            history.pushState({ step: 1 }, '', 'https://app.local/one');
            const pushed = history.length + ':' + history.state.step + ':' + location.href;
            history.replaceState({ step: 2 }, '', 'https://app.local/two');
            const replaced = history.length + ':' + history.state.step + ':' + location.href;
            document.getElementById('result').textContent =
              initialLen + ':' + initialState + '|' + pushed + '|' + replaced + '|' + window.history.scrollRestoration;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1:null|2:1:https://app.local/one|2:2:https://app.local/two|auto",
    )?;
    Ok(())
}

#[test]
fn history_back_forward_and_go_dispatch_popstate_with_state() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          window.addEventListener('popstate', (event) => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent +
              '[' + (event.state === null ? 'null' : event.state) + '@' + location.href + ']';
          });

          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = '';
            history.pushState('A', '', 'https://app.local/a');
            history.pushState('B', '', 'https://app.local/b');
            history.back();
            history.forward();
            history.go(-2);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "[A@https://app.local/a][B@https://app.local/b][null@about:blank]",
    )?;
    Ok(())
}

#[test]
fn history_out_of_bounds_navigation_is_noop() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            history.pushState('A', '', 'https://app.local/a');
            history.go(10);
            history.forward();
            history.go(-10);
            document.getElementById('result').textContent =
              history.length + ':' + history.state + ':' + location.href;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:A:https://app.local/a")?;
    Ok(())
}

#[test]
fn history_go_reload_works_with_location_mock_page() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            history.go();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_location_mock_page("about:blank", "<p id='marker'>reloaded</p>");
    h.click("#run")?;
    h.assert_text("#marker", "reloaded")?;
    assert_eq!(h.location_reload_count(), 1);
    Ok(())
}

#[test]
fn history_scroll_restoration_setter_and_window_history_access_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = window.history.scrollRestoration;
            history.scrollRestoration = 'manual';
            document.getElementById('result').textContent =
              before + ':' + history.scrollRestoration + ':' + window.history.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "auto:manual:1")?;
    Ok(())
}

#[test]
fn history_read_only_and_invalid_scroll_restoration_are_rejected() {
    let readonly_err = Harness::from_html(
        r#"
        <script>
          window.history.length = 2;
        </script>
        "#,
    )
    .expect_err("history.length should be read-only");
    match readonly_err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "history.length is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }

    let invalid_mode_err = Harness::from_html(
        r#"
        <script>
          history.scrollRestoration = 'smooth';
        </script>
        "#,
    )
    .expect_err("invalid scrollRestoration value should fail");
    match invalid_mode_err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "history.scrollRestoration must be 'auto' or 'manual'")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn document_has_focus_reports_active_element_state() -> Result<()> {
    let html = r#"
        <input id='name'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const before = document.hasFocus();
            document.getElementById('name').focus();
            const during = document.hasFocus();
            document.getElementById('name').blur();
            const after = document.hasFocus();
            document.getElementById('result').textContent = before + ':' + during + ':' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true:false")?;
    Ok(())
}

#[test]
fn document_body_chain_supports_query_selector_and_query_selector_all() -> Result<()> {
    let html = r#"
        <body>
          <div id='a' class='item'></div>
          <div id='b' class='item'></div>
          <button id='btn'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('btn').addEventListener('click', () => {
              const picked = document.body.querySelector('.item');
              const total = document.body.querySelectorAll('.item').length;
              picked.classList.remove('item');
              document.getElementById('result').textContent =
                picked.id + ':' + total + ':' + document.body.querySelectorAll('.item').length;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:2:1")?;
    Ok(())
}

#[test]
fn class_list_for_each_supports_single_arg_and_index() -> Result<()> {
    let html = r#"
        <div id='box' class='red green blue'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let joined = '';
            let indexes = '';
            document.getElementById('box').classList.forEach((name, index) => {
              joined = joined + name;
              indexes = indexes + index;
            });
            document.getElementById('result').textContent = joined + ':' + indexes;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "redgreenblue:012")?;
    Ok(())
}

#[test]
fn element_click_method_from_script_works() -> Result<()> {
    let html = r#"
        <button id='trigger'>click me</button>
        <input id='agree' type='checkbox'>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('agree').click();
            document.getElementById('result').textContent =
              (document.getElementById('agree').checked ? 'checked' : 'unchecked');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "checked")?;
    h.click("#trigger")?;
    h.assert_text("#result", "unchecked")?;
    Ok(())
}

#[test]
fn element_scroll_into_view_method_from_script_works() -> Result<()> {
    let html = r#"
        <button id='trigger'>scroll target</button>
        <section id='target'></section>
        <p id='result'></p>
        <script>
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('target').scrollIntoView();
            document.getElementById('result').textContent = 'done';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "done")?;
    Ok(())
}

#[test]
fn element_scroll_into_view_rejects_arguments() {
    let html = r#"
        <button id='trigger'>target</button>
        <script>
          document.getElementById('trigger').scrollIntoView('smooth');
        </script>
        "#;

    let err = match Harness::from_html(html) {
        Ok(_) => panic!("scrollIntoView should reject arguments"),
        Err(err) => err,
    };

    match err {
        Error::ScriptParse(msg) => {
            assert_eq!(
                msg,
                "scrollIntoView takes no arguments: document.getElementById('trigger').scrollIntoView('smooth')"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn form_submit_method_dispatches_submit_event() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' value='default'>
        </form>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('f').addEventListener('submit', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent =
              event.type + ':' + event.isTrusted + ':' + event.currentTarget.id;
          });
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('f').submit();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "submit:true:f")?;
    Ok(())
}

#[test]
fn form_reset_method_dispatches_reset_and_restores_defaults() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' value='default'>
          <input id='agree' type='checkbox' checked>
        </form>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          let marker = '';
          document.getElementById('f').addEventListener('reset', () => {
            marker = marker + 'reset';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            document.getElementById('name').value = 'changed';
            document.getElementById('agree').checked = false;
            document.getElementById('f').reset();
            document.getElementById('result').textContent =
              marker + ':' +
              document.getElementById('name').value + ':' +
              (document.getElementById('agree').checked ? 'on' : 'off');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "reset:default:on")?;
    Ok(())
}

#[test]
fn dialog_show_modal_close_and_toggle_events_work() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <button id='close' type='button'>Close</button>
          <form method='dialog' id='form'>
            <p>
              <label for='fav-animal'>Favorite animal:</label>
              <select id='fav-animal' name='favAnimal' required>
                <option></option>
                <option>Brine shrimp</option>
                <option>Red panda</option>
                <option>Spider monkey</option>
              </select>
            </p>
            <button id='submit' type='submit'>Confirm</button>
          </form>
        </dialog>
        <button id='open'>Open dialog</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let logs = '';

          dialog.addEventListener('beforetoggle', (event) => {
            logs = logs + 'before:' + event.oldState + '>' + event.newState + '|';
          });
          dialog.addEventListener('toggle', (event) => {
            logs = logs + 'toggle:' + event.newState + '|';
          });
          dialog.addEventListener('close', () => {
            logs = logs + 'close:' + dialog.returnValue + '|';
          });

          document.getElementById('open').addEventListener('click', () => {
            dialog.showModal();
            dialog.close('Red panda');
            document.getElementById('result').textContent = logs + 'open=' + dialog.open;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#open")?;
    h.assert_text(
        "#result",
        "before:closed>open|toggle:open|before:open>closed|toggle:closed|close:Red panda|open=false",
    )?;
    Ok(())
}

#[test]
fn dialog_request_close_fires_cancel_and_can_be_prevented() -> Result<()> {
    let html = r#"
        <dialog id='dialog' open></dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let marker = '';
          dialog.addEventListener('cancel', (event) => {
            marker = marker + 'cancel:' + dialog.returnValue;
            dialog.returnValue = '';
            event.preventDefault();
          });
          dialog.addEventListener('close', () => {
            marker = marker + '|close';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            dialog.returnValue = 'seed';
            dialog.requestClose('next');
            document.getElementById('result').textContent =
              marker + '|open=' + dialog.open + '|value=' + dialog.returnValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "cancel:next|open=true|value=")?;
    Ok(())
}

#[test]
fn dialog_form_method_dialog_closes_and_keeps_submit_return_value() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <form id='form' method='dialog'>
            <select id='fav-animal' required>
              <option></option>
              <option>Brine shrimp</option>
              <option>Red panda</option>
              <option>Spider monkey</option>
            </select>
            <button id='submit' type='submit'>Confirm</button>
          </form>
        </dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          const form = document.getElementById('form');
          const select = document.getElementById('fav-animal');

          form.addEventListener('submit', () => {
            dialog.returnValue = select.value;
          });
          dialog.addEventListener('close', () => {
            document.getElementById('result').textContent =
              dialog.returnValue + ':' + dialog.open;
          });

          document.getElementById('trigger').addEventListener('click', () => {
            dialog.show();
            select.value = 'Spider monkey';
            document.getElementById('submit').click();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "Spider monkey:false")?;
    Ok(())
}

#[test]
fn dialog_form_submit_is_blocked_when_required_control_is_empty() -> Result<()> {
    let html = r#"
        <dialog id='dialog'>
          <form id='form' method='dialog'>
            <select id='fav-animal' required>
              <option></option>
              <option>Brine shrimp</option>
            </select>
            <button id='submit' type='submit'>Confirm</button>
          </form>
        </dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          let marker = 'none';
          document.getElementById('form').addEventListener('submit', () => {
            marker = 'submitted';
          });
          dialog.addEventListener('close', () => {
            marker = marker + '|closed';
          });
          document.getElementById('trigger').addEventListener('click', () => {
            dialog.showModal();
            document.getElementById('submit').click();
            document.getElementById('result').textContent = marker + ':' + dialog.open;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "none:true")?;
    Ok(())
}

#[test]
fn dialog_closed_by_property_reflects_closedby_attribute() -> Result<()> {
    let html = r#"
        <dialog id='dialog' closedby='none'></dialog>
        <button id='trigger'>run</button>
        <p id='result'></p>
        <script>
          const dialog = document.getElementById('dialog');
          document.getElementById('trigger').addEventListener('click', () => {
            const before = dialog.closedBy;
            dialog.closedBy = 'any';
            document.getElementById('result').textContent =
              before + ':' + dialog.closedBy + ':' + dialog.getAttribute('closedby');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#trigger")?;
    h.assert_text("#result", "none:any:any")?;
    Ok(())
}

#[test]
fn element_matches_method_works() -> Result<()> {
    let html = r#"
        <div id='container'>
          <button id='target' class='item primary'></button>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const direct = document.getElementById('target').matches('#target.item');
            const byTag = document.getElementById('target').matches('button');
            const bySelectorMismatch = document.getElementById('target').matches('.secondary');
            document.getElementById('result').textContent =
              direct + ':' + byTag + ':' + bySelectorMismatch;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn element_closest_method_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <section id='scope'>
            <div id='container'>
              <button id='btn'>run</button>
            </div>
          </section>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const scoped = document.getElementById('btn').closest('section');
            const selfMatch = document.getElementById('btn').closest('#btn');
            document.getElementById('result').textContent =
              scoped.id + ':' + selfMatch.id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "scope:btn")?;
    Ok(())
}

#[test]
fn element_closest_method_returns_null_when_not_found() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const matched = document.getElementById('btn').closest('section');
            document.getElementById('result').textContent = matched ? 'found' : 'missing';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "missing")?;
    Ok(())
}

#[test]
fn query_selector_all_foreach_and_element_variables_work() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.querySelectorAll('.item').forEach((item, idx) => {
              item.setAttribute('data-idx', idx);
              item.classList.toggle('picked', idx === 1);
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + item.textContent + item.getAttribute('data-idx');
            });
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + document.querySelectorAll('.item')[1].classList.contains('picked');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A0B1:true")?;
    Ok(())
}

#[test]
fn query_selector_all_foreach_single_arg_callback_works() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
        document.querySelectorAll('.item').forEach(item => {
              item.classList.add('seen');
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + item.textContent;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn parse_for_each_callback_accepts_arrow_expression_body() -> Result<()> {
    let (item_var, index_var, body) = parse_for_each_callback("item => 1")?;
    assert_eq!(item_var, "item");
    assert!(index_var.is_none());
    assert_eq!(body.len(), 1);
    match body
        .first()
        .expect("callback body should include one statement")
    {
        Stmt::Expr(Expr::Number(value)) => assert_eq!(*value, 1),
        other => panic!("unexpected callback body stmt: {other:?}"),
    }
    Ok(())
}

#[test]
fn listener_arrow_expression_callback_body_executes() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click',
            () => 1
          );
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "")?;
    Ok(())
}

#[test]
fn for_of_loop_supports_query_selector_all() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let output = '';
            for (const item of document.querySelectorAll('.item')) {
              output = output + item.textContent;
            }
            document.getElementById('result').textContent = output;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABC")?;
    Ok(())
}

#[test]
fn for_in_loop_supports_query_selector_all_indexes() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let output = '';
            for (let index in document.querySelectorAll('.item')) {
              output = output + index + ',';
            }
            document.getElementById('result').textContent = output;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0,1,2,")?;
    Ok(())
}

#[test]
fn for_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (let i = 0; i < 5; i = i + 1) {
              if (i === 0) {
                continue;
              }
              if (i === 3) {
                break;
              }
              out = out + i;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12")?;
    Ok(())
}

#[test]
fn while_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            let i = 0;
            while (i < 5) {
              i = i + 1;
              if (i === 1) {
                continue;
              }
              if (i === 4) {
                break;
              }
              out = out + i;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "23")?;
    Ok(())
}

#[test]
fn do_while_executes_at_least_once() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let count = 0;
            do {
              count = count + 1;
            } while (false);
            document.getElementById('result').textContent = count;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn do_while_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            let out = '';
            do {
              i = i + 1;
              if (i === 1) {
                continue;
              }
              if (i === 4) {
                break;
              }
              out = out + i;
            } while (i < 5);
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "23")?;
    Ok(())
}

#[test]
fn foreach_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
          <li class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            document.querySelectorAll('.item').forEach((item, idx) => {
              if (idx === 0) {
                continue;
              }
              if (idx === 2) {
                break;
              }
              out = out + idx;
            });
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn for_in_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
          <li class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (let index in document.querySelectorAll('.item')) {
              if (index === 1) {
                continue;
              }
              if (index === 3) {
                break;
              }
              out = out + index;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "02")?;
    Ok(())
}

#[test]
fn for_of_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
          <li class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (const item of document.querySelectorAll('.item')) {
              if (item.textContent === 'B') {
                continue;
              }
              if (item.textContent === 'D') {
                break;
              }
              out = out + item.textContent;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AC")?;
    Ok(())
}

#[test]
fn foreach_supports_nested_if_else_and_event_variable() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.querySelectorAll('.item').forEach((item, idx) => {
              if (idx === 1) {
                if (event.target.id === 'btn') {
                  item.classList.add('mid');
                } else {
                  item.classList.add('other');
                }
              } else {
                item.classList.add('edge');
              }
            });
            document.getElementById('result').textContent =
              document.querySelectorAll('.edge').length + ':' +
              document.querySelectorAll('.mid').length + ':' +
              event.currentTarget.id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:1:btn")?;
    Ok(())
}

#[test]
fn if_without_braces_with_else_on_next_statement_works() -> Result<()> {
    let html = r#"
        <input id='agree' type='checkbox'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            if (document.getElementById('agree').checked) document.getElementById('result').textContent = 'yes';
            else document.getElementById('result').textContent = 'no';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "no")?;
    h.set_checked("#agree", true)?;
    h.click("#btn")?;
    h.assert_text("#result", "yes")?;
    Ok(())
}

#[test]
fn if_block_and_following_statement_without_semicolon_are_split() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let text = '';
            if (true) {
              text = 'A';
            }
            text += 'B';
            document.getElementById('result').textContent = text;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn while_block_and_following_statement_without_semicolon_are_split() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let count = 0;
            let n = 0;
            while (n < 2) {
              count = count + 1;
              n = n + 1;
            }
            count = count + 10;
            document.getElementById('result').textContent = count;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12")?;
    Ok(())
}

#[test]
fn for_block_and_following_statement_without_semicolon_are_split() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let sum = 0;
            for (let i = 0; i < 3; i = i + 1) {
              sum = sum + i;
            } sum = sum + 10;
            document.getElementById('result').textContent = sum;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "13")?;
    Ok(())
}

#[test]
fn if_block_and_following_statement_without_space_are_split() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let text = '';
            if (true) {
              text = 'A';
            } if (true) {
              text = text + 'B';
            }
            document.getElementById('result').textContent = text;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn for_loop_post_increment_with_function_callback_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', function() {
            let sum = 0;
            for (let i = 0; i < 3; i++) {
              sum = sum + i;
            }
            document.getElementById('result').textContent = sum;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn try_catch_catches_runtime_error_and_binds_exception() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = 'init';
            try {
              nonExistentFunction();
              out = 'not-caught';
            } catch (error) {
              out = typeof error + ':' + (error ? 'y' : 'n');
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "string:y")?;
    Ok(())
}

#[test]
fn try_catch_finally_and_rethrow_behavior_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            try {
              try {
                throw 'oops';
              } catch (ex) {
                out = out + 'inner:' + ex;
                throw ex;
              } finally {
                out = out + ':finally';
              }
            } catch (ex) {
              out = out + ':outer:' + ex;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "inner:oops:finally:outer:oops")?;
    Ok(())
}

#[test]
fn try_finally_runs_without_catch_and_finally_return_masks_try_return() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function doIt() {
            try {
              return 1;
            } finally {
              return 2;
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            let out = 'start';
            try {
              try {
                throw 'boom';
              } finally {
                out = out + ':inner-finally';
              }
            } catch (e) {
              out = out + ':outer-catch:' + e;
            }
            document.getElementById('result').textContent = doIt() + ':' + out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:start:inner-finally:outer-catch:boom")?;
    Ok(())
}

#[test]
fn catch_without_binding_and_pattern_binding_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function isValidJSON(text) {
            try {
              JSON.parse(text);
              return true;
            } catch {
              return false;
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            let out = isValidJSON('{"a":1}') + ':' + isValidJSON('{bad}');
            try {
              throw { name: 'TypeError', message: 'oops' };
            } catch ({ name, message }) {
              out = out + ':' + name + ':' + message;
            }
            try {
              throw ['A', 'B'];
            } catch ([first, second]) {
              out = out + ':' + first + second;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:TypeError:oops:AB")?;
    Ok(())
}

#[test]
fn promise_then_function_callback_runs_as_microtask() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', function() {
            const result = document.getElementById('result');
            result.textContent = 'A';
            Promise.resolve().then(function() {
              result.textContent = result.textContent + 'P';
            });
            setTimeout(function() {
              result.textContent = result.textContent + 'T';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AP")?;
    h.flush()?;
    h.assert_text("#result", "APT")?;
    Ok(())
}

#[test]
fn promise_direct_then_chain_parses_and_runs() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.resolve('A')
              .then((value) => value + 'B')
              .then((value) => {
                result.textContent = value;
              })
              .catch((reason) => {
                result.textContent = 'ERR:' + reason;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn promise_constructor_resolves_via_timer() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            new Promise((resolve) => {
              setTimeout(() => {
                resolve('done');
              }, 0);
            }).then((value) => {
              result.textContent = value;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.flush()?;
    h.assert_text("#result", "done")?;
    Ok(())
}

#[test]
fn promise_catch_and_finally_chain_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.reject('E')
              .catch((reason) => {
                result.textContent = reason;
                return 'recovered';
              })
              .finally(() => {
                result.textContent = result.textContent + 'F';
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "EF")?;
    Ok(())
}

#[test]
fn promise_finally_waits_for_returned_promise() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.resolve('A')
              .finally(() => {
                return new Promise((resolve) => {
                  setTimeout(() => resolve('x'), 0);
                });
              })
              .then((value) => {
                result.textContent = value;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.flush()?;
    h.assert_text("#result", "A")?;
    Ok(())
}

#[test]
fn promise_with_resolvers_can_be_used_externally() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const bag = Promise.withResolvers();
            const resolveBag = bag.resolve;
            const rejectBag = bag.reject;

            bag.promise
              .then((value) => {
                result.textContent = 'ok:' + value;
              })
              .catch((reason) => {
                result.textContent = 'ng:' + reason;
              });

            resolveBag('A');
            rejectBag('B');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok:A")?;
    Ok(())
}

#[test]
fn promise_all_resolves_values_in_input_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.all([Promise.resolve('A'), 2]).then((values) => {
              result.textContent = values.join(',');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A,2")?;
    Ok(())
}

#[test]
fn promise_all_settled_returns_outcome_objects() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.allSettled([Promise.resolve('A'), Promise.reject('B')]).then((values) => {
              result.textContent = JSON.stringify(values);
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        r#"[{"status":"fulfilled","value":"A"},{"status":"rejected","reason":"B"}]"#,
    )?;
    Ok(())
}

#[test]
fn promise_any_rejects_with_aggregate_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.any([Promise.reject('E1'), Promise.reject('E2')]).catch((reason) => {
              const errors = reason.errors;
              result.textContent = reason.name + ':' + errors.join(',');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AggregateError:E1,E2")?;
    Ok(())
}

#[test]
fn promise_race_settles_with_first_settled_value() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.race([
              Promise.resolve('fast'),
              new Promise((resolve) => {
                setTimeout(() => resolve('slow'), 0);
              })
            ]).then((value) => {
              result.textContent = value;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "fast")?;
    Ok(())
}

#[test]
fn promise_try_wraps_sync_return_and_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.try(() => 'ok')
              .then((value) => {
                result.textContent = value;
                return Promise.try(() => missingVar);
              })
              .catch((reason) => {
                if (reason.includes('unknown variable')) {
                  result.textContent = result.textContent + ':caught';
                } else {
                  result.textContent = 'unexpected:' + reason;
                }
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok:caught")?;
    Ok(())
}

#[test]
fn promise_constructor_requires_new_keyword() {
    let err = Harness::from_html("<script>Promise(() => {});</script>")
        .expect_err("Promise without new should throw");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Promise constructor must be called with new"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn arrow_function_value_can_be_called() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const fn = (value) => {
              result.textContent = value;
            };
            fn('A');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A")?;
    Ok(())
}

#[test]
fn iife_arrow_function_expression_can_be_called() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          (() => {
            document.getElementById('result').textContent = 'ok';
          })();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn default_function_parameters_apply_for_missing_or_undefined() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function multiply(a, b = 1) {
            return a * b;
          }

          function test(num = 1) {
            return typeof num;
          }

          document.getElementById('result').textContent =
            multiply(5, 2) + ':' +
            multiply(5) + ':' +
            multiply(5, undefined) + ':' +
            test() + ':' +
            test(undefined) + ':' +
            test('') + ':' +
            test(null);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "10:5:5:number:number:string:object")?;
    Ok(())
}

#[test]
fn default_function_parameters_are_evaluated_left_to_right_at_call_time() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function greet(name, greeting, message = `${greeting} ${name}`) {
            return message;
          }

          function append(value, array = []) {
            array.push(value);
            return array.length;
          }

          const exprFn = function(a = 2, b = a + 1) {
            return a + b;
          };

          const arrowFn = (a, b = a + 5) => {
            return a + ':' + b;
          };

          document.getElementById('result').textContent =
            greet('David', 'Hi') + ':' +
            greet('David', 'Hi', 'Happy Birthday!') + ':' +
            append(1) + ':' +
            append(2) + ':' +
            exprFn(undefined, undefined) + ':' +
            exprFn(5) + ':' +
            arrowFn(7);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "Hi David:Happy Birthday!:1:1:5:11:7:12")?;
    Ok(())
}

#[test]
fn arrow_function_parameters_support_rest_and_destructuring() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const sum = (a, b, ...rest) => a + b + rest[0] + rest[1];
          const pick = ({ x, y: z }) => x + ':' + z;
          const pair = ([m, n] = [9, 8]) => m + ':' + n;
          document.getElementById('result').textContent =
            sum(1, 2, 3, 4) + '|' +
            pick({ x: 'A', y: 'B' }) + '|' +
            pair() + '|' +
            pair([5, 6]);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "10|A:B|9:8|5:6")?;
    Ok(())
}

#[test]
fn async_arrow_function_expressions_are_supported() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const add = async (a, b) => a + b;
            const inc = async value => {
              return value + 1;
            };
            Promise.all([add(1, 2), inc(4)]).then((values) => {
              document.getElementById('result').textContent = values.join(':');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "3:5")?;
    Ok(())
}

#[test]
fn async_identifier_arrow_form_still_parses_as_normal_arrow() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const fn = async => async + 1;
          document.getElementById('result').textContent = String(fn(3));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "4")?;
    Ok(())
}

#[test]
fn arrow_function_line_break_before_arrow_is_rejected() {
    let err = Harness::from_html(
        "<script>const fn = (a, b)\n=> a + b; document.body.textContent = String(fn(1, 2));</script>",
    )
    .expect_err("line break between parameter list and arrow should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("=>") || msg.contains("unsupported")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn default_parameter_initializer_cannot_access_function_body_bindings() {
    let err = Harness::from_html(
        "<script>function f(a = go()) { function go() { return ':P'; } } f();</script>",
    )
    .expect_err("default parameter initializer should not see function body bindings");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("unknown variable: go")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn class_list_toggle_force_argument_works() -> Result<()> {
    let html = r#"
        <input id='force' type='checkbox'>
        <div id='box' class='base'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').classList.toggle('active', document.getElementById('force').checked);
            if (document.getElementById('box').classList.contains('active'))
              document.getElementById('result').textContent = 'active';
            else
              document.getElementById('result').textContent = 'inactive';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "inactive")?;
    h.set_checked("#force", true)?;
    h.click("#btn")?;
    h.assert_text("#result", "active")?;
    h.set_checked("#force", false)?;
    h.click("#btn")?;
    h.assert_text("#result", "inactive")?;
    Ok(())
}

#[test]
fn logical_and_relational_and_strict_operators_work() -> Result<()> {
    let html = r#"
        <input id='age' value='25'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const age = document.getElementById('age').value;
            const okRange = age >= 20 && age < 30;
            if ((okRange === true && age !== '40') || age === '18')
              document.getElementById('result').textContent = 'pass';
            else
              document.getElementById('result').textContent = 'fail';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "pass")?;
    h.type_text("#age", "40")?;
    h.click("#btn")?;
    h.assert_text("#result", "fail")?;
    h.type_text("#age", "18")?;
    h.click("#btn")?;
    h.assert_text("#result", "pass")?;
    Ok(())
}

#[test]
fn dom_properties_and_attribute_methods_work() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').setAttribute('data-x', 'v1');
            document.getElementById('box').className = 'a b';
            document.getElementById('box').id = 'box2';
            document.getElementById('box2').name = 'named';
            const x = document.getElementById('box2').getAttribute('data-x');
            document.getElementById('result').textContent =
              document.getElementById('box2').name + ':' + document.getElementById('box2').className + ':' + x;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_exists("#box2")?;
    h.assert_text("#result", "named:a b:v1")?;
    Ok(())
}

#[test]
fn dataset_property_read_write_works() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').dataset.userId = 'u42';
            document.getElementById('box').dataset.planType = 'pro';
            document.getElementById('result').textContent =
              document.getElementById('box').dataset.userId + ':' +
              document.getElementById('box').getAttribute('data-user-id') + ':' +
              document.getElementById('box').dataset.planType;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "u42:u42:pro")?;
    Ok(())
}

#[test]
fn disabled_property_read_write_works() -> Result<()> {
    let html = r#"
        <input id='name' value='init'>
        <button id='toggle'>toggle-disabled</button>
        <button id='enable'>enable</button>
        <p id='result'></p>
        <script>
          document.getElementById('toggle').addEventListener('click', () => {
            document.getElementById('name').disabled = true;
            document.getElementById('result').textContent =
              document.getElementById('name').disabled + ':' +
              document.getElementById('name').getAttribute('disabled');
          });
          document.getElementById('enable').addEventListener('click', () => {
            document.getElementById('name').disabled = false;
            document.getElementById('result').textContent =
              document.getElementById('name').disabled + ':' +
              document.getElementById('name').getAttribute('disabled');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#toggle")?;
    h.assert_text("#result", "true:true")?;
    h.click("#enable")?;
    h.assert_text("#result", "false:")?;
    Ok(())
}

#[test]
fn readonly_property_read_write_and_type_text_is_ignored() -> Result<()> {
    let html = r#"
        <input id='name' value='init' readonly>
        <button id='make-editable'>editable</button>
        <button id='confirm'>confirm</button>
        <p id='result'></p>
        <script>
          document.getElementById('make-editable').addEventListener('click', () => {
            document.getElementById('name').readonly = false;
            document.getElementById('result').textContent = document.getElementById('name').readonly;
          });
          document.getElementById('confirm').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('name').readonly + ':' +
              document.getElementById('name').value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "changed")?;
    h.assert_value("#name", "init")?;
    h.click("#make-editable")?;
    h.type_text("#name", "changed")?;
    h.assert_value("#name", "changed")?;
    h.click("#confirm")?;
    h.assert_text("#result", "false:changed")?;
    Ok(())
}

#[test]
fn required_property_read_write_works() -> Result<()> {
    let html = r#"
        <input id='name' required>
        <button id='unset'>unset</button>
        <button id='set'>set</button>
        <p id='result'></p>
        <script>
          document.getElementById('set').addEventListener('click', () => {
            document.getElementById('name').required = true;
            document.getElementById('result').textContent = document.getElementById('name').required;
          });
          document.getElementById('unset').addEventListener('click', () => {
            document.getElementById('name').required = false;
            document.getElementById('result').textContent = document.getElementById('name').required;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#unset")?;
    h.assert_text("#result", "false")?;
    h.click("#set")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn style_property_read_write_works() -> Result<()> {
    let html = r#"
        <div id='box' style='color: blue;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').style.backgroundColor = 'red';
            document.getElementById('box').style.color = '';
            document.getElementById('result').textContent =
              document.getElementById('box').style.backgroundColor + ':' +
              document.getElementById('box').style.color + ':' +
              document.getElementById('box').getAttribute('style');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "red::background-color: red;")?;
    Ok(())
}

#[test]
fn offset_and_scroll_properties_are_read_only_and_queryable() -> Result<()> {
    let html = r#"
        <div id='box' style='width: 120px; height: 90px;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.offsetWidth + ':' + box.offsetHeight + ':' +
              box.offsetTop + ':' + box.offsetLeft + ':' +
              box.scrollWidth + ':' + box.scrollHeight + ':' +
              box.scrollTop + ':' + box.scrollLeft;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:0:0:0:0:0:0:0")?;
    Ok(())
}

#[test]
fn offset_property_assignment_is_rejected() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').scrollTop = 10;
            document.getElementById('box').offsetWidth = 100;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("scrollTop/offsetWidth assignment should fail");
    assert!(format!("{err}").contains("is read-only"));
    Ok(())
}

#[test]
fn dataset_camel_case_mapping_works() -> Result<()> {
    let html = r#"
        <div id='box' data-user-id='u1' data-plan-type='starter'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.dataset.accountStatus = 'active';
            document.getElementById('result').textContent =
              box.dataset.userId + ':' +
              box.dataset.planType + ':' +
              box.getAttribute('data-account-status');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "u1:starter:active")?;
    Ok(())
}

#[test]
fn focus_and_blur_update_active_element_and_events() -> Result<()> {
    let html = r#"
        <input id='a'>
        <input id='b'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const a = document.getElementById('a');
          const b = document.getElementById('b');
          let order = '';

          a.addEventListener('focus', () => {
            order += 'aF';
          });
          a.addEventListener('blur', () => {
            order += 'aB';
          });
          b.addEventListener('focus', () => {
            order += 'bF';
          });
          b.addEventListener('blur', () => {
            order += 'bB';
          });

          document.getElementById('btn').addEventListener('click', () => {
            a.focus();
            b.focus();
            b.blur();
            document.getElementById('result').textContent =
              order + ':' + (document.activeElement === null ? 'none' : 'active');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "aFaBbFbB:none")?;
    Ok(())
}

#[test]
fn focus_in_and_focus_out_events_are_dispatched() -> Result<()> {
    let html = r#"
        <input id='a'>
        <input id='b'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const a = document.getElementById('a');
          const b = document.getElementById('b');
          let order = '';

          a.addEventListener('focusin', () => {
            order += 'aI';
          });
          a.addEventListener('focus', () => {
            order += 'aF';
          });
          a.addEventListener('focusout', () => {
            order += 'aO';
          });
          a.addEventListener('blur', () => {
            order += 'aB';
          });

          b.addEventListener('focusin', () => {
            order += 'bI';
          });
          b.addEventListener('focus', () => {
            order += 'bF';
          });
          b.addEventListener('focusout', () => {
            order += 'bO';
          });
          b.addEventListener('blur', () => {
            order += 'bB';
          });

          document.getElementById('btn').addEventListener('click', () => {
            a.focus();
            b.focus();
            b.blur();
            document.getElementById('result').textContent =
              order + ':' + (document.activeElement === null ? 'none' : 'active');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "aIaFaOaBbIbFbObB:none")?;
    Ok(())
}

#[test]
fn html_dom_input_event_toggles_submit_button_disabled_state() -> Result<()> {
    let html = r#"
        <form action='' method='get'>
          <input type='text' id='userName'>
          <input type='submit' value='Send' id='sendButton'>
        </form>
        <p id='result'></p>
        <script>
          const nameField = document.getElementById('userName');
          const sendButton = document.getElementById('sendButton');

          sendButton.disabled = true;

          nameField.addEventListener('input', (event) => {
            const elem = event.target;
            const valid = elem.value.length !== 0;

            if (valid && sendButton.disabled) {
              sendButton.disabled = false;
            } else if (!valid && !sendButton.disabled) {
              sendButton.disabled = true;
            }

            document.getElementById('result').textContent =
              sendButton.disabled ? 'disabled' : 'enabled';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#userName", "Kaz")?;
    h.assert_text("#result", "enabled")?;
    h.type_text("#userName", "")?;
    h.assert_text("#result", "disabled")?;
    Ok(())
}

#[test]
fn html_element_hidden_and_inner_text_properties_work() -> Result<()> {
    let html = r#"
        <div id='box'>Hello <span>DOM</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const box = document.getElementById('box');
          document.getElementById('run').addEventListener('click', () => {
            const before = box.hidden + ':' + box.hasAttribute('hidden');

            box.hidden = true;
            const hiddenState = box.hidden + ':' + box.hasAttribute('hidden');

            box.hidden = false;
            box.innerText = 'Replaced';

            document.getElementById('result').textContent =
              before + '|' +
              hiddenState + '|' +
              box.innerText + '|' +
              box.textContent + '|' +
              box.hasAttribute('hidden');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:false|true:true|Replaced|Replaced|false")?;
    Ok(())
}

#[test]
fn document_hidden_remains_read_only_while_element_hidden_is_writable() -> Result<()> {
    let html = r#"
        <div id='box'>x</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let docErr = '';
            try {
              document.hidden = true;
            } catch (e) {
              docErr = '' + e;
            }
            const box = document.getElementById('box');
            box.hidden = true;
            document.getElementById('result').textContent =
              docErr + '|' + box.hidden + ':' + box.hasAttribute('hidden');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "hidden is read-only|true:true")?;
    Ok(())
}

#[test]
fn focus_skips_disabled_element() -> Result<()> {
    let html = r#"
        <input id='name' disabled>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('name').focus();
            document.getElementById('result').textContent = document.activeElement ? 'has' : 'none';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "none")?;
    Ok(())
}

#[test]
fn selector_focus_and_focus_within_runtime() -> Result<()> {
    let html = r#"
        <div id='scope'>
          <input id='child'>
        </div>
        <input id='outside'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const child = document.getElementById('child');
            const outside = document.getElementById('outside');
            child.focus();
            const before = document.querySelector('input:focus').id + ':' +
              (document.querySelectorAll('#scope:focus-within').length ? 'yes' : 'no');
            outside.focus();
            const after = document.querySelector('input:focus').id + ':' +
              (document.querySelectorAll('#scope:focus-within').length ? 'yes' : 'no');
            document.getElementById('result').textContent = before + ':' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "child:yes:outside:no")?;
    Ok(())
}

#[test]
fn selector_active_is_set_during_click_and_cleared_after() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const during = document.querySelectorAll('#btn:active').length ? 'yes' : 'no';
            setTimeout(() => {
              const after = document.querySelectorAll('#btn:active').length ? 'yes' : 'no';
              document.getElementById('result').textContent = during + ':' + after;
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.advance_time(0)?;
    h.assert_text("#result", "yes:no")?;
    Ok(())
}

#[test]
fn active_element_assignment_is_read_only() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.activeElement = null;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("activeElement should be read-only");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("read-only"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn style_empty_value_removes_attribute_when_last_property() -> Result<()> {
    let html = r#"
        <div id='box' style='color: blue;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.style.color = '';
            document.getElementById('result').textContent =
              box.getAttribute('style') === '' ? 'none' : 'some';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "none")?;
    Ok(())
}

#[test]
fn style_overwrite_updates_existing_declaration_without_duplicate() -> Result<()> {
    let html = r#"
        <div id='box' style='color: blue; border-color: black;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.style.color = 'red';
            box.style.backgroundColor = 'white';
            document.getElementById('result').textContent = box.getAttribute('style');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "color: red; border-color: black; background-color: white;",
    )?;
    Ok(())
}

#[test]
fn get_computed_style_property_value_works() -> Result<()> {
    let html = r#"
        <div id='box' style='color: blue; background-color: transparent;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.style.color = 'red';
            const color = getComputedStyle(box).getPropertyValue('color');
            const missing = getComputedStyle(box).getPropertyValue('padding-top');
            document.getElementById('result').textContent = color + ':' + missing;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "red:")?;
    Ok(())
}

#[test]
fn style_parser_supports_quoted_colon_and_semicolon() -> Result<()> {
    let html = r#"
        <div id='box' style='content: "a:b;c"; color: blue;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.style.content + ':' + box.style.color + ':' + box.getAttribute('style');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "\"a:b;c\":blue:content: \"a:b;c\"; color: blue;")?;
    Ok(())
}

#[test]
fn style_parser_supports_parentheses_values() -> Result<()> {
    let html = r#"
        <div id='box' style='background-image: url("a;b:c"); font-family: Arial, sans-serif;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.style.backgroundImage + ':' + box.style.fontFamily + ':' + box.getAttribute('style');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "url(\"a;b:c\"):Arial, sans-serif:background-image: url(\"a;b:c\"); font-family: Arial, sans-serif;",
        )?;
    Ok(())
}

#[test]
fn element_reference_expression_assignment_works() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            const box = document.getElementById('box');
            const second = document.querySelectorAll('.item')[1];
            box.textContent = second.textContent + ':' + event.target.id;
            box.dataset.state = 'ok';
            document.getElementById('result').textContent =
              box.dataset.state + ':' + box.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok:B:btn")?;
    Ok(())
}

#[test]
fn event_properties_and_stop_immediate_propagation_work() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn'>run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.getElementById('result').textContent =
              event.type + ':' + event.target.id + ':' + event.currentTarget.id;
            event.stopImmediatePropagation();
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'second';
          });
          document.getElementById('root').addEventListener('click', () => {
            document.getElementById('result').textContent = 'root';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "click:btn:btn")?;
    Ok(())
}

#[test]
fn event_trusted_and_target_subproperties_are_accessible() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn' name='target-name'>run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent =
              event.isTrusted + ':' +
              event.target.name + ':' +
              event.currentTarget.name;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:target-name:target-name")?;
    Ok(())
}

#[test]
fn event_bubbles_and_cancelable_properties_are_available() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn' name='target-name'>run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.getElementById('result').textContent =
              event.bubbles + ':' + event.cancelable + ':' + event.isTrusted;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true")?;
    Ok(())
}

#[test]
fn dispatch_event_origin_is_untrusted_and_supports_event_methods() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='box'></div>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('root').addEventListener('custom', (event) => {
            document.getElementById('result').textContent = 'root:' + event.target.id;
          });
          document.getElementById('box').addEventListener('custom', (event) => {
            event.preventDefault();
            event.stopPropagation();
            document.getElementById('result').textContent =
              event.isTrusted + ':' + event.defaultPrevented;
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').dispatchEvent('custom');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true")?;
    Ok(())
}

#[test]
fn event_default_prevented_property_reflects_prevent_default() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.getElementById('result').textContent =
              event.defaultPrevented + ',';
            event.preventDefault();
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + event.defaultPrevented;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false,true")?;
    Ok(())
}

#[test]
fn event_phase_and_timestamp_are_available_in_handler() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn'>run</button>
        </div>
        <p id='result'></p>
        <script>
          let phases = '';
          document.getElementById('root').addEventListener('click', (event) => {
            phases = phases + (phases === '' ? '' : ',') + event.eventPhase + ':' + event.timeStamp;
          }, true);
          document.getElementById('btn').addEventListener('click', (event) => {
            phases = phases + ',' + event.eventPhase + ':' + event.timeStamp;
          }, true);
          document.getElementById('btn').addEventListener('click', (event) => {
            phases = phases + ',' + event.eventPhase + ':' + event.timeStamp;
          });
          document.getElementById('root').addEventListener('click', (event) => {
            phases = phases + ',' + event.eventPhase + ':' + event.timeStamp;
            document.getElementById('result').textContent = phases;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:0,2:0,2:0,3:0")?;
    Ok(())
}

#[test]
fn remove_event_listener_works_for_matching_handler() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'A';
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });
          document.getElementById('btn').removeEventListener('click', () => {
            document.getElementById('result').textContent = 'A';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "B")?;
    Ok(())
}

#[test]
fn dispatch_event_statement_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('box').addEventListener('custom', (event) => {
            document.getElementById('result').textContent =
              event.type + ':' + event.target.id + ':' + event.currentTarget.id;
          });
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.dispatchEvent('custom');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "custom:box:box")?;
    Ok(())
}

#[test]
fn dynamic_add_event_listener_inside_handler_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.addEventListener('custom', () => {
              document.getElementById('result').textContent = 'ok';
            });
            box.dispatchEvent('custom');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn dynamic_remove_event_listener_inside_handler_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('box').addEventListener('custom', () => {
            document.getElementById('result').textContent = 'A';
          });
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.removeEventListener('custom', () => {
              document.getElementById('result').textContent = 'A';
            });
            box.dispatchEvent('custom');
            if (document.getElementById('result').textContent === '')
              document.getElementById('result').textContent = 'none';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "none")?;
    Ok(())
}

#[test]
fn set_timeout_runs_on_flush_and_captures_env() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = 'A';
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A")?;
    h.flush()?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn timer_arguments_support_additional_parameters_and_comments() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            // comment: schedule timer with extra arg and inline delay comment
            setTimeout((message) => {
              document.getElementById('result').textContent = message;
            }, 5, 'ok');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(4)?;
    h.assert_text("#result", "")?;
    h.advance_time(1)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn timer_callback_supports_multiple_parameters() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout((first, second, third) => {
              document.getElementById('result').textContent =
                first + ':' + second + ':' + third;
            }, 5, 'A', 'B', 'C');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(5)?;
    h.assert_text("#result", "A:B:C")?;
    Ok(())
}

#[test]
fn timer_callback_assigns_undefined_for_missing_arguments() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout((first, second, third) => {
              document.getElementById('result').textContent =
                first + ':' + second + ':' + third;
            }, 5, 'only');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(5)?;
    h.assert_text("#result", "only:undefined:undefined")?;
    Ok(())
}

#[test]
fn timer_function_reference_supports_additional_parameters() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const onTimeout = (value) => {
            document.getElementById('result').textContent = value;
          };
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(onTimeout, 5, 'ref');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(5)?;
    h.assert_text("#result", "ref")?;
    Ok(())
}

#[test]
fn timer_interval_function_reference_supports_additional_parameters() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          let count = 0;
          const onTick = (value) => {
            count = count + 1;
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + value;
            if (count === 2) {
              clearInterval(intervalId);
            }
          };
          let intervalId = 0;
          document.getElementById('btn').addEventListener('click', () => {
            intervalId = setInterval(onTick, 5, 'tick');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(11)?;
    h.assert_text("#result", "ticktick")?;
    h.advance_time(10)?;
    h.assert_text("#result", "ticktick")?;
    Ok(())
}

#[test]
fn timer_interval_supports_multiple_additional_parameters() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          let id = 0;
          document.getElementById('btn').addEventListener('click', () => {
            let tick = 0;
            id = setInterval((value, suffix) => {
              tick = tick + 1;
              document.getElementById('result').textContent =
                document.getElementById('result').textContent + value + suffix;
              if (tick > 2) {
                clearInterval(id);
              }
            }, 0, 'I', '!');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "I!I!I!")?;
    Ok(())
}

#[test]
fn line_and_block_comments_are_ignored_in_script_parser() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          // top level comment
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'A'; // inline comment
            /* block comment */
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn run_due_timers_runs_only_currently_due_tasks() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert_eq!(h.now_ms(), 0);

    let ran = h.run_due_timers()?;
    assert_eq!(ran, 1);
    assert_eq!(h.now_ms(), 0);
    h.assert_text("#result", "A")?;

    let ran = h.run_due_timers()?;
    assert_eq!(ran, 0);
    h.assert_text("#result", "A")?;
    Ok(())
}

#[test]
fn run_due_timers_returns_zero_for_empty_queue() -> Result<()> {
    let html = r#"<button id='btn'>run</button>"#;
    let mut h = Harness::from_html(html)?;
    assert_eq!(h.run_due_timers()?, 0);
    Ok(())
}

#[test]
fn clear_timer_cancels_specific_pending_timer() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 5);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert!(h.clear_timer(1));
    assert!(!h.clear_timer(1));
    assert!(!h.clear_timer(999));

    h.advance_time(0)?;
    h.assert_text("#result", "B")?;
    h.advance_time(10)?;
    h.assert_text("#result", "B")?;
    Ok(())
}

#[test]
fn clear_all_timers_empties_pending_queue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = 'A';
            }, 0);
            setInterval(() => {
              result.textContent = 'B';
            }, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert_eq!(h.pending_timers().len(), 2);
    assert_eq!(h.clear_all_timers(), 2);
    assert!(h.pending_timers().is_empty());
    h.flush()?;
    h.assert_text("#result", "")?;
    Ok(())
}

#[test]
fn run_next_due_timer_runs_only_one_due_task_without_advancing_clock() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert_eq!(h.now_ms(), 0);

    assert!(h.run_next_due_timer()?);
    assert_eq!(h.now_ms(), 0);
    h.assert_text("#result", "A")?;

    assert!(!h.run_next_due_timer()?);
    assert_eq!(h.now_ms(), 0);
    h.assert_text("#result", "A")?;
    Ok(())
}

#[test]
fn run_next_due_timer_returns_false_for_empty_queue() -> Result<()> {
    let html = r#"<button id='btn'>run</button>"#;
    let mut h = Harness::from_html(html)?;
    assert!(!h.run_next_due_timer()?);
    Ok(())
}

#[test]
fn pending_timers_returns_due_ordered_snapshot() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {}, 10);
            setInterval(() => {}, 5);
            setTimeout(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    let timers = h.pending_timers();
    assert_eq!(
        timers,
        vec![
            PendingTimer {
                id: 3,
                due_at: 0,
                order: 2,
                interval_ms: None,
            },
            PendingTimer {
                id: 2,
                due_at: 5,
                order: 1,
                interval_ms: Some(5),
            },
            PendingTimer {
                id: 1,
                due_at: 10,
                order: 0,
                interval_ms: None,
            },
        ]
    );
    Ok(())
}

#[test]
fn pending_timers_reflects_advance_time_execution() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 5);
            setTimeout(() => {}, 7);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.advance_time(5)?;

    let timers = h.pending_timers();
    assert_eq!(
        timers,
        vec![
            PendingTimer {
                id: 2,
                due_at: 7,
                order: 1,
                interval_ms: None,
            },
            PendingTimer {
                id: 1,
                due_at: 10,
                order: 2,
                interval_ms: Some(5),
            },
        ]
    );
    Ok(())
}

#[test]
fn run_next_timer_executes_single_task_in_due_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 10);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + 'C';
            }, 10);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    assert_eq!(h.now_ms(), 0);

    assert!(h.run_next_timer()?);
    assert_eq!(h.now_ms(), 0);
    h.assert_text("#result", "B")?;

    assert!(h.run_next_timer()?);
    assert_eq!(h.now_ms(), 10);
    h.assert_text("#result", "BA")?;

    assert!(h.run_next_timer()?);
    assert_eq!(h.now_ms(), 10);
    h.assert_text("#result", "BAC")?;

    assert!(!h.run_next_timer()?);
    assert_eq!(h.now_ms(), 10);
    Ok(())
}

#[test]
fn advance_time_to_runs_due_timers_until_target() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 5);
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 10);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.advance_time_to(7)?;
    assert_eq!(h.now_ms(), 7);
    h.assert_text("#result", "A")?;

    h.advance_time_to(10)?;
    assert_eq!(h.now_ms(), 10);
    h.assert_text("#result", "AB")?;

    h.advance_time_to(10)?;
    assert_eq!(h.now_ms(), 10);
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn advance_time_to_rejects_past_target() -> Result<()> {
    let html = r#"<button id='btn'>run</button>"#;
    let mut h = Harness::from_html(html)?;
    h.advance_time(3)?;
    let err = h
        .advance_time_to(2)
        .expect_err("advance_time_to with past target should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("advance_time_to requires target >= now_ms"));
            assert!(msg.contains("target=2"));
            assert!(msg.contains("now_ms=3"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn set_timeout_respects_delay_order_and_nested_queueing() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + '1';
            }, 10);
            setTimeout(() => {
              result.textContent = result.textContent + '0';
              setTimeout(() => {
                result.textContent = result.textContent + 'N';
              });
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.flush()?;
    h.assert_text("#result", "0N1")?;
    Ok(())
}

#[test]
fn queue_microtask_runs_after_synchronous_task_body() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = 'A';
            queueMicrotask(() => {
              result.textContent = result.textContent + 'B';
            });
            result.textContent = result.textContent + 'C';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ACB")?;
    Ok(())
}

#[test]
fn promise_then_microtask_runs_before_next_timer() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = 'A';
            Promise.resolve().then(() => {
              result.textContent = result.textContent + 'P';
            });
            setTimeout(() => {
              result.textContent = result.textContent + 'T';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AP")?;
    h.flush()?;
    h.assert_text("#result", "APT")?;
    Ok(())
}

#[test]
fn fake_time_advance_controls_timer_execution() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            setTimeout(() => {
              result.textContent = result.textContent + '0';
            }, 0);
            setTimeout(() => {
              result.textContent = result.textContent + '1';
            }, 10);
            setTimeout(() => {
              result.textContent = result.textContent + '2';
            }, 20);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    assert_eq!(h.now_ms(), 0);

    h.advance_time(0)?;
    h.assert_text("#result", "0")?;
    assert_eq!(h.now_ms(), 0);

    h.advance_time(9)?;
    h.assert_text("#result", "0")?;
    assert_eq!(h.now_ms(), 9);

    h.advance_time(1)?;
    h.assert_text("#result", "01")?;
    assert_eq!(h.now_ms(), 10);

    h.advance_time(10)?;
    h.assert_text("#result", "012")?;
    assert_eq!(h.now_ms(), 20);
    Ok(())
}

#[test]
fn fake_time_advance_runs_interval_ticks_by_due_time() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const id = setInterval(() => {
              result.textContent = result.textContent + 'I';
              if (result.textContent === 'III') clearInterval(id);
            }, 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;

    h.advance_time(4)?;
    h.assert_text("#result", "")?;

    h.advance_time(1)?;
    h.assert_text("#result", "I")?;

    h.advance_time(10)?;
    h.assert_text("#result", "III")?;

    h.advance_time(100)?;
    h.assert_text("#result", "III")?;
    Ok(())
}

#[test]
fn date_now_uses_fake_clock_for_handlers_and_timers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = Date.now() + ':';
            setTimeout(() => {
              result.textContent = result.textContent + Date.now();
            }, 10);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.advance_time(7)?;
    h.click("#btn")?;
    h.assert_text("#result", "7:")?;

    h.advance_time(9)?;
    h.assert_text("#result", "7:")?;

    h.advance_time(1)?;
    h.assert_text("#result", "7:17")?;
    Ok(())
}

#[test]
fn date_now_with_flush_advances_to_timer_due_time() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = Date.now();
            setTimeout(() => {
              result.textContent = result.textContent + ':' + Date.now();
            }, 25);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0")?;
    h.flush()?;
    h.assert_text("#result", "0:25")?;
    assert_eq!(h.now_ms(), 25);
    Ok(())
}

#[test]
fn performance_now_uses_fake_clock_for_handlers_and_timers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = performance.now() + ':' + window.performance.now();
            setTimeout(() => {
              result.textContent = result.textContent + ':' + performance.now();
            }, 10);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.advance_time(7)?;
    h.click("#btn")?;
    h.assert_text("#result", "7:7")?;

    h.advance_time(9)?;
    h.assert_text("#result", "7:7")?;

    h.advance_time(1)?;
    h.assert_text("#result", "7:7:17")?;
    Ok(())
}

#[test]
fn date_constructor_and_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const nowDate = new Date();
            const fromNumber = new Date(1000);
            const parsed = Date.parse('1970-01-01T00:00:02Z');
            const utc = Date.UTC(1970, 0, 1, 0, 0, 3);
            const parsedViaWindow = window.Date.parse('1970-01-01');
            document.getElementById('result').textContent =
              nowDate.getTime() + ':' + fromNumber.getTime() + ':' +
              parsed + ':' + utc + ':' + parsedViaWindow;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.advance_time(42)?;
    h.click("#btn")?;
    h.assert_text("#result", "42:1000:2000:3000:0")?;
    Ok(())
}

#[test]
fn date_instance_methods_and_set_time_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const d = new Date('2024-03-05T01:02:03Z');
            const y = d.getFullYear();
            const m = d.getMonth();
            const day = d.getDate();
            const h = d.getHours();
            const min = d.getMinutes();
            const s = d.getSeconds();
            const iso = d.toISOString();
            const updated = d.setTime(Date.UTC(1970, 0, 2, 3, 4, 5));
            const iso2 = d.toISOString();
            document.getElementById('result').textContent =
              y + ':' + m + ':' + day + ':' + h + ':' + min + ':' + s +
              '|' + iso + '|' + updated + '|' + iso2;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "2024:2:5:1:2:3|2024-03-05T01:02:03.000Z|97445000|1970-01-02T03:04:05.000Z",
    )?;
    Ok(())
}

#[test]
fn date_parse_invalid_input_returns_nan_and_utc_normalizes_overflow() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parsedValue = Date.parse('invalid-date');
            const isInvalid = isNaN(parsedValue);
            const ts = Date.UTC(2020, 12, 1, 25, 61, 61);
            const normalizedDate = new Date(ts);
            const normalized = normalizedDate.toISOString();
            document.getElementById('result').textContent =
              isInvalid + ':' + normalized;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:2021-01-02T02:02:01.000Z")?;
    Ok(())
}

#[test]
fn date_method_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>new Date(1, 2);</script>",
            "new Date supports zero or one argument",
        ),
        (
            "<script>Date.parse();</script>",
            "Date.parse requires exactly one argument",
        ),
        (
            "<script>Date.UTC(1970);</script>",
            "Date.UTC requires between 2 and 7 arguments",
        ),
        (
            "<script>Date.UTC(1970, , 1);</script>",
            "Date.UTC argument cannot be empty",
        ),
        (
            "<script>const d = new Date(); d.getTime(1);</script>",
            "getTime does not take arguments",
        ),
        (
            "<script>const d = new Date(); d.setTime();</script>",
            "setTime requires exactly one argument",
        ),
        (
            "<script>const d = new Date(); d.toISOString(1);</script>",
            "toISOString does not take arguments",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail to parse");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains(expected), "expected '{expected}' in '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn math_constants_and_symbol_to_string_tag_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.round(Math.E * 1000) + ':' +
              Math.round(Math.LN10 * 1000) + ':' +
              Math.round(Math.LN2 * 1000) + ':' +
              Math.round(Math.LOG10E * 1000) + ':' +
              Math.round(Math.LOG2E * 1000) + ':' +
              Math.round(Math.PI * 1000) + ':' +
              Math.round(Math.SQRT1_2 * 1000) + ':' +
              Math.round(Math.SQRT2 * 1000) + ':' +
              (window.Math.PI === Math.PI) + ':' +
              Math[Symbol.toStringTag];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2718:2303:693:434:1443:3142:707:1414:true:Math")?;
    Ok(())
}

#[test]
fn math_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.abs(-3.5) + ':' +
              Math.acos(1) + ':' +
              Math.acosh(1) + ':' +
              Math.round(Math.asin(1) * 1000) + ':' +
              Math.asinh(0) + ':' +
              Math.round(Math.atan(1) * 1000) + ':' +
              Math.round(Math.atan2(1, 1) * 1000) + ':' +
              Math.round(Math.atanh(0.5) * 1000000) + ':' +
              Math.cbrt(27) + ':' +
              Math.ceil(1.2) + ':' +
              Math.clz32(1) + ':' +
              Math.cos(0) + ':' +
              Math.cosh(0) + ':' +
              Math.round(Math.exp(1) * 1000) + ':' +
              Math.round(Math.expm1(1) * 1000) + ':' +
              Math.floor(1.8) + ':' +
              Math.round(Math.f16round(1.337) * 1000000) + ':' +
              Math.round(Math.fround(1.337) * 1000000) + ':' +
              Math.hypot(3, 4) + ':' +
              Math.imul(2147483647, 2) + ':' +
              Math.round(Math.log(Math.E) * 1000) + ':' +
              Math.log10(1000) + ':' +
              Math.round(Math.log1p(1) * 1000) + ':' +
              Math.log2(8) + ':' +
              Math.max(1, 5, 3) + ':' +
              Math.min(1, 5, 3) + ':' +
              Math.pow(2, 8) + ':' +
              Math.round(1.5) + ':' +
              Math.round(-1.5) + ':' +
              Math.sign(-3) + ':' +
              Math.round(Math.sin(Math.PI / 2) * 1000) + ':' +
              Math.sinh(0) + ':' +
              Math.sqrt(9) + ':' +
              Math.sumPrecise([1, 2, 3]) + ':' +
              Math.tan(0) + ':' +
              Math.tanh(0) + ':' +
              Math.trunc(-1.9) + ':' +
              isNaN(Math.sign(NaN)) + ':' +
              Math.hypot() + ':' +
              isFinite(Math.max()) + ':' +
              isFinite(Math.min());
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "3.5:0:0:1571:0:785:785:549306:3:2:31:1:1:2718:1718:1:1336914:1337000:5:-2:1000:3:693:3:5:1:256:2:-1:-1:1000:0:3:6:0:0:-1:true:0:false:false",
        )?;
    Ok(())
}

#[test]
fn math_sum_precise_requires_array_argument() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Math.sumPrecise(1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("Math.sumPrecise should reject non-array argument");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Math.sumPrecise argument must be an array"))
        }
        other => panic!("unexpected Math.sumPrecise error: {other:?}"),
    }
    Ok(())
}

#[test]
fn math_method_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>Math.abs();</script>",
            "Math.abs requires exactly one argument",
        ),
        (
            "<script>Math.random(1);</script>",
            "Math.random does not take arguments",
        ),
        (
            "<script>Math.atan2(1);</script>",
            "Math.atan2 requires exactly two arguments",
        ),
        (
            "<script>Math.imul(1);</script>",
            "Math.imul requires exactly two arguments",
        ),
        (
            "<script>Math.pow(2);</script>",
            "Math.pow requires exactly two arguments",
        ),
        (
            "<script>Math.sumPrecise();</script>",
            "Math.sumPrecise requires exactly one argument",
        ),
        (
            "<script>Math.max(1, , 2);</script>",
            "Math.max argument cannot be empty",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail to parse");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains(expected), "expected '{expected}' in '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn math_random_is_deterministic_with_seed() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.random() + ':' + Math.random() + ':' + Math.random();
          });
        </script>
        "#;

    let mut h1 = Harness::from_html(html)?;
    let mut h2 = Harness::from_html(html)?;
    h1.set_random_seed(12345);
    h2.set_random_seed(12345);

    h1.click("#btn")?;
    h2.click("#btn")?;

    let out1 = h1.dump_dom("#result")?;
    let out2 = h2.dump_dom("#result")?;
    assert_eq!(out1, out2);
    Ok(())
}

#[test]
fn math_random_returns_unit_interval() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const r = Math.random();
            if (r >= 0 && r < 1) document.getElementById('result').textContent = 'ok';
            else document.getElementById('result').textContent = 'ng';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_random_seed(42);
    h.click("#btn")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn number_constructor_and_static_properties_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = Number('123');
            const b = Number('12.3');
            const c = Number('');
            const d = Number(null);
            const e = Number('0x11');
            const f = Number('0b11');
            const g = Number('0o11');
            const h = Number('-Infinity') === Number.NEGATIVE_INFINITY;
            const i = Number('foo');
            const j = new Number('5');
            const k = Number.MAX_SAFE_INTEGER === 9007199254740991;
            const l = Number.POSITIVE_INFINITY === Infinity;
            const m = Number.NEGATIVE_INFINITY === -Infinity;
            const n = Number.MIN_VALUE > 0;
            const o = Number.MAX_VALUE > 1e300;
            const p = Number.EPSILON > 0 && Number.EPSILON < 1;
            const q = Number.NaN === Number.NaN;
            const r = window.Number.MAX_SAFE_INTEGER === Number.MAX_SAFE_INTEGER;
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' +
              h + ':' + (i === i) + ':' + j + ':' + k + ':' + l + ':' + m + ':' +
              n + ':' + o + ':' + p + ':' + q + ':' + r;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "123:12.3:0:0:17:3:9:true:false:5:true:true:true:true:true:true:false:true",
    )?;
    Ok(())
}

#[test]
fn number_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = Number.isFinite(1 / 3);
            const b = Number.isFinite('1');
            const c = Number.isInteger(3);
            const d = Number.isInteger(3.1);
            const e = Number.isNaN(NaN);
            const f = Number.isNaN('NaN');
            const g = Number.isSafeInteger(9007199254740991);
            const h = Number.isSafeInteger(9007199254740992);
            const i = Number.parseFloat('3.5px');
            const j = Number.parseInt('10', 2);
            const k = window.Number.parseInt('0x10', 16);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' +
              g + ':' + h + ':' + i + ':' + j + ':' + k;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:false:true:false:true:false:true:false:3.5:2:16",
    )?;
    Ok(())
}

#[test]
fn number_instance_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const n = Number('255');
            document.getElementById('result').textContent =
              (12.34).toFixed() + ':' +
              (12.34).toFixed(1) + ':' +
              (12.34).toExponential() + ':' +
              (12.34).toExponential(2) + ':' +
              (12.34).toPrecision() + ':' +
              (12.34).toPrecision(3) + ':' +
              n.toString(16) + ':' +
              n.toString() + ':' +
              (1.5).toString(2) + ':' +
              (1.5).toLocaleString() + ':' +
              (1.5).valueOf();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "12:12.3:1.234e+1:1.23e+1:12.34:12.3:ff:255:1.1:1.5:1.5",
    )?;
    Ok(())
}

#[test]
fn number_method_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>Number(1, 2);</script>",
            "Number supports zero or one argument",
        ),
        (
            "<script>Number.isFinite();</script>",
            "Number.isFinite requires exactly one argument",
        ),
        (
            "<script>window.Number.parseInt();</script>",
            "Number.parseInt requires one or two arguments",
        ),
        (
            "<script>Number.parseInt('10', );</script>",
            "Number.parseInt radix argument cannot be empty",
        ),
        (
            "<script>(1).toFixed(1, 2);</script>",
            "toFixed supports at most one argument",
        ),
        (
            "<script>(1).toLocaleString(1);</script>",
            "toLocaleString does not take arguments",
        ),
        (
            "<script>(1).valueOf(1);</script>",
            "valueOf does not take arguments",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail to parse");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains(expected), "expected '{expected}' in '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn number_instance_method_runtime_range_errors_are_reported() -> Result<()> {
    let html = r#"
        <button id='fixed'>fixed</button>
        <button id='string'>string</button>
        <script>
          document.getElementById('fixed').addEventListener('click', () => {
            (1).toFixed(101);
          });
          document.getElementById('string').addEventListener('click', () => {
            (1).toString(1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let fixed_err = h
        .click("#fixed")
        .expect_err("toFixed should reject out-of-range fractionDigits");
    match fixed_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("toFixed fractionDigits must be between 0 and 100"))
        }
        other => panic!("unexpected toFixed error: {other:?}"),
    }

    let string_err = h
        .click("#string")
        .expect_err("toString should reject out-of-range radix");
    match string_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("toString radix must be between 2 and 36"))
        }
        other => panic!("unexpected toString error: {other:?}"),
    }

    Ok(())
}

#[test]
fn intl_date_time_and_number_format_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const count = 26254.39;
            const date = new Date('2012-05-24');
            const us =
              new Intl.DateTimeFormat('en-US').format(date) + ' ' +
              new Intl.NumberFormat('en-US').format(count);
            const de =
              new Intl.DateTimeFormat('de-DE').format(date) + ' ' +
              new Intl.NumberFormat('de-DE').format(count);
            document.getElementById('result').textContent = us + '|' + de;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5/24/2012 26,254.39|24.5.2012 26.254,39")?;
    Ok(())
}

#[test]
fn intl_uses_navigator_language_preferences() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const date = new Date('2012-05-24');
            const formattedDate = new Intl.DateTimeFormat(navigator.language).format(date);
            const formattedCount = new Intl.NumberFormat(navigator.languages).format(26254.39);
            document.getElementById('result').textContent = formattedDate + '|' + formattedCount;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5/24/2012|26,254.39")?;
    Ok(())
}

#[test]
fn intl_static_methods_and_to_string_tag_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const canonical = Intl.getCanonicalLocales(['EN-us', 'de-de', 'EN-us']);
            const currencies = Intl.supportedValuesOf('currency');
            document.getElementById('result').textContent =
              canonical.join(',') + '|' + currencies.join(',') + '|' + Intl[Symbol.toStringTag];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "en-US,de-DE|EUR,JPY,USD|Intl")?;
    Ok(())
}

#[test]
fn intl_get_canonical_locales_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const one = Intl.getCanonicalLocales('EN-US');
            const two = Intl.getCanonicalLocales(['EN-US', 'Fr']);
            document.getElementById('result').textContent = one.join(',') + '|' + two.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "en-US|en-US,fr")?;

    let html_error = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Intl.getCanonicalLocales('EN_US');
          });
        </script>
        "#;
    let mut h = Harness::from_html(html_error)?;
    let err = h
        .click("#btn")
        .expect_err("invalid language tag should throw");
    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "RangeError: invalid language tag: \"EN_US\"")
        }
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn intl_supported_values_of_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const calendar = Intl.supportedValuesOf('calendar');
            const collation = Intl.supportedValuesOf('collation');
            const currency = Intl.supportedValuesOf('currency');
            const numberingSystem = Intl.supportedValuesOf('numberingSystem');
            const timeZone = Intl.supportedValuesOf('timeZone');
            const unit = Intl.supportedValuesOf('unit');
            document.getElementById('result').textContent =
              calendar.join(',') + '|' +
              collation.join(',') + '|' +
              currency.join(',') + '|' +
              numberingSystem.join(',') + '|' +
              timeZone.join(',') + '|' +
              unit.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "gregory,islamic-umalqura,japanese|default,emoji,phonebk|EUR,JPY,USD|arab,latn,thai|America/Los_Angeles,America/New_York,Asia/Kolkata,UTC|day,hour,meter,minute,month,second,week,year",
        )?;

    let html_error = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            Intl.supportedValuesOf('someInvalidKey');
          });
        </script>
        "#;
    let mut h = Harness::from_html(html_error)?;
    let err = h
        .click("#btn")
        .expect_err("invalid supportedValuesOf key should throw");
    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "RangeError: invalid key: \"someInvalidKey\"")
        }
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn intl_namespace_is_not_a_constructor() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const i = new Intl();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h.click("#btn").expect_err("new Intl should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Intl is not a constructor")),
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn intl_date_time_format_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const date = new Date(Date.UTC(2020, 11, 20, 3, 23, 16, 738));
            const us = new Intl.DateTimeFormat('en-US').format(date);
            const fallback = new Intl.DateTimeFormat(['ban', 'id']).format(date);
            const styled = new Intl.DateTimeFormat('en-GB', {
              dateStyle: 'full',
              timeStyle: 'long',
              timeZone: 'Australia/Sydney',
            }).format(date);
            document.getElementById('result').textContent = us + '|' + fallback + '|' + styled;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "12/20/2020|20/12/2020|Sunday, 20 December 2020 at 14:23:16 GMT+11",
    )?;
    Ok(())
}

#[test]
fn intl_date_time_format_instance_methods_and_getter_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const dtf = new Intl.DateTimeFormat('en-US', {
              year: 'numeric',
              month: '2-digit',
              day: '2-digit',
              timeZone: 'UTC'
            });
            const d1 = new Date(Date.UTC(2020, 11, 20, 3, 23, 16, 738));
            const d2 = new Date(Date.UTC(2020, 11, 21, 3, 23, 16, 738));
            const fmt = dtf.format;
            const fromGetter = fmt(d1);
            const parts = dtf.formatToParts(d1);
            const range = dtf.formatRange(d1, d2);
            const rangeParts = dtf.formatRangeToParts(d1, d2);
            const partsOk = JSON.stringify(parts).includes('"type":"month"');
            const rangePartsOk =
              JSON.stringify(rangeParts).includes('"source":"startRange"') &&
              JSON.stringify(rangeParts).includes('"source":"endRange"');
            document.getElementById('result').textContent =
              fromGetter + '|' + range + '|' + partsOk + ':' + rangePartsOk;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12/20/2020|12/20/2020 - 12/21/2020|true:true")?;
    Ok(())
}

#[test]
fn intl_date_time_format_supported_locales_and_resolved_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const supportedLocales = Intl.DateTimeFormat.supportedLocalesOf(['ban', 'id', 'en-GB', 'fr']);
            const supported = supportedLocales.join(',');
            const ro = new Intl.DateTimeFormat('ja-JP-u-ca-japanese', {
              numberingSystem: 'arab',
              timeZone: 'America/Los_Angeles',
              dateStyle: 'short',
            }).resolvedOptions();
            const tag = Intl.DateTimeFormat.prototype[Symbol.toStringTag];
            const ar = new Intl.DateTimeFormat('ar-EG').format(new Date(Date.UTC(2012, 11, 20, 3, 0, 0)));
            document.getElementById('result').textContent =
              supported + '|' + ro.locale + ':' + ro.calendar + ':' + ro.numberingSystem + ':' +
              ro.timeZone + ':' + ro.dateStyle + '|' + tag + '|' + ar;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "id,en-GB|ja-JP-u-ca-japanese:japanese:arab:America/Los_Angeles:short|Intl.DateTimeFormat|٢٠/١٢/٢٠١٢",
        )?;
    Ok(())
}

#[test]
fn intl_duration_format_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const duration = {
              hours: 1,
              minutes: 46,
              seconds: 40,
            };
            const fr = new Intl.DurationFormat('fr-FR', { style: 'long' }).format(duration);
            const en = new Intl.DurationFormat('en', { style: 'short' }).format(duration);
            const pt = new Intl.DurationFormat('pt', { style: 'narrow' }).format(duration);
            document.getElementById('result').textContent = fr + '|' + en + '|' + pt;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "1 heure, 46 minutes et 40 secondes|1 hr, 46 min and 40 sec|1 h 46 min 40 s",
    )?;
    Ok(())
}

#[test]
fn intl_duration_format_instance_methods_and_static_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const duration = {
              hours: 1,
              minutes: 2,
              seconds: 3,
            };
            const df = new Intl.DurationFormat('en', { style: 'short' });
            const fmt = df.format;
            const fromGetter = fmt(duration);
            const parts = df.formatToParts(duration);
            const partsOk =
              JSON.stringify(parts).includes('"type":"hour"') &&
              JSON.stringify(parts).includes('"type":"literal"');
            const supported = Intl.DurationFormat.supportedLocalesOf(['fr-FR', 'en', 'pt', 'de']);
            const ro = new Intl.DurationFormat('pt', { style: 'narrow' }).resolvedOptions();
            const tag = Intl.DurationFormat.prototype[Symbol.toStringTag];
            document.getElementById('result').textContent =
              supported.join(',') + '|' + fromGetter + '|' + partsOk + '|' +
              ro.locale + ':' + ro.style + '|' + tag;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "fr-FR,en,pt|1 hr, 2 min and 3 sec|true|pt:narrow|Intl.DurationFormat",
    )?;
    Ok(())
}

#[test]
fn intl_list_format_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const vehicles = ['Motorcycle', 'Bus', 'Car'];
            const formatter = new Intl.ListFormat('en', {
              style: 'long',
              type: 'conjunction',
            });
            const formatter2 = new Intl.ListFormat('de', {
              style: 'short',
              type: 'disjunction',
            });
            const formatter3 = new Intl.ListFormat('en', { style: 'narrow', type: 'unit' });
            document.getElementById('result').textContent =
              formatter.format(vehicles) + '|' +
              formatter2.format(vehicles) + '|' +
              formatter3.format(vehicles);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Motorcycle, Bus, and Car|Motorcycle, Bus oder Car|Motorcycle Bus Car",
    )?;
    Ok(())
}

#[test]
fn intl_list_format_methods_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const list = ['Motorcycle', 'Bus', 'Car'];
            const long = new Intl.ListFormat('en-GB', { style: 'long', type: 'conjunction' }).format(list);
            const short = new Intl.ListFormat('en-GB', { style: 'short', type: 'disjunction' }).format(list);
            const narrow = new Intl.ListFormat('en-GB', { style: 'narrow', type: 'unit' }).format(list);
            const parts = new Intl.ListFormat('en-GB', {
              style: 'long',
              type: 'conjunction',
            }).formatToParts(list);
            const partsOk =
              JSON.stringify(parts).includes('"type":"element"') &&
              JSON.stringify(parts).includes('"value":" and "');
            const supportedLocales = Intl.ListFormat.supportedLocalesOf(['de', 'en-GB', 'fr']);
            const supported = supportedLocales.join(',');
            const ro = new Intl.ListFormat('en-GB', {
              style: 'short',
              type: 'disjunction'
            }).resolvedOptions();
            const roTypeOk = JSON.stringify(ro).includes('"type":"disjunction"');
            const tag = Intl.ListFormat.prototype[Symbol.toStringTag];
            const defaultList = new Intl.ListFormat('en');
            const ctor = defaultList.constructor === Intl.ListFormat;
            const format = defaultList.format;
            const fromGetter = format(list);
            document.getElementById('result').textContent =
              long + '|' + short + '|' + narrow + '|' + partsOk + '|' +
              supported + '|' + ro.locale + ':' + ro.style + ':' + roTypeOk + '|' +
              tag + '|' + ctor + '|' + fromGetter;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "Motorcycle, Bus and Car|Motorcycle, Bus or Car|Motorcycle Bus Car|true|de,en-GB|en-GB:short:true|Intl.ListFormat|true|Motorcycle, Bus, and Car",
        )?;
    Ok(())
}

#[test]
fn intl_locale_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const korean = new Intl.Locale('ko', {
              script: 'Kore',
              region: 'KR',
              hourCycle: 'h23',
              calendar: 'gregory',
            });
            const japanese = new Intl.Locale('ja-Jpan-JP-u-ca-japanese-hc-h12');
            document.getElementById('result').textContent =
              korean.baseName + '|' + japanese.baseName + '|' +
              korean.hourCycle + '|' + japanese.hourCycle;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ko-Kore-KR|ja-Jpan-JP|h23|h12")?;
    Ok(())
}

#[test]
fn intl_locale_properties_and_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const locale = new Intl.Locale('en-Latn-US-u-ca-gregory-kf-upper-co-phonebk-kn-nu-latn');
            const us = new Intl.Locale('en-US', { hourCycle: 'h12' });
            const textInfo = new Intl.Locale('he-IL').getTextInfo();
            const weekInfo = us.getWeekInfo();
            const weekend = weekInfo.weekend;
            const maximize = new Intl.Locale('zh').maximize();
            const minimize = maximize.minimize();
            const props =
              locale.language + ':' + locale.script + ':' + locale.region + ':' +
              locale.calendar + ':' + locale.caseFirst + ':' + locale.collation + ':' +
              locale.numberingSystem + ':' + locale.numeric + ':' +
              locale.hourCycle + ':' + locale.variants.length;
            const calendars = us.getCalendars();
            const collations = us.getCollations();
            const hourCycles = us.getHourCycles();
            const numberingSystems = us.getNumberingSystems();
            const timeZones = us.getTimeZones();
            const tag = Intl.Locale.prototype[Symbol.toStringTag];
            const ctor = us.constructor === Intl.Locale;
            const full = us.toString();
            document.getElementById('result').textContent =
              props + '|' + calendars.join(',') + '|' + collations.join(',') + '|' + hourCycles.join(',') + '|' +
              numberingSystems.join(',') + '|' + textInfo.direction + '|' + timeZones.join(',') + '|' +
              weekInfo.firstDay + ':' + weekInfo.minimalDays + ':' + weekend.join('/') + '|' +
              maximize.baseName + ':' + minimize.baseName + '|' + tag + '|' + ctor + '|' + full;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "en:Latn:US:gregory:upper:phonebk:latn:true:undefined:0|gregory|default,emoji|h12,h23|latn|rtl|America/New_York,America/Los_Angeles|7:1:6/7|zh-Hans-CN:zh|Intl.Locale|true|en-US-u-hc-h12",
        )?;
    Ok(())
}

#[test]
fn intl_plural_rules_locales_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const enCardinalRules = new Intl.PluralRules('en-US');
            const arCardinalRules = new Intl.PluralRules('ar-EG');
            const enOrdinalRules = new Intl.PluralRules('en-US', { type: 'ordinal' });
            document.getElementById('result').textContent =
              enCardinalRules.select(0) + ':' + enCardinalRules.select(1) + ':' +
              enCardinalRules.select(2) + ':' + enCardinalRules.select(3) + '|' +
              arCardinalRules.select(0) + ':' + arCardinalRules.select(1) + ':' +
              arCardinalRules.select(2) + ':' + arCardinalRules.select(6) + ':' +
              arCardinalRules.select(18) + '|' +
              enOrdinalRules.select(0) + ':' + enOrdinalRules.select(1) + ':' +
              enOrdinalRules.select(2) + ':' + enOrdinalRules.select(3) + ':' +
              enOrdinalRules.select(4) + ':' + enOrdinalRules.select(21);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "other:one:other:other|zero:one:two:few:many|other:one:two:few:other:one",
    )?;
    Ok(())
}

#[test]
fn intl_plural_rules_methods_and_static_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const enOrdinalRules = new Intl.PluralRules('en-US', { type: 'ordinal' });
            const supported = Intl.PluralRules.supportedLocalesOf(['ar-EG', 'en-US', 'de']);
            const ro = enOrdinalRules.resolvedOptions();
            const categories = ro.pluralCategories;
            const categoriesText = categories.join(',');
            const range =
              enOrdinalRules.selectRange(1, 1) + ':' +
              enOrdinalRules.selectRange(1, 2) + ':' +
              new Intl.PluralRules('ar-EG').selectRange(0, 0);
            const suffixes = { one: 'st', two: 'nd', few: 'rd', other: 'th' };
            const formatOrdinals = (n) => {
              const rule = enOrdinalRules.select(n);
              return n + suffixes[rule];
            };
            const tag = Intl.PluralRules.prototype[Symbol.toStringTag];
            const ctor = enOrdinalRules.constructor === Intl.PluralRules;
            document.getElementById('result').textContent =
              supported.join(',') + '|' +
              ro.locale + ':' + ro['type'] + ':' + categoriesText + '|' +
              range + '|' +
              formatOrdinals(0) + ',' + formatOrdinals(1) + ',' + formatOrdinals(2) + ',' +
              formatOrdinals(3) + ',' + formatOrdinals(4) + ',' + formatOrdinals(11) + ',' +
              formatOrdinals(21) + ',' + formatOrdinals(42) + ',' + formatOrdinals(103) + '|' +
              tag + '|' + ctor;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "ar-EG,en-US|en-US:ordinal:one,two,few,other|one:other:zero|0th,1st,2nd,3rd,4th,11th,21st,42nd,103rd|Intl.PluralRules|true",
        )?;
    Ok(())
}

#[test]
fn intl_relative_time_format_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const rtf1 = new Intl.RelativeTimeFormat('en', { style: 'short' });
            const qtrs = rtf1.format(3, 'quarter');
            const ago = rtf1.format(-1, 'day');
            const rtf2 = new Intl.RelativeTimeFormat('es', { numeric: 'auto' });
            const auto = rtf2.format(2, 'day');
            document.getElementById('result').textContent = qtrs + '|' + ago + '|' + auto;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "in 3 qtrs.|1 day ago|pasado mañana")?;
    Ok(())
}

#[test]
fn intl_relative_time_format_methods_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const rtf = new Intl.RelativeTimeFormat('en', {
              localeMatcher: 'best fit',
              numeric: 'auto',
              style: 'long',
            });
            const text = rtf.format(-1, 'day');
            const partsAuto = rtf.formatToParts(-1, 'day');
            const parts = rtf.formatToParts(100, 'day');
            const partsOk =
              JSON.stringify(partsAuto).includes('"value":"yesterday"') &&
              JSON.stringify(parts).includes('"type":"integer"') &&
              JSON.stringify(parts).includes('"unit":"day"') &&
              JSON.stringify(parts).includes('"value":"in "') &&
              JSON.stringify(parts).includes('"value":"100"') &&
              JSON.stringify(parts).includes('"value":" days"');
            const supportedLocales = Intl.RelativeTimeFormat.supportedLocalesOf(['es', 'en', 'de']);
            const supported = supportedLocales.join(',');
            const ro = rtf.resolvedOptions();
            const tag = Intl.RelativeTimeFormat.prototype[Symbol.toStringTag];
            const ctor = rtf.constructor === Intl.RelativeTimeFormat;
            document.getElementById('result').textContent =
              text + '|' + partsOk + '|' + supported + '|' +
              ro.locale + ':' + ro.style + ':' + ro.numeric + ':' + ro.localeMatcher + '|' +
              tag + '|' + ctor;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "yesterday|true|es,en|en:long:auto:best fit|Intl.RelativeTimeFormat|true",
    )?;
    Ok(())
}

#[test]
fn intl_segmenter_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const segmenterFr = new Intl.Segmenter('fr', { granularity: 'word' });
            const string = 'Que ma joie demeure';
            const segments = segmenterFr.segment(string);
            const iteratorFactory = segments[Symbol.iterator];
            const iterator = iteratorFactory();
            const next = iterator.next;
            const first = next();
            const second = next();
            document.getElementById('result').textContent =
              first.value.segment + '|' + second.value.segment + '|' + first.done + ':' + second.done;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Que| |false:false")?;
    Ok(())
}

#[test]
fn intl_segmenter_methods_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const str = '吾輩は猫である。名前はたぬき。';
            const split = str.split(' ');
            const segmenter = new Intl.Segmenter('ja-JP', { granularity: 'word' });
            const segments = segmenter.segment(str);
            const first = segments[0];
            const second = segments[1];
            const count = segments['length'];
            const arr = Array.from(segments);
            const arrFirst = arr[0];
            const supported = Intl.Segmenter.supportedLocalesOf(['fr', 'ja-JP', 'de']);
            const ro = segmenter.resolvedOptions();
            const tag = Intl.Segmenter.prototype[Symbol.toStringTag];
            const ctor = segmenter.constructor === Intl.Segmenter;
            document.getElementById('result').textContent =
              split.length + '|' +
              first.segment + ':' + second.segment + ':' + first.isWordLike + ':' + count + ':' +
              arrFirst.segment + ':' + arr.length + '|' +
              supported.join(',') + '|' +
              ro.locale + ':' + ro.granularity + ':' + ro.localeMatcher + '|' +
              tag + '|' + ctor;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "1|吾輩:は:true:10:吾輩:10|fr,ja-JP|ja-JP:word:best fit|Intl.Segmenter|true",
    )?;
    Ok(())
}

#[test]
fn intl_display_names_try_it_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const regionNamesInEnglish = new Intl.DisplayNames(['en'], { type: 'region' });
            const regionNamesInTraditionalChinese = new Intl.DisplayNames(['zh-Hant'], {
              type: 'region',
            });
            document.getElementById('result').textContent =
              regionNamesInEnglish.of('US') + '|' + regionNamesInTraditionalChinese.of('US');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "United States|美國")?;
    Ok(())
}

#[test]
fn intl_display_names_of_examples_for_multiple_types_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const regionNamesEn = new Intl.DisplayNames(['en'], { type: 'region' });
            const regionNamesZh = new Intl.DisplayNames(['zh-Hant'], { type: 'region' });
            const languageNamesEn = new Intl.DisplayNames(['en'], { type: 'language' });
            const languageNamesZh = new Intl.DisplayNames(['zh-Hant'], { type: 'language' });
            const scriptNamesEn = new Intl.DisplayNames(['en'], { type: 'script' });
            const scriptNamesZh = new Intl.DisplayNames(['zh-Hant'], { type: 'script' });
            const currencyNamesEn = new Intl.DisplayNames(['en'], { type: 'currency' });
            const currencyNamesZh = new Intl.DisplayNames(['zh-Hant'], { type: 'currency' });

            document.getElementById('result').textContent =
              regionNamesEn.of('419') + ':' + regionNamesZh.of('MM') + '|' +
              languageNamesEn.of('fr-CA') + ':' + languageNamesZh.of('fr') + '|' +
              scriptNamesEn.of('Latn') + ':' + scriptNamesZh.of('Kana') + '|' +
              currencyNamesEn.of('TWD') + ':' + currencyNamesZh.of('USD');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Latin America:緬甸|Canadian French:法文|Latin:片假名|New Taiwan Dollar:美元",
    )?;
    Ok(())
}

#[test]
fn intl_display_names_static_and_resolved_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const supported = Intl.DisplayNames.supportedLocalesOf(['zh-Hant', 'en', 'de']);
            const ro = new Intl.DisplayNames(['zh-Hant'], {
              type: 'language',
              style: 'short',
              fallback: 'none',
              languageDisplay: 'standard'
            }).resolvedOptions();
            const tag = Intl.DisplayNames.prototype[Symbol.toStringTag];
            const unknown = new Intl.DisplayNames(['en'], { type: 'region', fallback: 'none' }).of('ZZ');
            document.getElementById('result').textContent =
              supported.join(',') + '|' +
              ro.locale + ':' + ro.style + ':' + ro.fallback + ':' + ro.languageDisplay + '|' +
              tag + '|' + (unknown === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "zh-Hant,en|zh-Hant:short:none:standard|Intl.DisplayNames|true",
    )?;
    Ok(())
}

#[test]
fn intl_display_names_ja_and_he_dictionaries_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const supported = Intl.DisplayNames.supportedLocalesOf(['he', 'ja', 'de']);
            const regionJa = new Intl.DisplayNames(['ja'], { type: 'region' }).of('US');
            const regionHe = new Intl.DisplayNames(['he'], { type: 'region' }).of('US');
            const languageJa = new Intl.DisplayNames(['ja'], { type: 'language' }).of('fr-CA');
            const languageHe = new Intl.DisplayNames(['he'], { type: 'language' }).of('fr-CA');
            const scriptJa = new Intl.DisplayNames(['ja'], { type: 'script' }).of('Latn');
            const scriptHe = new Intl.DisplayNames(['he'], { type: 'script' }).of('Arab');
            const currencyJa = new Intl.DisplayNames(['ja'], { type: 'currency' }).of('USD');
            const currencyHe = new Intl.DisplayNames(['he'], { type: 'currency' }).of('EUR');
            document.getElementById('result').textContent =
              supported.join(',') + '|' +
              regionJa + ':' + regionHe + '|' +
              languageJa + ':' + languageHe + '|' +
              scriptJa + ':' + scriptHe + '|' +
              currencyJa + ':' + currencyHe;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "he,ja|アメリカ合衆国:ארצות הברית|カナダのフランス語:צרפתית קנדית|ラテン文字:ערבי|米ドル:אירו",
        )?;
    Ok(())
}

#[test]
fn intl_display_names_es_and_fr_dictionaries_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const supported = Intl.DisplayNames.supportedLocalesOf(['es', 'fr', 'de']);
            const regionEs = new Intl.DisplayNames(['es'], { type: 'region' }).of('US');
            const regionFr = new Intl.DisplayNames(['fr'], { type: 'region' }).of('US');
            const languageEs = new Intl.DisplayNames(['es'], { type: 'language' }).of('fr-CA');
            const languageFr = new Intl.DisplayNames(['fr'], { type: 'language' }).of('fr-CA');
            const scriptEs = new Intl.DisplayNames(['es'], { type: 'script' }).of('Arab');
            const scriptFr = new Intl.DisplayNames(['fr'], { type: 'script' }).of('Latn');
            const currencyEs = new Intl.DisplayNames(['es'], { type: 'currency' }).of('USD');
            const currencyFr = new Intl.DisplayNames(['fr'], { type: 'currency' }).of('TWD');
            document.getElementById('result').textContent =
              supported.join(',') + '|' +
              regionEs + ':' + regionFr + '|' +
              languageEs + ':' + languageFr + '|' +
              scriptEs + ':' + scriptFr + '|' +
              currencyEs + ':' + currencyFr;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "es,fr|Estados Unidos:États-Unis|francés canadiense:français canadien|árabe:latin|dólar estadounidense:nouveau dollar taïwanais",
        )?;
    Ok(())
}

#[test]
fn intl_collator_compare_returns_sign_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const c = new Intl.Collator();
            const a = c.compare('a', 'c');
            const b = c.compare('c', 'a');
            const d = c.compare('a', 'a');
            document.getElementById('result').textContent =
              (a < 0) + ':' + (b > 0) + ':' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:0")?;
    Ok(())
}

#[test]
fn intl_collator_demo_sort_and_options_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const deValues = ['Z', 'a', 'z', 'ä'];
            deValues.sort(new Intl.Collator('de').compare);
            const de = deValues.join(',');
            const svValues = ['Z', 'a', 'z', 'ä'];
            svValues.sort(new Intl.Collator('sv').compare);
            const sv = svValues.join(',');
            const upValues = ['Z', 'a', 'z', 'ä'];
            upValues.sort(new Intl.Collator('de', { caseFirst: 'upper' }).compare);
            const up = upValues.join(',');
            document.getElementById('result').textContent = de + '|' + sv + '|' + up;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a,ä,z,Z|a,z,Z,ä|a,ä,Z,z")?;
    Ok(())
}

#[test]
fn intl_collator_locales_sensitivity_and_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const deCmp = new Intl.Collator('de').compare('ä', 'z');
            const svCmp = new Intl.Collator('sv').compare('ä', 'z');
            const deBase = new Intl.Collator('de', { sensitivity: 'base' }).compare('ä', 'a');
            const svBase = new Intl.Collator('sv', { sensitivity: 'base' }).compare('ä', 'a');
            const supportedLocales = Intl.Collator.supportedLocalesOf(['de', 'sv', 'fr']);
            const supported = supportedLocales.join(',');
            const ro = new Intl.Collator('sv', {
              caseFirst: 'upper',
              sensitivity: 'base'
            }).resolvedOptions();
            const tag = Intl.Collator.prototype[Symbol.toStringTag];
            document.getElementById('result').textContent =
              (deCmp < 0) + ':' + (svCmp > 0) + ':' + deBase + ':' + (svBase > 0) + ':' +
              supported + ':' + ro.locale + ':' + ro.caseFirst + ':' + ro.sensitivity + ':' + tag;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:true:0:true:de,sv:sv:upper:base:Intl.Collator",
    )?;
    Ok(())
}

#[test]
fn bigint_literals_constructor_and_typeof_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const bigA = 9007199254740991n;
            const bigB = BigInt(9007199254740991);
            const bigC = BigInt("0x1fffffffffffff");
            const bigD = BigInt("0o377777777777777777");
            const bigE = BigInt("0b11111111111111111111111111111111111111111111111111111");
            const typeA = typeof bigA;
            const typeB = typeof bigB;
            const falsyBranch = 0n ? 't' : 'f';
            const truthyBranch = 12n ? 't' : 'f';
            const concat = 'x' + 1n;
            document.getElementById('result').textContent =
              bigA + ':' + bigB + ':' + bigC + ':' + bigD + ':' + bigE + ':' +
              typeA + ':' + typeB + ':' + falsyBranch + ':' + truthyBranch + ':' + concat;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "9007199254740991:9007199254740991:9007199254740991:9007199254740991:9007199254740991:bigint:bigint:f:t:x1",
        )?;
    Ok(())
}

#[test]
fn bigint_static_and_instance_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = BigInt.asIntN(8, 257n);
            const b = BigInt.asIntN(8, 255n);
            const c = BigInt.asUintN(8, -1n);
            const d = window.BigInt.asUintN(8, 257n);
            const e = (255n).toString(16);
            const f = (255n).toString();
            const g = (255n).toLocaleString();
            const h = (255n).valueOf() === 255n;
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' + h;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:-1:255:1:ff:255:255:true")?;
    Ok(())
}

#[test]
fn bigint_arithmetic_and_bitwise_operations_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const previousMaxSafe = BigInt(Number.MAX_SAFE_INTEGER);
            const maxPlusTwo = previousMaxSafe + 2n;
            const prod = previousMaxSafe * 2n;
            const diff = prod - 10n;
            const mod = prod % 10n;
            const pow = 2n ** 54n;
            const neg = pow * -1n;
            const div1 = 4n / 2n;
            const div2 = 5n / 2n;
            const bitAnd = 6n & 3n;
            const bitOr = 6n | 3n;
            const bitXor = 6n ^ 3n;
            const shl = 8n << 1n;
            const shr = 8n >> 1n;
            const shlNeg = 8n << -1n;
            document.getElementById('result').textContent =
              maxPlusTwo + ':' + diff + ':' + mod + ':' + pow + ':' + neg + ':' +
              div1 + ':' + div2 + ':' + bitAnd + ':' + bitOr + ':' + bitXor + ':' +
              shl + ':' + shr + ':' + shlNeg;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "9007199254740993:18014398509481972:2:18014398509481984:-18014398509481984:2:2:2:7:5:16:4:4",
        )?;
    Ok(())
}

#[test]
fn bigint_comparisons_and_increment_decrement_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 1n;
            ++n;
            n++;
            --n;
            n--;
            const a = 0n == 0;
            const b = 0n === 0;
            const c = 1n < 2;
            const d = 2n > 1;
            const e = 2n > 2;
            const f = 2n >= 2;
            document.getElementById('result').textContent =
              n + ':' + a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' +
              (typeof n) + ':' + (0n ? 't' : 'f') + ':' + (12n ? 't' : 'f');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:true:false:true:true:false:true:bigint:f:t")?;
    Ok(())
}

#[test]
fn bigint_mixed_type_and_unsupported_operations_report_errors() -> Result<()> {
    let html = r#"
        <button id='mix'>mix</button>
        <button id='ushift'>ushift</button>
        <button id='unary'>unary</button>
        <script>
          document.getElementById('mix').addEventListener('click', () => {
            const v = 1n + 1;
          });
          document.getElementById('ushift').addEventListener('click', () => {
            const v = 1n >>> 0n;
          });
          document.getElementById('unary').addEventListener('click', () => {
            const v = +1n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let mix_err = h
        .click("#mix")
        .expect_err("mixed BigInt/Number arithmetic should fail");
    match mix_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("cannot mix BigInt and other types in addition"))
        }
        other => panic!("unexpected mixed operation error: {other:?}"),
    }

    let us_err = h
        .click("#ushift")
        .expect_err("unsigned right shift for BigInt should fail");
    match us_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("BigInt values do not support unsigned right shift"))
        }
        other => panic!("unexpected unsigned shift error: {other:?}"),
    }

    let unary_err = h
        .click("#unary")
        .expect_err("unary plus for BigInt should fail");
    match unary_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("unary plus is not supported for BigInt values"))
        }
        other => panic!("unexpected unary plus error: {other:?}"),
    }

    Ok(())
}

#[test]
fn bigint_constructor_and_json_stringify_errors_are_reported() -> Result<()> {
    let html = r#"
        <button id='ctor'>ctor</button>
        <button id='newctor'>newctor</button>
        <button id='json'>json</button>
        <script>
          document.getElementById('ctor').addEventListener('click', () => {
            BigInt('1.5');
          });
          document.getElementById('newctor').addEventListener('click', () => {
            new BigInt(1);
          });
          document.getElementById('json').addEventListener('click', () => {
            JSON.stringify({ a: 1n });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let ctor_err = h
        .click("#ctor")
        .expect_err("BigInt constructor should reject decimal string");
    match ctor_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("cannot convert 1.5 to a BigInt")),
        other => panic!("unexpected BigInt conversion error: {other:?}"),
    }

    let new_ctor_err = h.click("#newctor").expect_err("new BigInt should fail");
    match new_ctor_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("BigInt is not a constructor")),
        other => panic!("unexpected new BigInt error: {other:?}"),
    }

    let json_err = h
        .click("#json")
        .expect_err("JSON.stringify with BigInt should fail");
    match json_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("JSON.stringify does not support BigInt values"))
        }
        other => panic!("unexpected JSON.stringify BigInt error: {other:?}"),
    }

    Ok(())
}

#[test]
fn decimal_numeric_literals_work_in_comparisons_and_assignment() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 0.5;
            const b = 1.0;
            if (a < b && a === 0.5 && b >= 1)
              document.getElementById('result').textContent = a;
            else
              document.getElementById('result').textContent = 'ng';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0.5")?;
    Ok(())
}

#[test]
fn multiplication_and_division_work_for_numbers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 6 * 7;
            const b = 5 / 2;
            document.getElementById('result').textContent = a + ':' + b;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "42:2.5")?;
    Ok(())
}

#[test]
fn subtraction_and_unary_minus_work_for_numbers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 10 - 3;
            const b = -2;
            const c = 1 - -2;
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "7:-2:3")?;
    Ok(())
}

#[test]
fn addition_supports_numeric_and_string_left_fold() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 1 + 2;
            const b = 1 + 2 + 'x';
            const c = 1 + '2' + 3;
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:3x:123")?;
    Ok(())
}

#[test]
fn timer_delay_accepts_arithmetic_expression() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 5 * 2);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(9)?;
    h.assert_text("#result", "")?;
    h.advance_time(1)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn timer_delay_accepts_addition_expression() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 5 + 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(10)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn timer_delay_accepts_subtraction_expression() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 15 - 5);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(10)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn timer_arrow_expression_callback_executes() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(
              () => setTimeout(() => {
                document.getElementById('result').textContent = 'ok';
              }, 0),
              5
            );
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(5)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn math_random_seed_reset_repeats_sequence() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              Math.random() + ':' + Math.random();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_random_seed(7);
    h.click("#btn")?;
    let first = h.dump_dom("#result")?;

    h.set_random_seed(7);
    h.click("#btn")?;
    let second = h.dump_dom("#result")?;

    assert_eq!(first, second);
    Ok(())
}

#[test]
fn clear_timeout_cancels_task_and_set_timeout_returns_ids() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const first = setTimeout(() => {
              result.textContent = result.textContent + 'A';
            }, 5);
            const second = setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
            clearTimeout(first);
            result.textContent = first + ':' + second + ':';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:2:")?;
    h.flush()?;
    h.assert_text("#result", "1:2:B")?;
    Ok(())
}

#[test]
fn clear_timeout_unknown_id_is_ignored() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            clearTimeout(999);
            setTimeout(() => {
              document.getElementById('result').textContent = 'ok';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.flush()?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn script_extractor_ignores_script_like_strings() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const marker = "</script>";
          const htmlLike = "<script>not real</script>";
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = marker + '|' + htmlLike;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "</script>|<script>not real</script>")?;
    Ok(())
}

#[test]
fn script_extractor_handles_regex_literals_with_quotes_for_end_tag_scan() -> Result<()> {
    let html = r##"
        <script>
          const sanitizer = /["]/g;
        </script>
        <p id="result"></p>
        "##;

    let parsed = parse_html(html)?;
    assert_eq!(parsed.scripts.len(), 1);
    assert!(parsed.scripts[0].contains(r#"/["]/g"#));
    Ok(())
}

#[test]
fn doctype_declaration_is_ignored_during_html_parse() -> Result<()> {
    let html = r#"
        <!DOCTYPE html>
        <p id="result">ok</p>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn set_interval_repeats_and_clear_interval_stops_requeue() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const id = setInterval(() => {
              result.textContent = result.textContent + 'I';
              if (result.textContent === '1:III') clearInterval(id);
            }, 0);
            result.textContent = id + ':';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:")?;
    h.flush()?;
    h.assert_text("#result", "1:III")?;
    h.flush()?;
    h.assert_text("#result", "1:III")?;
    Ok(())
}

#[test]
fn clear_timeout_can_cancel_interval_id() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const id = setInterval(() => {
              result.textContent = result.textContent + 'X';
            }, 0);
            clearTimeout(id);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "")?;
    Ok(())
}

#[test]
fn flush_step_limit_error_contains_timer_diagnostics() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    let err = h
        .flush()
        .expect_err("flush should fail on uncleared interval");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("flush exceeded max task steps"));
            assert!(msg.contains("limit=10000"));
            assert!(msg.contains("pending_tasks="));
            assert!(msg.contains("next_task=id=1"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn timer_step_limit_can_be_configured() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_timer_step_limit(3)?;
    h.click("#btn")?;
    let err = h
        .flush()
        .expect_err("flush should fail with configured small step limit");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("limit=3"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn timer_step_limit_rejects_zero() -> Result<()> {
    let html = r#"<button id='btn'>run</button>"#;
    let mut h = Harness::from_html(html)?;
    let err = h
        .set_timer_step_limit(0)
        .expect_err("zero step limit should be rejected");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("set_timer_step_limit requires at least 1 step"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn advance_time_step_limit_error_contains_due_limit() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setInterval(() => {}, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_timer_step_limit(2)?;
    h.click("#btn")?;
    let err = h
        .advance_time(7)
        .expect_err("advance_time should fail with configured small step limit");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("limit=2"));
            assert!(msg.contains("now_ms=7"));
            assert!(msg.contains("due_limit=7"));
            assert!(msg.contains("next_task=id=1"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn assertion_failure_contains_dom_snippet() -> Result<()> {
    let html = r#"
        <p id='result'>NG</p>
        "#;
    let h = Harness::from_html(html)?;

    let err = match h.assert_text("#result", "OK") {
        Ok(()) => panic!("assert_text should fail"),
        Err(err) => err,
    };

    match err {
        Error::AssertionFailed {
            selector,
            expected,
            actual,
            dom_snippet,
        } => {
            assert_eq!(selector, "#result");
            assert_eq!(expected, "OK");
            assert_eq!(actual, "NG");
            assert!(dom_snippet.contains("<p"));
            assert!(dom_snippet.contains("NG"));
        }
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn remove_and_has_attribute_work() -> Result<()> {
    let html = r#"
        <div id='box' data-x='1' class='a'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            const before = box.hasAttribute('data-x');
            box.removeAttribute('data-x');
            const after = box.hasAttribute('data-x');
            box.removeAttribute('class');
            document.getElementById('result').textContent =
              before + ':' + after + ':' + box.className + ':' + box.getAttribute('data-x');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false::")?;
    Ok(())
}

#[test]
fn remove_id_attribute_updates_id_selector_index() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.removeAttribute('id');
            document.getElementById('result').textContent =
              document.querySelectorAll('#box').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0")?;
    Ok(())
}

#[test]
fn create_element_append_and_remove_child_work() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const node = document.createElement('span');
            node.id = 'tmp';
            node.textContent = 'X';

            document.getElementById('result').textContent =
              document.querySelectorAll('#tmp').length + ':';
            root.appendChild(node);
            document.getElementById('result').textContent =
              document.getElementById('result').textContent +
              document.querySelectorAll('#tmp').length + ':' +
              document.querySelector('#root>#tmp').textContent;
            root.removeChild(node);
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' +
              document.querySelectorAll('#tmp').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:1:X:0")?;
    Ok(())
}

#[test]
fn insert_before_inserts_new_node_before_reference() -> Result<()> {
    let html = r#"
        <div id='root'><span id='a'>A</span><span id='c'>C</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const b = document.createElement('span');
            b.id = 'b';
            b.textContent = 'B';
            root.insertBefore(b, document.getElementById('c'));
            document.getElementById('result').textContent =
              root.textContent + ':' + document.querySelector('#root>#b').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABC:B")?;
    Ok(())
}

#[test]
fn insert_before_reorders_existing_child() -> Result<()> {
    let html = r#"
        <div id='root'><span id='a'>A</span><span id='b'>B</span><span id='c'>C</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            root.insertBefore(
              document.getElementById('c'),
              document.getElementById('a')
            );
            document.getElementById('result').textContent = root.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "CAB")?;
    Ok(())
}

#[test]
fn append_alias_adds_child_to_end() -> Result<()> {
    let html = r#"
        <div id='root'><span>A</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const b = document.createElement('span');
            b.id = 'b';
            b.textContent = 'B';
            root.append(b);
            document.getElementById('result').textContent =
              root.textContent + ':' + document.querySelector('#root>#b').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB:B")?;
    Ok(())
}

#[test]
fn prepend_adds_child_to_start() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span><span id='c'>C</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            root.prepend(a);
            document.getElementById('result').textContent =
              root.textContent + ':' + document.querySelector('#root>#a').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABC:A")?;
    Ok(())
}

#[test]
fn before_and_after_insert_relative_to_target() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            const c = document.createElement('span');
            c.id = 'c';
            c.textContent = 'C';
            b.before(a);
            b.after(c);
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelector('#root>#a').textContent + ':' +
              document.querySelector('#root>#c').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABC:A:C")?;
    Ok(())
}

#[test]
fn replace_with_replaces_node_and_updates_id_index() -> Result<()> {
    let html = r#"
        <div id='root'><span id='old'>O</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const old = document.getElementById('old');
            const neo = document.createElement('span');
            neo.id = 'new';
            neo.textContent = 'N';
            old.replaceWith(neo);
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#old').length + ':' +
              document.querySelectorAll('#new').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "N:0:1")?;
    Ok(())
}

#[test]
fn insert_adjacent_element_positions_work() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            const a = document.createElement('span');
            a.id = 'a';
            a.textContent = 'A';
            const c = document.createElement('span');
            c.id = 'c';
            c.textContent = 'C';
            const d = document.createElement('span');
            d.id = 'd';
            d.textContent = 'D';
            const e = document.createElement('span');
            e.id = 'e';
            e.textContent = 'E';
            b.insertAdjacentElement('beforebegin', a);
            b.insertAdjacentElement('afterbegin', d);
            b.insertAdjacentElement('beforeend', e);
            b.insertAdjacentElement('afterend', c);
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#a').length + ':' +
              document.querySelectorAll('#c').length + ':' +
              document.querySelector('#b>#d').textContent + ':' +
              document.querySelector('#b>#e').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ADBEC:1:1:D:E")?;
    Ok(())
}

#[test]
fn insert_adjacent_text_positions_and_expression_work() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <input id='v' value='Y'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            b.insertAdjacentText('beforebegin', 'A');
            b.insertAdjacentText('afterbegin', 'X');
            b.insertAdjacentText('beforeend', document.getElementById('v').value);
            b.insertAdjacentText('afterend', 'C');
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' + b.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AXBYC:XBY")?;
    Ok(())
}

#[test]
fn insert_adjacent_html_positions_and_order_work() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            b.insertAdjacentHTML('beforebegin', '<i id="y1">Y</i><i id="y2">Z</i>');
            b.insertAdjacentHTML('afterbegin', 'X<span id="x1">X</span>');
            b.insertAdjacentHTML('beforeend', '<span id="x2">W</span><span id="x3">Q</span>');
            b.insertAdjacentHTML('afterend', 'T<em id="t">T</em>');
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#y1').length + ':' +
              document.querySelectorAll('#y2').length + ':' +
              document.querySelectorAll('#x1').length + ':' +
              document.querySelectorAll('#x2').length + ':' +
              document.querySelectorAll('#x3').length + ':' +
              document.querySelectorAll('#t').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "YZXXBWQTT:1:1:1:1:1:1")?;
    Ok(())
}

#[test]
fn insert_adjacent_html_position_expression_works() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const b = document.getElementById('b');
            let head = 'beforebegin';
            let inner = 'afterbegin';
            let tail = 'AFTEREND';
            b.insertAdjacentHTML(head, '<i id="head">H</i>');
            b.insertAdjacentHTML(inner, '<i id="mid">M</i>');
            b.insertAdjacentHTML(tail, '<i id="tail">T</i>');
            document.getElementById('result').textContent =
              document.getElementById('root').textContent + ':' +
              document.querySelectorAll('#head').length + ':' +
              document.querySelectorAll('#mid').length + ':' +
              document.querySelectorAll('#tail').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "HMBT:1:1:1")?;
    Ok(())
}

#[test]
fn insert_adjacent_html_invalid_position_expression_fails() -> Result<()> {
    let html = r#"
        <div id='root'><span id='b'>B</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const pos = 'outer';
            const b = document.getElementById('b');
            b.insertAdjacentHTML(pos, '<i>T</i>');
            document.getElementById('result').textContent = 'ok';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h.click("#btn").expect_err("invalid position should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("unsupported insertAdjacentHTML position"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn inner_html_set_replaces_children_and_updates_id_index() -> Result<()> {
    let html = r#"
        <div id='box'><span id='old'>O</span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML = '<span id="new">N</span><b>B</b>';
            const same = box.innerHTML === '<span id="new">N</span><b>B</b>';
            document.getElementById('result').textContent =
              box.textContent + ':' +
              document.querySelectorAll('#old').length + ':' +
              document.querySelectorAll('#new').length + ':' +
              same;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "NB:0:1:true")?;
    Ok(())
}

#[test]
fn inner_html_getter_returns_markup_with_text_nodes() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML = 'A<i id="x">X</i>C';
            document.getElementById('result').textContent =
              box.innerHTML + ':' + document.getElementById('x').textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A<i id=\"x\">X</i>C:X")?;
    Ok(())
}

#[test]
fn inner_html_set_sanitizes_scripts_and_dangerous_attrs() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML =
              '<script id="evil">document.getElementById("result").textContent = "pwned";</script>' +
              '<a id="link" href="javascript:alert(1)" onclick="alert(1)">safe</a>';
            document.getElementById('result').textContent =
              document.querySelectorAll('#evil').length + ':' +
              document.getElementById('link').hasAttribute('onclick') + ':' +
              document.getElementById('link').hasAttribute('href') + ':' +
              box.innerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:false:false:<a id=\"link\">safe</a>")?;
    Ok(())
}

#[test]
fn inner_html_getter_escapes_text_and_attr_and_keeps_void_tags() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.innerHTML = '<span id="x" title="a&b"></span><br>';
            document.getElementById('x').textContent = '1 < 2 & 3 > 0';
            document.getElementById('result').textContent = box.innerHTML;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "<span id=\"x\" title=\"a&amp;b\">1 &lt; 2 &amp; 3 &gt; 0</span><br>",
    )?;
    Ok(())
}

#[test]
fn detached_element_id_is_not_queryable_until_attached() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const node = document.createElement('div');
            node.id = 'late';
            document.getElementById('result').textContent =
              document.querySelectorAll('#late').length + ':';
            document.getElementById('root').appendChild(node);
            document.getElementById('result').textContent =
              document.getElementById('result').textContent +
              document.querySelectorAll('#late').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:1")?;
    Ok(())
}

#[test]
fn create_text_node_append_and_remove_work() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const text = document.createTextNode('A');
            root.appendChild(text);
            document.getElementById('result').textContent = root.textContent + ':';
            text.remove();
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + root.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A:")?;
    Ok(())
}

#[test]
fn node_remove_detaches_and_updates_id_index() -> Result<()> {
    let html = r#"
        <div id='root'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const root = document.getElementById('root');
            const el = document.createElement('div');
            el.id = 'gone';
            root.appendChild(el);
            el.remove();
            el.remove();
            document.getElementById('result').textContent =
              document.querySelectorAll('#gone').length + ':' + root.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:")?;
    Ok(())
}

#[test]
fn duplicate_id_prefers_first_match_for_id_selector_api() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='dup'>first</span>
          <span id='dup'>second</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const byId = document.getElementById('dup');
            const all = document.querySelectorAll('#dup').length;
            const bySelector = document.querySelector('#dup');
            document.getElementById('result').textContent =
              byId.textContent + ':' + all + ':' + bySelector.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first:2:first")?;
    Ok(())
}

#[test]
fn duplicate_id_returns_next_match_after_removal_of_first() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='first'>first</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.getElementById('first');
            first.remove();

            const a = document.createElement('span');
            a.id = 'dup';
            a.textContent = 'a';
            const b = document.createElement('span');
            b.id = 'dup';
            b.textContent = 'b';
            const root = document.getElementById('root');
            root.appendChild(a);
            root.appendChild(b);

            const active = document.getElementById('dup');
            const all = document.querySelectorAll('#dup').length;
            document.getElementById('result').textContent =
              active.textContent + ':' + all + ':' + document.querySelectorAll('#first').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:2:0")?;
    Ok(())
}

#[test]
fn selector_child_combinator_and_attr_exists_work() -> Result<()> {
    let html = r#"
        <div id='wrap'>
          <div><span id='nested' data-role='x'></span></div>
          <span id='direct' data-role='x'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.querySelector('#wrap>[data-role]').id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "direct")?;
    Ok(())
}

#[test]
fn selector_group_and_document_order_dedup_work() -> Result<()> {
    let html = r#"
        <div>
          <span id='second'></span>
          <span id='first'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const firstMatch = document.querySelector('#first, #second').id;
            const same = document.querySelectorAll('#first, #first').length;
            const both = document.querySelectorAll('#first, #second').length;
            document.getElementById('result').textContent =
              firstMatch + ':' + same + ':' + both;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "second:1:2")?;
    Ok(())
}

#[test]
fn selector_adjacent_and_general_sibling_combinators_work() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='a' class='item'>A</li>
          <li id='b' class='item'>B</li>
          <li id='c' class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const adjacent = document.querySelector('#a + .item').id;
            const siblings = document.querySelectorAll('#a ~ .item').length;
            document.getElementById('result').textContent = adjacent + ':' + siblings;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "b:2")?;
    Ok(())
}

#[test]
fn selector_compound_tag_id_class_and_attr_work() -> Result<()> {
    let html = r#"
        <div>
          <span id='target' class='x y' data-role='main' data-on='1'></span>
          <span id='other' class='x' data-role='main'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const exact = document.querySelector("span#target.x.y[data-role='main'][data-on]").id;
            const many = document.querySelectorAll("span.x[data-role='main']").length;
            document.getElementById('result').textContent = exact + ':' + many;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "target:2")?;
    Ok(())
}

#[test]
fn selector_attr_operators_work() -> Result<()> {
    let html = r#"
        <div>
          <span id='first'
            data-code='pre-middle-post'
            tags='alpha one beta'
            lang='en-US'></span>
          <span id='second' data-code='other' tags='two three' lang='fr'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const p1 = document.querySelector('[data-code^=\"pre\"]').id;
            const p2 = document.querySelector('[data-code$=\"post\"]').id;
            const p3 = document.querySelector('[data-code*=\"middle\"]').id;
            const p4 = document.querySelector('[tags~=\"one\"]').id;
            const p5 = document.querySelector('[lang|=\"en\"]').id;
            document.getElementById('result').textContent =
              p1 + ':' + p2 + ':' + p3 + ':' + p4 + ':' + p5;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first:first:first:first:first")?;
    Ok(())
}

#[test]
fn selector_attr_empty_and_case_insensitive_key_work() -> Result<()> {
    let html = r#"
        <div>
          <span id='target' data-empty='' data-flag='X'></span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const exact = document.querySelector('[data-empty=\"\"]').id;
            const empty = document.querySelector('[data-empty=]').id;
            const keycase = document.querySelector('[DATA-EMPTY=\"\"]').id;
            document.getElementById('result').textContent = exact + ':' + empty + ':' + keycase;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "target:target:target")?;
    Ok(())
}

#[test]
fn selector_universal_selector_matches_first_element() -> Result<()> {
    let html = r#"
        <div id='root'>
          <section id='first'>A</section>
          <p id='second'>B</p>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = document.querySelector('*').id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "root")?;
    Ok(())
}

#[test]
fn selector_universal_with_class_selector_work() -> Result<()> {
    let html = r#"
        <main id='root'>
          <p id='first' class='x'>A</p>
          <span id='second' class='x'>B</span>
          <div id='third' class='x'>C</div>
        </main>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = document.querySelector('*.x').id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first")?;
    Ok(())
}

#[test]
fn selector_pseudo_classes_work() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first' class='item'>A</li>
          <li id='second' class='item'>B</li>
          <li id='third' class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('.item:first-child').id;
            const last = document.querySelector('.item:last-child').id;
            const second = document.querySelector('li:nth-child(2)').id;
            document.getElementById('result').textContent = first + ':' + last + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first:third:second")?;
    Ok(())
}

#[test]
fn selector_empty_works() -> Result<()> {
    let html = r#"
        <div id='root'><span id='empty'></span><span id='filled'>A</span><span id='nested'><em></em></span></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('#root span:empty').id;
            const total = document.querySelectorAll('#root span:empty').length;
            document.getElementById('result').textContent =
              first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "empty:1")?;
    Ok(())
}

#[test]
fn selector_nth_child_odd_even_work() -> Result<()> {
    let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-child(odd)').id;
            const even = document.querySelector('li:nth-child(even)').id;
            document.getElementById('result').textContent = odd + ':' + even;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "one:two")?;
    Ok(())
}

#[test]
fn selector_nth_child_n_work() -> Result<()> {
    let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const every = document.querySelector('li:nth-child(n)').id;
            const count = document.querySelectorAll('li:nth-child( n )').length;
            document.getElementById('result').textContent = every + ':' + count;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "one:3")?;
    Ok(())
}

#[test]
fn selector_parse_rejects_invalid_nth_child() {
    assert!(
        parse_selector_step("li:nth-child(0)").is_err(),
        "nth-child(0) should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:nth-child(-1)").is_err(),
        "negative nth-child should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:nth-child(2n+)").is_err(),
        "malformed expression nth-child should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-child(n1)").is_err(),
        "invalid expression nth-child should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-last-child(n1)").is_err(),
        "invalid expression nth-last-child should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-last-child(0)").is_err(),
        "nth-last-child(0) should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:nth-last-child(2n+)").is_err(),
        "malformed expression nth-last-child should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-of-type(0)").is_err(),
        "nth-of-type(0) should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:nth-of-type(2n+)").is_err(),
        "malformed expression nth-of-type should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-last-of-type(2n+)").is_err(),
        "malformed expression nth-last-of-type should be rejected"
    );
    assert!(
        parse_selector_step("li:nth-last-of-type(0)").is_err(),
        "nth-last-of-type(0) should be invalid in this engine"
    );
    assert!(
        parse_selector_step("li:not()").is_err(),
        "empty :not should be invalid"
    );

    assert_eq!(
        split_selector_groups("li:not([data='a,b']) , #x").map(|groups| groups.len()),
        Ok(2)
    );
    assert_eq!(
        parse_selector_groups("li:not(.skip, #first), #x").map(|groups| groups.len()),
        Ok(2)
    );
}

#[test]
fn selector_nth_child_an_plus_b_work() -> Result<()> {
    let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
          <li id='five' class='item'>E</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first_odd = document.querySelector('li:nth-child(2n+1)').id;
            const odd_count = document.querySelectorAll('li:nth-child(2n+1)').length;
            const shifted = document.querySelector('li:nth-child(-n+3)').id;
            document.getElementById('result').textContent = first_odd + ':' + odd_count + ':' + shifted;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "one:3:one")?;
    Ok(())
}

#[test]
fn selector_nth_last_child_an_plus_b_work() -> Result<()> {
    let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
          <li id='five' class='item'>E</li>
          <li id='six' class='item'>F</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:nth-last-child(2n+1)').id;
            const count = document.querySelectorAll('li:nth-last-child(2n+1)').length;
            const shifted = document.querySelector('li:nth-last-child(-n+3)').id;
            document.getElementById('result').textContent = first + ':' + count + ':' + shifted;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "two:3:four")?;
    Ok(())
}

#[test]
fn selector_nth_of_type_works() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-of-type(odd)').id;
            const even = document.querySelector('li:nth-of-type(even)').id;
            const exact = document.querySelector('li:nth-of-type(3)').id;
            const expression = document.querySelectorAll('li:nth-of-type(2n)').length;
            document.getElementById('result').textContent = odd + ':' + even + ':' + exact + ':' + expression;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first-li:second-li:third-li:2")?;
    Ok(())
}

#[test]
fn selector_nth_of_type_n_works() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:nth-of-type(n)').id;
            const all = document.querySelectorAll('li:nth-of-type(n)').length;
            document.getElementById('result').textContent = first + ':' + all;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first-li:4")?;
    Ok(())
}

#[test]
fn selector_nth_last_of_type_works() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-last-of-type(odd)').id;
            const even = document.querySelector('li:nth-last-of-type(even)').id;
            const exact = document.querySelector('li:nth-last-of-type(2)').id;
            const expression = document.querySelectorAll('li:nth-last-of-type(2n)').length;
            document.getElementById('result').textContent = odd + ':' + even + ':' + exact + ':' + expression;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "second-li:first-li:third-li:2")?;
    Ok(())
}

#[test]
fn selector_nth_last_of_type_n_works() -> Result<()> {
    let html = r#"
        <ul id='list'>
          <li id='first-li'>A</li>
          <span id='only-span'>S</span>
          <li id='second-li'>B</li>
          <em id='not-li'>E</em>
          <li id='third-li'>C</li>
          <li id='fourth-li'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:nth-last-of-type(n)').id;
            const all = document.querySelectorAll('li:nth-last-of-type(n)').length;
            document.getElementById('result').textContent = first + ':' + all;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "first-li:4")?;
    Ok(())
}

#[test]
fn selector_not_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='a' class='target'>A</span>
          <span id='b'>B</span>
          <span id='c' class='target'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.target)').id;
            const total = document.querySelectorAll('span:not(.target)').length;
            const explicit = document.querySelector('span:not(#b)').id;
            document.getElementById('result').textContent = first + ':' + total + ':' + explicit;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "b:1:a")?;
    Ok(())
}

#[test]
fn selector_nested_not_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <li id='a' class='target'>A</li>
          <li id='b'>B</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const matched = document.querySelector('li:not(:not(.target))').id;
            const total = document.querySelectorAll('li:not(:not(.target))').length;
            document.getElementById('result').textContent = matched + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:1")?;
    Ok(())
}

#[test]
fn selector_not_with_multiple_selectors_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <li id='first' class='target'>A</li>
          <li id='middle'>B</li>
          <li id='skip' class='skip'>C</li>
          <li id='last'>D</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:not(.skip, #first)').id;
            const total = document.querySelectorAll('li:not(.skip, #first)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "middle:2")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_list_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='forbidden' class='scope'>
            <span id='forbidden-a'>A</span>
            <span id='forbidden-b'>B</span>
          </div>
          <span id='skip-me'>C</span>
          <div id='safe'>
            <span id='safe-a'>D</span>
          </div>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope *, #skip-me)').id;
            const total = document.querySelectorAll('span:not(.scope *, #skip-me)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "safe-a:1")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_adjacent_combinator_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='scope' class='scope'></div>
          <span id='excluded'>A</span>
          <span id='included'>B</span>
          <span id='included-2'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope + span)').id;
            const total = document.querySelectorAll('span:not(.scope + span)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "included:2")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_general_sibling_combinator_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='included-before'>A</span>
          <div id='scope' class='scope'></div>
          <span id='excluded-1'>B</span>
          <span id='excluded-2'>C</span>
          <p>between</p>
          <span id='excluded-3'>D</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope ~ span)').id;
            const total = document.querySelectorAll('span:not(.scope ~ span)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "included-before:1")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_list_general_sibling_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <span id='included-before'>A</span>
          <div id='scope' class='scope'></div>
          <span id='excluded-id'>B</span>
          <span id='excluded-sibling'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope ~ span, #excluded-id)').id;
            const total = document.querySelectorAll('span:not(.scope ~ span, #excluded-id)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "included-before:1")?;
    Ok(())
}

#[test]
fn selector_not_with_complex_selector_child_combinator_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='scope' class='scope'>
            <span id='excluded'>A</span>
          </div>
          <span id='included'>B</span>
          <span id='included-2'>C</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('span:not(.scope > span)').id;
            const total = document.querySelectorAll('span:not(.scope > span)').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "included:2")?;
    Ok(())
}

#[test]
fn selector_not_with_multiple_not_chain_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <li id='both' class='foo bar'>A</li>
          <li id='foo-only' class='foo'>B</li>
          <li id='bar-only' class='bar'>C</li>
          <li id='none'>D</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = document.querySelector('li:not(:not(.foo), :not(.bar))').id;
            const total = document.querySelectorAll('li:not(:not(.foo), :not(.bar))').length;
            document.getElementById('result').textContent = first + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "both:1")?;
    Ok(())
}

#[test]
fn selector_first_last_of_type_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <p id='first-p'>A</p>
          <span id='first-span'>B</span>
          <p id='last-p'>C</p>
          <span id='middle-span'>D</span>
          <span id='last-span'>E</span>
          <li id='first-li'>F</li>
          <li id='last-li'>G</li>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const firstSpan = document.querySelector('span:first-of-type').id;
            const lastSpan = document.querySelector('span:last-of-type').id;
            const firstP = document.querySelector('p:first-of-type').id;
            const lastP = document.querySelector('p:last-of-type').id;
            const firstLi = document.querySelector('li:first-of-type').id;
            const lastLi = document.querySelector('li:last-of-type').id;
            document.getElementById('result').textContent = firstSpan + ':' + lastSpan + ':' + firstP + ':' + lastP + ':' + firstLi + ':' + lastLi;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "first-span:last-span:first-p:last-p:first-li:last-li",
    )?;
    Ok(())
}

#[test]
fn selector_only_child_and_only_of_type_works() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='single-p'>
            <p id='lonely-p'>A</p>
          </div>
          <div id='group'>
            <span id='only-span'>B</span>
          </div>
          <section id='mixed-of-type'>
            <span id='mixed-only-span'>C</span>
            <em id='mixed-only-em'>D</em>
          </section>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const lonely = document.querySelector('p:only-child').id;
            const onlySpanInGroup = document.querySelector('#group span:only-child').id;
            const onlySpanOfType = document.querySelector('#mixed-of-type span:only-of-type').id;
            const onlyEmOfType = document.querySelector('#mixed-of-type em:only-of-type').id;
            document.getElementById('result').textContent = lonely + ':' + onlySpanInGroup + ':' + onlySpanOfType + ':' + onlyEmOfType;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "lonely-p:only-span:mixed-only-span:mixed-only-em",
    )?;
    Ok(())
}

#[test]
fn selector_checked_disabled_enabled_works() -> Result<()> {
    let html = r#"
        <input id='enabled' value='ok'>
        <input id='disabled' disabled value='ng'>
        <input id='unchecked' type='checkbox'>
        <input id='checked' type='checkbox' checked>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const checked = document.querySelector('input:checked').id;
            const disabled = document.querySelector('input:disabled').id;
            const enabled = document.querySelector('input:enabled').id;
            document.getElementById('result').textContent = checked + ':' + disabled + ':' + enabled;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "checked:disabled:enabled")?;
    Ok(())
}

#[test]
fn selector_required_optional_readonly_readwrite_works() -> Result<()> {
    let html = r#"
        <input id='r' required value='r'>
        <input id='o'>
        <input id='ro' readonly>
        <input id='rw'>
        <input id='r2'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const required = document.querySelector('input:required').id;
            const optional = document.querySelector('input:optional').id;
            const readOnly = document.querySelector('input:readonly').id;
            const readWrite = document.querySelector('input:read-write').id;
            const summary =
              required + ':' + optional + ':' + readOnly + ':' + readWrite;
            document.getElementById('r').required = false;
            document.getElementById('r2').required = true;
            const afterRequired = document.querySelector('input:required').id;
            const afterOptional = document.querySelector('input:optional').id;
            document.getElementById('result').textContent =
              summary + ':' + afterRequired + ':' + afterOptional;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "r:o:ro:r:r2:r")?;
    Ok(())
}

#[test]
fn selector_trailing_group_separator_is_rejected() -> Result<()> {
    let html = r#"<div id='x'></div>"#;
    let h = Harness::from_html(html)?;
    let err = h
        .assert_exists("#x,")
        .expect_err("selector should be invalid");
    match err {
        Error::UnsupportedSelector(selector) => assert_eq!(selector, "#x,"),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn selector_parse_supports_nth_child_single_arg() {
    let step = parse_selector_step("li:nth-child(2)").expect("parse should succeed");
    assert_eq!(step.tag, Some("li".into()));
    assert_eq!(
        step.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::Exact(2))]
    );
}

#[test]
fn selector_parse_supports_nth_child_odd_even() {
    let odd = parse_selector_step("li:nth-child(odd)").expect("parse should succeed");
    let even = parse_selector_step("li:nth-child(even)").expect("parse should succeed");
    assert_eq!(
        odd.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::Odd)]
    );
    assert_eq!(
        even.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::Even)]
    );
}

#[test]
fn selector_parse_supports_nth_child_n() {
    let n = parse_selector_step("li:nth-child(n)").expect("parse should succeed");
    assert_eq!(
        n.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::AnPlusB(
            1, 0
        ))]
    );
}

#[test]
fn selector_parse_supports_nth_last_child_an_plus_b() {
    let direct = parse_selector_step("li:nth-last-child(2n+1)").expect("parse should succeed");
    assert_eq!(
        direct.pseudo_classes,
        vec![SelectorPseudoClass::NthLastChild(
            NthChildSelector::AnPlusB(2, 1)
        )]
    );
}

#[test]
fn selector_parse_supports_nth_last_child_odd_even_exact() {
    let odd = parse_selector_step("li:nth-last-child(odd)").expect("parse should succeed");
    let even = parse_selector_step("li:nth-last-child(even)").expect("parse should succeed");
    let exact = parse_selector_step("li:nth-last-child(2)").expect("parse should succeed");
    assert_eq!(
        odd.pseudo_classes,
        vec![SelectorPseudoClass::NthLastChild(NthChildSelector::Odd)]
    );
    assert_eq!(
        even.pseudo_classes,
        vec![SelectorPseudoClass::NthLastChild(NthChildSelector::Even)]
    );
    assert_eq!(
        exact.pseudo_classes,
        vec![SelectorPseudoClass::NthLastChild(NthChildSelector::Exact(
            2
        ))]
    );
}

#[test]
fn selector_parse_supports_nth_child_an_plus_b() {
    let direct = parse_selector_step("li:nth-child(2n+1)").expect("parse should succeed");
    let shifted = parse_selector_step("li:nth-child(-n+3)").expect("parse should succeed");
    assert_eq!(
        direct.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::AnPlusB(
            2, 1
        ))]
    );
    assert_eq!(
        shifted.pseudo_classes,
        vec![SelectorPseudoClass::NthChild(NthChildSelector::AnPlusB(
            -1, 3
        ))]
    );
}

#[test]
fn selector_parse_supports_first_last_of_type() {
    let first = parse_selector_step("li:first-of-type").expect("parse should succeed");
    let last = parse_selector_step("li:last-of-type").expect("parse should succeed");
    assert_eq!(first.pseudo_classes, vec![SelectorPseudoClass::FirstOfType]);
    assert_eq!(last.pseudo_classes, vec![SelectorPseudoClass::LastOfType]);
}

#[test]
fn selector_parse_supports_empty() {
    let parsed = parse_selector_step("span:empty").expect("parse should succeed");
    assert_eq!(parsed.pseudo_classes, vec![SelectorPseudoClass::Empty]);
}

#[test]
fn selector_parse_supports_only_child_and_only_of_type() {
    let only_child = parse_selector_step("li:only-child").expect("parse should succeed");
    let only_of_type = parse_selector_step("li:only-of-type").expect("parse should succeed");
    assert_eq!(
        only_child.pseudo_classes,
        vec![SelectorPseudoClass::OnlyChild]
    );
    assert_eq!(
        only_of_type.pseudo_classes,
        vec![SelectorPseudoClass::OnlyOfType]
    );
}

#[test]
fn selector_parse_supports_checked_disabled_enabled() {
    let checked = parse_selector_step("input:checked").expect("parse should succeed");
    let disabled = parse_selector_step("input:disabled").expect("parse should succeed");
    let enabled = parse_selector_step("input:enabled").expect("parse should succeed");
    assert_eq!(checked.pseudo_classes, vec![SelectorPseudoClass::Checked]);
    assert_eq!(disabled.pseudo_classes, vec![SelectorPseudoClass::Disabled]);
    assert_eq!(enabled.pseudo_classes, vec![SelectorPseudoClass::Enabled]);
}

#[test]
fn selector_parse_supports_required_optional_readonly_readwrite() {
    let required = parse_selector_step("input:required").expect("parse should succeed");
    let optional = parse_selector_step("input:optional").expect("parse should succeed");
    let read_only = parse_selector_step("input:read-only").expect("parse should succeed");
    let read_only_alias = parse_selector_step("input:readonly").expect("parse should succeed");
    let read_write = parse_selector_step("input:read-write").expect("parse should succeed");
    assert_eq!(required.pseudo_classes, vec![SelectorPseudoClass::Required]);
    assert_eq!(optional.pseudo_classes, vec![SelectorPseudoClass::Optional]);
    assert_eq!(
        read_only.pseudo_classes,
        vec![SelectorPseudoClass::Readonly]
    );
    assert_eq!(
        read_only_alias.pseudo_classes,
        vec![SelectorPseudoClass::Readonly]
    );
    assert_eq!(
        read_write.pseudo_classes,
        vec![SelectorPseudoClass::Readwrite]
    );
}

#[test]
fn selector_parse_supports_focus_and_focus_within() {
    let focus = parse_selector_step("input:focus").expect("parse should succeed");
    let focus_within = parse_selector_step("div:focus-within").expect("parse should succeed");
    assert_eq!(focus.pseudo_classes, vec![SelectorPseudoClass::Focus]);
    assert_eq!(
        focus_within.pseudo_classes,
        vec![SelectorPseudoClass::FocusWithin]
    );
}

#[test]
fn selector_parse_supports_active() {
    let active = parse_selector_step("button:active").expect("parse should succeed");
    assert_eq!(active.pseudo_classes, vec![SelectorPseudoClass::Active]);
}

#[test]
fn selector_parse_supports_not() {
    let by_id = parse_selector_step("span:not(#x)").expect("parse should succeed");
    let by_class = parse_selector_step("span:not(.x)").expect("parse should succeed");
    let nested = parse_selector_step("span:not(:not(.x))").expect("parse should succeed");
    let with_attribute = parse_selector_step("li:not([data='a,b'])").expect("parse should succeed");
    if let SelectorPseudoClass::Not(inners) = &by_id.pseudo_classes[0] {
        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 1);
        assert_eq!(inners[0][0].step.id.as_deref(), Some("x"));
    } else {
        panic!("expected not pseudo");
    }
    if let SelectorPseudoClass::Not(inners) = &by_class.pseudo_classes[0] {
        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 1);
        assert_eq!(inners[0][0].step.tag.as_deref(), None);
        assert_eq!(inners[0][0].step.classes.as_slice(), &["x"]);
    } else {
        panic!("expected not pseudo");
    }
    if let SelectorPseudoClass::Not(inners) = &nested.pseudo_classes[0] {
        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 1);
        if let SelectorPseudoClass::Not(inner_inners) = &inners[0][0].step.pseudo_classes[0] {
            assert_eq!(inner_inners.len(), 1);
            assert_eq!(inner_inners[0][0].step.tag.as_deref(), None);
            assert_eq!(inner_inners[0][0].step.classes.as_slice(), &["x"]);
            assert!(inner_inners[0][0].step.pseudo_classes.is_empty());
        } else {
            panic!("expected nested not pseudo");
        }
    } else {
        panic!("expected not pseudo");
    }
    if let SelectorPseudoClass::Not(inners) = &with_attribute.pseudo_classes[0] {
        assert_eq!(inners.len(), 1);
        assert_eq!(inners[0].len(), 1);
        let inner = &inners[0][0].step;
        assert_eq!(
            inner.attrs,
            vec![SelectorAttrCondition::Eq {
                key: "data".into(),
                value: "a,b".into()
            }]
        );
        assert!(inner.classes.is_empty());
        assert!(inner.id.is_none());
        assert!(inner.pseudo_classes.is_empty());
        assert!(!inner.universal);
    } else {
        panic!("expected not pseudo");
    }
}

#[test]
fn selector_parse_supports_where_is_and_has() {
    let where_step =
        parse_selector_step("span:where(.a, #b, :not(.skip))").expect("parse should succeed");
    let is_step =
        parse_selector_step("span:is(.a, #b, :not(.skip))").expect("parse should succeed");
    let has_step = parse_selector_step("section:has(.c, #d)").expect("parse should succeed");

    assert!(matches!(
        where_step.pseudo_classes[0],
        SelectorPseudoClass::Where(_)
    ));
    if let SelectorPseudoClass::Where(inners) = &where_step.pseudo_classes[0] {
        assert_eq!(inners.len(), 3);
        assert_eq!(inners[0].len(), 1);
        assert_eq!(inners[1].len(), 1);
        assert_eq!(inners[2].len(), 1);
    }

    assert!(matches!(
        is_step.pseudo_classes[0],
        SelectorPseudoClass::Is(_)
    ));
    assert!(matches!(
        has_step.pseudo_classes[0],
        SelectorPseudoClass::Has(_)
    ));
}

#[test]
fn selector_parse_supports_attribute_operators() {
    let exists = parse_selector_step("[flag]").expect("parse should succeed");
    let eq = parse_selector_step("[data='value']").expect("parse should succeed");
    let starts_with = parse_selector_step("[data^='pre']").expect("parse should succeed");
    let ends_with = parse_selector_step("[data$='post']").expect("parse should succeed");
    let contains = parse_selector_step("[data*='med']").expect("parse should succeed");
    let includes = parse_selector_step("[tags~='one']").expect("parse should succeed");
    let dash = parse_selector_step("[lang|='en']").expect("parse should succeed");

    assert_eq!(
        exists.attrs,
        vec![SelectorAttrCondition::Exists { key: "flag".into() }]
    );
    assert_eq!(
        eq.attrs,
        vec![SelectorAttrCondition::Eq {
            key: "data".into(),
            value: "value".into()
        }]
    );
    assert_eq!(
        starts_with.attrs,
        vec![SelectorAttrCondition::StartsWith {
            key: "data".into(),
            value: "pre".into()
        }]
    );
    assert_eq!(
        ends_with.attrs,
        vec![SelectorAttrCondition::EndsWith {
            key: "data".into(),
            value: "post".into()
        }]
    );
    assert_eq!(
        contains.attrs,
        vec![SelectorAttrCondition::Contains {
            key: "data".into(),
            value: "med".into()
        }]
    );
    assert_eq!(
        includes.attrs,
        vec![SelectorAttrCondition::Includes {
            key: "tags".into(),
            value: "one".into()
        }]
    );
    assert_eq!(
        dash.attrs,
        vec![SelectorAttrCondition::DashMatch {
            key: "lang".into(),
            value: "en".into()
        }]
    );
    let empty = parse_selector_step("[data='']").expect("parse should succeed");
    let case_key = parse_selector_step("[DATA='v']").expect("parse should succeed");
    let unquoted_empty = parse_selector_step("[data=]").expect("parse should succeed");
    assert_eq!(
        empty.attrs,
        vec![SelectorAttrCondition::Eq {
            key: "data".into(),
            value: "".into()
        }]
    );
    assert_eq!(
        case_key.attrs,
        vec![SelectorAttrCondition::Eq {
            key: "data".into(),
            value: "v".into()
        }]
    );
    assert_eq!(
        unquoted_empty.attrs,
        vec![SelectorAttrCondition::Eq {
            key: "data".into(),
            value: "".into()
        }]
    );
}

#[test]
fn selector_parse_supports_not_with_multiple_selectors() {
    let multi =
        parse_selector_step("li:not(.a, #target, :not(.skip))").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &multi.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };
    assert_eq!(inners.len(), 3);
    assert_eq!(inners[0].len(), 1);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["a"]);

    assert_eq!(inners[1].len(), 1);
    assert_eq!(inners[1][0].step.id.as_deref(), Some("target"));

    assert_eq!(inners[2].len(), 1);
    assert_eq!(inners[2][0].step.pseudo_classes.len(), 1);
    let inner = &inners[2][0].step.pseudo_classes[0];
    assert!(matches!(inner, SelectorPseudoClass::Not(_)));
}

#[test]
fn selector_parse_supports_not_with_multiple_not_pseudos() {
    let parsed =
        parse_selector_step("li:not(:not(.foo), :not(.bar))").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 2);

    assert_eq!(inners[0].len(), 1);
    assert_eq!(inners[0][0].step.pseudo_classes.len(), 1);
    let first = &inners[0][0].step.pseudo_classes[0];
    if let SelectorPseudoClass::Not(inner_inners) = first {
        assert_eq!(inner_inners.len(), 1);
        assert_eq!(inner_inners[0][0].step.classes.as_slice(), &["foo"]);
    } else {
        panic!("expected nested not pseudo in first arg");
    }

    assert_eq!(inners[1].len(), 1);
    assert_eq!(inners[1][0].step.pseudo_classes.len(), 1);
    let second = &inners[1][0].step.pseudo_classes[0];
    if let SelectorPseudoClass::Not(inner_inners) = second {
        assert_eq!(inner_inners.len(), 1);
        assert_eq!(inner_inners[0][0].step.classes.as_slice(), &["bar"]);
    } else {
        panic!("expected nested not pseudo in second arg");
    }
}

#[test]
fn selector_parse_supports_not_with_complex_selector_list() {
    let parsed = parse_selector_step("span:not(.scope *, #skip-me, .area :not(.nested .leaf))")
        .expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 3);

    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert!(inners[0][0].combinator.is_none());
    assert_eq!(inners[0][1].step.tag.as_deref(), None);
    assert!(inners[0][1].step.universal);
    assert_eq!(
        inners[0][1].combinator,
        Some(SelectorCombinator::Descendant)
    );

    assert_eq!(inners[1].len(), 1);
    assert_eq!(inners[1][0].step.id.as_deref(), Some("skip-me"));
    assert!(inners[1][0].combinator.is_none());

    assert_eq!(inners[2].len(), 2);
    assert_eq!(inners[2][0].step.classes.as_slice(), &["area"]);
    assert_eq!(inners[2][1].step.pseudo_classes.len(), 1);
    let nested = &inners[2][1].step.pseudo_classes[0];
    if let SelectorPseudoClass::Not(nested_inners) = nested {
        assert_eq!(nested_inners.len(), 1);
        assert_eq!(nested_inners[0].len(), 2);
        assert_eq!(nested_inners[0][0].step.classes.as_slice(), &["nested"]);
        assert_eq!(nested_inners[0][1].step.classes.as_slice(), &["leaf"]);
        assert_eq!(
            nested_inners[0][1].combinator,
            Some(SelectorCombinator::Descendant)
        );
    } else {
        panic!("expected nested not pseudo");
    }
}

#[test]
fn selector_parse_supports_not_with_adjacent_selector() {
    let parsed = parse_selector_step("span:not(.scope + span)").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 1);
    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
    assert_eq!(
        inners[0][1].combinator,
        Some(SelectorCombinator::AdjacentSibling)
    );
}

#[test]
fn selector_parse_supports_not_with_selector_list_general_sibling_selector() {
    let parsed =
        parse_selector_step("span:not(.scope ~ span, #excluded-id)").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 2);
    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
    assert_eq!(
        inners[0][1].combinator,
        Some(SelectorCombinator::GeneralSibling)
    );

    assert_eq!(inners[1].len(), 1);
    assert_eq!(inners[1][0].step.id.as_deref(), Some("excluded-id"));
    assert!(inners[1][0].combinator.is_none());
}

#[test]
fn selector_parse_supports_not_with_general_sibling_selector() {
    let parsed = parse_selector_step("span:not(.scope ~ span)").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 1);
    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
    assert_eq!(
        inners[0][1].combinator,
        Some(SelectorCombinator::GeneralSibling)
    );
}

#[test]
fn selector_parse_supports_not_with_child_selector() {
    let parsed = parse_selector_step("span:not(.scope > span)").expect("parse should succeed");
    let SelectorPseudoClass::Not(inners) = &parsed.pseudo_classes[0] else {
        panic!("expected not pseudo");
    };

    assert_eq!(inners.len(), 1);
    assert_eq!(inners[0].len(), 2);
    assert_eq!(inners[0][0].step.classes.as_slice(), &["scope"]);
    assert_eq!(inners[0][1].step.tag.as_deref(), Some("span"));
    assert_eq!(inners[0][1].combinator, Some(SelectorCombinator::Child));
}

#[test]
fn selector_parse_rejects_invalid_not_argument_forms() {
    assert!(parse_selector_step("span:not()").is_err());
    assert!(parse_selector_step("span:not(,)").is_err());
    assert!(parse_selector_step("span:not(.a,,#b)").is_err());
    assert!(parse_selector_step("span:not(.a,").is_err());
    assert!(parse_selector_step("span:not(.a,#b,)").is_err());
}

#[test]
fn selector_parse_rejects_unclosed_not_parenthesis() {
    assert!(parse_selector_step("span:not(.a, #b").is_err());
    assert!(parse_selector_step("span:not(:not(.a)").is_err());
}

#[test]
fn selector_runtime_rejects_invalid_not_selector() -> Result<()> {
    let html = "<div id='root'></div>";
    let h = Harness::from_html(html)?;

    let err = h
        .assert_exists("span:not()")
        .expect_err("invalid selector should be rejected");
    match err {
        Error::UnsupportedSelector(selector) => assert_eq!(selector, "span:not()"),
        other => panic!("expected unsupported selector error, got: {other:?}"),
    }

    let err = h
        .assert_exists("span:not(.a,)")
        .expect_err("invalid selector should be rejected");
    match err {
        Error::UnsupportedSelector(selector) => assert_eq!(selector, "span:not(.a,)"),
        other => panic!("expected unsupported selector error, got: {other:?}"),
    }

    Ok(())
}

#[test]
fn selector_parse_supports_nth_of_type() {
    let odd = parse_selector_step("li:nth-of-type(odd)").expect("parse should succeed");
    let expr = parse_selector_step("li:nth-of-type(2n)").expect("parse should succeed");
    let n = parse_selector_step("li:nth-of-type(n)").expect("parse should succeed");
    let exact = parse_selector_step("li:nth-of-type(3)").expect("parse should succeed");
    assert_eq!(
        odd.pseudo_classes,
        vec![SelectorPseudoClass::NthOfType(NthChildSelector::Odd)]
    );
    assert_eq!(
        expr.pseudo_classes,
        vec![SelectorPseudoClass::NthOfType(NthChildSelector::AnPlusB(
            2, 0
        ))]
    );
    assert_eq!(
        n.pseudo_classes,
        vec![SelectorPseudoClass::NthOfType(NthChildSelector::AnPlusB(
            1, 0
        ))]
    );
    assert_eq!(
        exact.pseudo_classes,
        vec![SelectorPseudoClass::NthOfType(NthChildSelector::Exact(3))]
    );
}

#[test]
fn selector_parse_supports_nth_last_of_type() {
    let odd = parse_selector_step("li:nth-last-of-type(odd)").expect("parse should succeed");
    let even = parse_selector_step("li:nth-last-of-type(even)").expect("parse should succeed");
    let n = parse_selector_step("li:nth-last-of-type(n)").expect("parse should succeed");
    let exact = parse_selector_step("li:nth-last-of-type(2)").expect("parse should succeed");
    assert_eq!(
        odd.pseudo_classes,
        vec![SelectorPseudoClass::NthLastOfType(NthChildSelector::Odd)]
    );
    assert_eq!(
        even.pseudo_classes,
        vec![SelectorPseudoClass::NthLastOfType(NthChildSelector::Even)]
    );
    assert_eq!(
        n.pseudo_classes,
        vec![SelectorPseudoClass::NthLastOfType(
            NthChildSelector::AnPlusB(1, 0)
        )]
    );
    assert_eq!(
        exact.pseudo_classes,
        vec![SelectorPseudoClass::NthLastOfType(NthChildSelector::Exact(
            2
        ))]
    );
}

#[test]
fn selector_nth_last_child_odd_even_work() -> Result<()> {
    let html = r#"
        <ul>
          <li id='one' class='item'>A</li>
          <li id='two' class='item'>B</li>
          <li id='three' class='item'>C</li>
          <li id='four' class='item'>D</li>
          <li id='five' class='item'>E</li>
          <li id='six' class='item'>F</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const odd = document.querySelector('li:nth-last-child(odd)').id;
            const even = document.querySelector('li:nth-last-child(even)').id;
            const second_last = document.querySelector('li:nth-last-child(2)').id;
            const total = document.querySelectorAll('li:nth-last-child(odd)').length;
            document.getElementById('result').textContent = odd + ':' + even + ':' + second_last + ':' + total;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "two:one:five:3")?;
    Ok(())
}

#[test]
fn radio_group_exclusive_selection_works() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='r1' type='radio' name='plan'>
          <input id='r2' type='radio' name='plan'>
        </form>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#r1")?;
    h.assert_checked("#r1", true)?;
    h.assert_checked("#r2", false)?;
    h.click("#r2")?;
    h.assert_checked("#r1", false)?;
    h.assert_checked("#r2", true)?;
    Ok(())
}

#[test]
fn radio_checked_property_assignment_preserves_group_exclusivity() -> Result<()> {
    let html = r#"
        <form id='f1'>
          <input id='r1' type='radio' name='plan'>
          <input id='r2' type='radio' name='plan'>
        </form>
        <form id='f2'>
          <input id='r3' type='radio' name='plan'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('r1').checked = true;
            document.getElementById('r3').checked = true;
            document.getElementById('r2').checked = true;
            document.getElementById('result').textContent =
              document.getElementById('r1').checked + ':' +
              document.getElementById('r2').checked + ':' +
              document.getElementById('r3').checked;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true:true")?;
    Ok(())
}

#[test]
fn radio_group_defaults_are_normalized_on_parse_and_form_reset() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='r1' type='radio' name='plan' checked>
          <input id='r2' type='radio' name='plan' checked>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('r1').checked = true;
            document.getElementById('f').reset();
            document.getElementById('result').textContent =
              document.getElementById('r1').checked + ':' +
              document.getElementById('r2').checked;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.assert_checked("#r1", false)?;
    h.assert_checked("#r2", true)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true")?;
    Ok(())
}

#[test]
fn disabled_controls_ignore_user_actions() -> Result<()> {
    let html = r#"
        <input id='name' disabled value='init'>
        <input id='agree' type='checkbox' disabled checked>
        <p id='result'></p>
        <script>
          document.getElementById('name').addEventListener('input', () => {
            document.getElementById('result').textContent = 'name-input';
          });
          document.getElementById('agree').addEventListener('change', () => {
            document.getElementById('result').textContent = 'agree-change';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "next")?;
    h.assert_value("#name", "init")?;
    h.assert_text("#result", "")?;

    h.click("#agree")?;
    h.assert_checked("#agree", true)?;
    h.assert_text("#result", "")?;

    h.set_checked("#agree", false)?;
    h.assert_checked("#agree", true)?;
    h.assert_text("#result", "")?;
    Ok(())
}

#[test]
fn disabled_property_prevents_user_actions_and_can_be_cleared() -> Result<()> {
    let html = r#"
        <input id='name' value='init'>
        <input id='agree' type='checkbox' checked>
        <button id='disable'>disable</button>
        <button id='enable'>enable</button>
        <p id='result'></p>
        <script>
          document.getElementById('disable').addEventListener('click', () => {
            document.getElementById('name').disabled = true;
            document.getElementById('agree').disabled = true;
          });
          document.getElementById('enable').addEventListener('click', () => {
            document.getElementById('name').disabled = false;
            document.getElementById('agree').disabled = false;
          });
          document.getElementById('name').addEventListener('input', () => {
            document.getElementById('result').textContent = 'name-input';
          });
          document.getElementById('agree').addEventListener('change', () => {
            document.getElementById('result').textContent = 'agree-change';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#disable")?;

    h.type_text("#name", "next")?;
    h.assert_value("#name", "init")?;
    h.click("#agree")?;
    h.assert_checked("#agree", true)?;
    h.assert_text("#result", "")?;

    h.click("#enable")?;
    h.type_text("#name", "next")?;
    h.set_checked("#agree", false)?;
    h.assert_value("#name", "next")?;
    h.assert_checked("#agree", false)?;
    Ok(())
}

#[test]
fn assignment_and_remainder_expressions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 20;
            n += 5;
            n -= 3;
            n *= 2;
            n /= 4;
            n %= 6;
            const eq = (10 % 3) == 1;
            const neq = (10 % 3) != 2;
            document.getElementById('result').textContent =
              n + ':' + (eq ? 'eq' : 'neq') + ':' + (neq ? 'neq' : 'eq');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5:eq:neq")?;
    Ok(())
}

#[test]
fn loose_equality_and_inequality_follow_js_coercion_rules() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = 0 == false;
            const b = 1 == true;
            const c = '' == 0;
            const d = ' \t ' == 0;
            const e = '1' == 1;
            const f = null == undefined;
            const g = null == 0;
            const h = undefined == 0;
            const i = [1] == 1;
            const j = [] == '';
            const k = ({ a: 1 }) == '[object Object]';
            const l = '1' != 1;
            const m = '2' != 1;
            const n = 0 === false;
            const o = 0 !== false;
            const p = NaN == NaN;
            const q = NaN != NaN;
            const arr = [1];
            const r = arr == arr;
            const s = arr != arr;
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' + h + ':' +
              i + ':' + j + ':' + k + ':' + l + ':' + m + ':' + n + ':' + o + ':' + p + ':' +
              q + ':' + r + ':' + s;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "true:true:true:true:true:true:false:false:true:true:true:false:true:false:true:false:true:true:false",
        )?;
    Ok(())
}

#[test]
fn unary_plus_works_as_numeric_expression() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = '12';
            const value = +text;
            const direct = +'-3.5';
            const paren = +('+7');
            document.getElementById('result').textContent =
              value + ':' + direct + ':' + paren;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12:-3.5:7")?;
    Ok(())
}

#[test]
fn bitwise_expression_supports_binary_operations() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const bit_and = 5 & 3;
            const bit_or = 5 | 2;
            const bit_xor = 5 ^ 1;
            const left = 1 + 2 << 2;
            const masked = 5 + 2 & 4;
            const shift = 8 >>> 1;
            const signed_shift = -8 >> 1;
            const unsigned_shift = (-1) >>> 1;
            const inv = ~1;
            document.getElementById('result').textContent =
              bit_and + ':' + bit_or + ':' + bit_xor + ':' + left + ':' + masked + ':' +
              shift + ':' + signed_shift + ':' + unsigned_shift + ':' + inv;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:7:4:12:4:4:-4:2147483647:-2")?;
    Ok(())
}

#[test]
fn bitwise_compound_assignment_operators_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 6;
            n &= 3;
            n |= 4;
            n ^= 1;
            n <<= 1;
            n >>= 1;
            n >>>= 1;
            document.getElementById('result').textContent = n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn exponentiation_expression_and_compound_assignment_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const value = 2 ** 3 ** 2;
            const with_mul = 2 * 3 ** 2;
            const grouped = (2 + 2) ** 3;
            let n = 2;
            n **= 3;
            document.getElementById('result').textContent =
              value + ':' + with_mul + ':' + grouped + ':' + n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "512:18:64:8")?;
    Ok(())
}

#[test]
fn update_statements_change_identifier_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 1;
            ++n;
            n++;
            --n;
            n--;
            document.getElementById('result').textContent = n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn typeof_operator_works_for_known_and_undefined_identifiers() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const known = 1;
            const a = typeof known;
            const b = typeof unknownName;
            const c = typeof false;
            document.getElementById('result').textContent = a + ':' + b + ':' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "number:undefined:boolean")?;
    Ok(())
}

#[test]
fn undefined_void_delete_and_special_literals_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const known = 1;
            const is_void = void known;
            const a = typeof undefined;
            const b = typeof is_void;
            const c = typeof NaN;
            const d = typeof Infinity;
            const e = is_void === undefined;
            const f = delete known;
            const g = delete missing;
            const h = NaN === NaN;
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g + ':' + h;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "undefined:undefined:number:number:true:false:true:false",
    )?;
    Ok(())
}

#[test]
fn await_operator_supports_values_and_fulfilled_promises() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const direct = await 7;
            const promised = await Promise.resolve('ok');
            document.getElementById('result').textContent = direct + ':' + promised;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "7:ok")?;
    Ok(())
}

#[test]
fn async_function_declaration_and_expression_return_promises() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function resolveNow(value) {
            return Promise.resolve(value);
          }

          async function asyncDecl() {
            const first = await resolveNow('A');
            return first + 'B';
          }

          const asyncExpr = async function(value = 'C') {
            const second = await Promise.resolve(value);
            return second + 'D';
          };

          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const p1 = asyncDecl();
            const p2 = asyncExpr();
            result.textContent = typeof p1;
            Promise.all([p1, p2]).then((values) => {
              result.textContent = result.textContent + ':' + values[0] + ':' + values[1];
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "object:AB:CD")?;
    Ok(())
}

#[test]
fn async_function_returned_promise_reference_differs_from_returned_value() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const p = Promise.resolve(1);

          async function asyncReturn() {
            return p;
          }

          function basicReturn() {
            return Promise.resolve(p);
          }

          document.getElementById('btn').addEventListener('click', () => {
            const sameBasic = p === basicReturn();
            const sameAsync = p === asyncReturn();
            document.getElementById('result').textContent = sameBasic + ':' + sameAsync;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false")?;
    Ok(())
}

#[test]
fn async_function_errors_reject_promise_instead_of_throwing_synchronously() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          async function explode() {
            missingFunction();
            return 'never';
          }

          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const promise = explode();
            result.textContent = 'called';
            promise.catch(() => {
              result.textContent = result.textContent + ':caught';
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "called:caught")?;
    Ok(())
}

#[test]
fn nullish_coalescing_operator_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = null ?? 'x';
            const b = undefined ?? 'y';
            const c = false ?? 'z';
            const d = 0 ?? 9;
            const e = '' ?? 'fallback';
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "x:y:false:0:")?;
    Ok(())
}

#[test]
fn logical_assignment_operators_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let a = 0;
            let b = 2;
            let c = null;
            let d = 'keep';
            let e = 0;
            let f = 'set';

            a ||= 5;
            b &&= 7;
            c ??= 9;
            d ||= 'alt';
            e &&= 4;
            f ??= 'x';

            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5:7:9:keep:0:set")?;
    Ok(())
}

#[test]
fn destructuring_assignment_for_array_and_object_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let first = 0;
            let second = 2;
            let third = 0;
            let a = '';
            let b = '';

            [first, , third] = [10, 20, 30];
            { a, b } = { a: 'A', b: 'B', c: 'C' };

            document.getElementById('result').textContent =
              first + ':' + second + ':' + third + ':' + a + ':' + b;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "10:2:30:A:B")?;
    Ok(())
}

#[test]
fn yield_and_yield_star_operators_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = yield 3;
            const b = yield* (2 + 3);
            document.getElementById('result').textContent = a + ':' + b;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:5")?;
    Ok(())
}

#[test]
fn spread_syntax_for_array_and_object_literals_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const base = [2, 3];
            const arr = [1, ...base, 4];
            const obj1 = { a: 1, b: 2 };
            const obj2 = { ...obj1, b: 9, c: 3 };
            document.getElementById('result').textContent =
              arr.join(',') + '|' + obj2.a + ':' + obj2.b + ':' + obj2.c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1,2,3,4|1:9:3")?;
    Ok(())
}

#[test]
fn comma_operator_returns_last_value_in_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const x = (1, 2, 3);
            const y = (alert('first'), alert('second'), 'ok');
            document.getElementById('result').textContent = x + ':' + y;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:ok")?;
    assert_eq!(
        h.take_alert_messages(),
        vec!["first".to_string(), "second".to_string()]
    );
    Ok(())
}

#[test]
fn in_operator_works_with_query_selector_all_indexes() -> Result<()> {
    let html = r#"
        <div id='a'>A</div>
        <div id='b'>B</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const nodes = document.querySelectorAll('#a, #b');
            const a = 0 in nodes;
            const b = 1 in nodes;
            const c = 2 in nodes;
            const d = '1' in nodes;
            document.getElementById('result').textContent = a + ':' + b + ':' + c + ':' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:false:true")?;
    Ok(())
}

#[test]
fn instanceof_operator_works_with_node_membership_and_identity() -> Result<()> {
    let html = r#"
        <div id='a'>A</div>
        <div id='b'>B</div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a_node = document.getElementById('a');
            const b_node = document.getElementById('b');
            const a_only = document.querySelectorAll('#a');
            const same = a_node instanceof a_node;
            const member = a_node instanceof a_only;
            const other = b_node instanceof a_only;
            document.getElementById('result').textContent = same + ':' + member + ':' + other;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn regex_literal_test_and_exec_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = /ab+c/i;
            const ok1 = re.test('xxABBCyy');
            const ok2 = /foo.bar/s.test('foo\nbar');
            const hit = /(ab)(cd)/.exec('xabcdz');
            document.getElementById('result').textContent =
              ok1 + ':' + ok2 + ':' + hit[0] + ':' + hit[1] + ':' + hit[2];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:abcd:ab:cd")?;
    Ok(())
}

#[test]
fn regexp_constructor_and_global_sticky_exec_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = new RegExp('a.', 'g');
            const m1 = re.exec('a1a2');
            const m2 = re.exec('a1a2');
            const m3 = re.exec('a1a2');

            const sticky = /a./y;
            const y1 = sticky.exec('a1xa2');
            const y2 = sticky.exec('a1xa2');
            const y3 = sticky.exec('a1xa2');

            document.getElementById('result').textContent =
              m1[0] + ':' + m2[0] + ':' + m3 + ':' +
              y1[0] + ':' + y2 + ':' + y3[0];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a1:a2:null:a1:null:a1")?;
    Ok(())
}

#[test]
fn regex_parse_and_runtime_errors_are_reported() -> Result<()> {
    let parse_err = Harness::from_html("<script>const re = /a/gg;</script>")
        .expect_err("duplicate regex flags should fail during parse");
    match parse_err {
        Error::ScriptParse(msg) => assert!(msg.contains("invalid regular expression flags")),
        other => panic!("unexpected regex parse error: {other:?}"),
    }

    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            new RegExp('(', 'g');
          });
        </script>
        "#;
    let mut h = Harness::from_html(html)?;
    let runtime_err = h
        .click("#btn")
        .expect_err("invalid RegExp constructor pattern should fail");
    match runtime_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("invalid regular expression")),
        other => panic!("unexpected regex runtime error: {other:?}"),
    }

    Ok(())
}

#[test]
fn regexp_constructor_properties_and_escape_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = RegExp('a.', 'gimsydu');
            re.lastIndex = 3.8;
            const info =
              re.source + ':' + re.flags + ':' +
              re.global + ':' + re.ignoreCase + ':' + re.multiline + ':' +
              re.dotAll + ':' + re.sticky + ':' + re.hasIndices + ':' +
              re.unicode + ':' + re.unicodeSets + ':' +
              re.lastIndex + ':' + (re.constructor === RegExp) + ':' + typeof RegExp;
            const escaped = RegExp.escape('a+b*c?');
            const escapedWindow = window.RegExp.escape('x.y');
            document.getElementById('result').textContent =
              info + '|' + escaped + '|' + escapedWindow;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "a.:gimsydu:true:true:true:true:true:true:true:false:3:true:function|a\\+b\\*c\\?|x\\.y",
    )?;
    Ok(())
}

#[test]
fn regexp_string_match_split_and_replace_examples_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = /(\w+)\s(\w+)/;
            const changed = 'Maria Cruz'.replace(re, '$2, $1');
            const text = 'Some text\nAnd some more\r\nAnd yet\nThis is the end';
            const lines = text.split(/\r?\n/);
            const multi = 'Please yes\nmake my day!';
            const noDotAll = multi.match(/yes.*day/) === null;
            const withDotAll = multi.match(/yes.*day/s);
            const withDotAllOk = withDotAll[0] === 'yes\nmake my day';
            const order = 'Let me get some bacon and eggs, please';
            const picks = order.match(new RegExp('\\b(bacon|eggs)\\b', 'g'));

            document.getElementById('result').textContent =
              changed + '|' +
              lines[0] + ':' + lines[1] + ':' + lines[2] + ':' + lines[3] + '|' +
              noDotAll + ':' + withDotAllOk + '|' +
              picks[0] + ':' + picks[1];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "Cruz, Maria|Some text:And some more:And yet:This is the end|true:true|bacon:eggs",
    )?;
    Ok(())
}

#[test]
fn regexp_constructor_call_without_new_and_to_string_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const re = RegExp(/ab+c/, 'i');
            const text = re.toString();
            const ok = re.test('xxABBCyy');
            const hit = re.exec('xxABBCyy');
            document.getElementById('result').textContent = text + ':' + ok + ':' + hit[0];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "/ab+c/i:true:ABBC")?;
    Ok(())
}

#[test]
fn string_ends_with_rejects_regexp_argument() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            'foobar'.endsWith(/bar/);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("endsWith should reject RegExp arguments");
    match err {
        Error::ScriptRuntime(msg) => assert!(
            msg.contains("must not be a regular expression"),
            "unexpected message: {msg}"
        ),
        other => panic!("unexpected endsWith error: {other:?}"),
    }
    Ok(())
}

#[test]
fn symbol_constructor_and_typeof_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sym1 = Symbol();
            const sym2 = Symbol('foo');
            const sym3 = Symbol('foo');
            document.getElementById('result').textContent =
              (typeof sym1) + ':' +
              (typeof sym2) + ':' +
              (typeof Symbol.iterator) + ':' +
              (sym2 === sym3) + ':' +
              (sym1.description === undefined) + ':' +
              sym2.description + ':' +
              (Symbol.iterator === Symbol.iterator);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "symbol:symbol:symbol:false:true:foo:true")?;
    Ok(())
}

#[test]
fn symbol_for_and_key_for_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const reg1 = Symbol.for('tokenString');
            const reg2 = Symbol.for('tokenString');
            const local = Symbol('tokenString');
            document.getElementById('result').textContent =
              (reg1 === reg2) + ':' +
              (reg1 === local) + ':' +
              Symbol.keyFor(reg1) + ':' +
              (Symbol.keyFor(local) === undefined) + ':' +
              (Symbol.keyFor(Symbol.for('tokenString')) === 'tokenString') + ':' +
              (Symbol.keyFor(Symbol.iterator) === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:tokenString:true:true:true")?;
    Ok(())
}

#[test]
fn symbol_properties_and_get_own_property_symbols_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = {};
            obj[Symbol('a')] = 'a';
            obj[Symbol.for('b')] = 'b';
            obj['c'] = 'c';
            obj.d = 'd';

            const keys = Object.keys(obj);
            const values = Object.values(obj);
            const entries = Object.entries(obj);
            const symbols = Object.getOwnPropertySymbols(obj);
            const first = obj[symbols[0]];
            const second = obj[symbols[1]];

            document.getElementById('result').textContent =
              keys.join(',') + '|' +
              values.join(',') + '|' +
              entries.length + '|' +
              symbols.length + '|' +
              first + ':' + second + '|' +
              JSON.stringify(obj);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "c,d|c,d|2|2|a:b|{\"c\":\"c\",\"d\":\"d\"}")?;
    Ok(())
}

#[test]
fn symbol_wrapper_objects_can_be_used_as_property_keys() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const sym = Symbol('foo');
            const obj = { [sym]: 1 };
            document.getElementById('result').textContent =
              (typeof sym) + ':' +
              (typeof Object(sym)) + ':' +
              obj[sym] + ':' +
              obj[Object(sym)] + ':' +
              (Object(sym) == sym);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "symbol:object:1:1:true")?;
    Ok(())
}

#[test]
fn symbol_constructor_and_key_for_errors_are_reported() -> Result<()> {
    let err =
        Harness::from_html("<script>new Symbol();</script>").expect_err("new Symbol should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Symbol is not a constructor")),
        other => panic!("unexpected new Symbol error: {other:?}"),
    }

    let err = Harness::from_html("<script>Symbol.keyFor('x');</script>")
        .expect_err("Symbol.keyFor non-symbol should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Symbol.keyFor argument must be a Symbol"))
        }
        other => panic!("unexpected Symbol.keyFor error: {other:?}"),
    }

    Ok(())
}

#[test]
fn symbol_implicit_conversion_errors_are_reported() {
    let err = Harness::from_html("<script>const sym = Symbol('foo'); sym + 'bar';</script>")
        .expect_err("symbol string concatenation should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Cannot convert a Symbol value to a string"))
        }
        other => panic!("unexpected symbol concat error: {other:?}"),
    }

    let err = Harness::from_html("<script>const sym = Symbol('foo'); +sym;</script>")
        .expect_err("unary plus on symbol should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Cannot convert a Symbol value to a number"))
        }
        other => panic!("unexpected unary plus symbol error: {other:?}"),
    }
}

#[test]
fn numeric_literals_support_hex_octal_binary_and_scientific_notation() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const hex = 0x10;
            const oct = 0o10;
            const bin = 0b10;
            const exp = 1e3;
            document.getElementById('result').textContent =
              hex + ':' + oct + ':' + bin + ':' + exp;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "16:8:2:1000")?;
    Ok(())
}

#[test]
fn encode_decode_uri_global_functions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = encodeURI('https://a.example/a b?x=1&y=2#f');
            const b = encodeURIComponent('a b&c=d');
            const c = decodeURI('https://a.example/a%20b?x=1&y=2#f');
            const d = decodeURI('%3Fx%3D1');
            const e = decodeURIComponent('a%20b%26c%3Dd');
            const f = window.encodeURIComponent('x y');
            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "https://a.example/a%20b?x=1&y=2#f|a%20b%26c%3Dd|https://a.example/a b?x=1&y=2#f|%3Fx%3D1|a b&c=d|x%20y",
        )?;
    Ok(())
}

#[test]
fn decode_uri_invalid_sequence_returns_runtime_error_for_decode_uri() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            decodeURIComponent('%E0%A4%A');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("decodeURIComponent should fail for malformed input");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("malformed URI sequence")),
        other => panic!("unexpected decode URI error: {other:?}"),
    }
    Ok(())
}

#[test]
fn escape_and_unescape_global_functions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const kana = unescape('%u3042');
            const escaped = escape('ABC abc +/' + kana);
            const unescaped = unescape(escaped);
            const viaWindow = window.unescape('%u3042%20A');
            const viaWindowEscaped = window.escape('hello world');
            document.getElementById('result').textContent =
              escaped + '|' + unescaped + '|' + viaWindow + '|' + viaWindowEscaped;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "ABC%20abc%20+/%u3042|ABC abc +/あ|あ A|hello%20world",
    )?;
    Ok(())
}

#[test]
fn window_aliases_for_global_functions_match_direct_calls() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              window.encodeURI('a b?x=1') + '|' + encodeURI('a b?x=1') + '|' +
              window.decodeURIComponent('a%20b%2Bc') + '|' + decodeURIComponent('a%20b%2Bc') + '|' +
              window.unescape(window.escape('A B')) + '|' +
              window.atob(window.btoa('ok')) + '|' +
              window.isNaN('x') + '|' +
              window.isFinite('3') + '|' +
              window.parseInt('11', 2) + '|' +
              window.parseFloat('2.5z');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "a%20b?x=1|a%20b?x=1|a b+c|a b+c|A B|ok|true|true|3|2.5",
    )?;
    Ok(())
}

#[test]
fn fetch_uses_registered_mock_response_and_records_calls() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = fetch('/api/message');
            const second = window.fetch('/api/message');
            document.getElementById('result').textContent = first + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_fetch_mock("/api/message", "ok");
    h.click("#btn")?;
    h.assert_text("#result", "ok:ok")?;
    assert_eq!(
        h.take_fetch_calls(),
        vec!["/api/message".to_string(), "/api/message".to_string()]
    );
    Ok(())
}

#[test]
fn fetch_without_mock_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            fetch('/api/missing');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("fetch without mock should fail with runtime error");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("fetch mock not found")),
        other => panic!("unexpected fetch error: {other:?}"),
    }
    Ok(())
}

#[test]
fn match_media_uses_registered_mocks_and_records_calls() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = matchMedia('(min-width: 768px)');
            const b = window.matchMedia('(prefers-color-scheme: dark)');
            const c = matchMedia('(min-width: 768px)').matches;
            const d = window.matchMedia('(prefers-color-scheme: dark)').media;
            document.getElementById('result').textContent =
              a.matches + ':' + a.media + ':' +
              b.matches + ':' + b.media + ':' +
              c + ':' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_match_media_mock("(min-width: 768px)", true);
    h.set_match_media_mock("(prefers-color-scheme: dark)", false);
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "true:(min-width: 768px):false:(prefers-color-scheme: dark):true:(prefers-color-scheme: dark)",
        )?;
    assert_eq!(
        h.take_match_media_calls(),
        vec![
            "(min-width: 768px)".to_string(),
            "(prefers-color-scheme: dark)".to_string(),
            "(min-width: 768px)".to_string(),
            "(prefers-color-scheme: dark)".to_string(),
        ]
    );
    Ok(())
}

#[test]
fn match_media_default_value_can_be_configured() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const first = matchMedia('(unknown-query)').matches;
            const second = window.matchMedia('(unknown-query)').matches;
            document.getElementById('result').textContent = first + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:false")?;

    h.set_default_match_media_matches(true);
    h.click("#btn")?;
    h.assert_text("#result", "true:true")?;
    Ok(())
}

#[test]
fn navigator_clipboard_read_text_then_updates_dom() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p class='clip-text'>initial</p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            navigator.clipboard
              .readText()
              .then((clipText) => {
                document.querySelector('.clip-text').textContent = clipText;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("from-clipboard");
    h.click("#btn")?;
    h.assert_text(".clip-text", "from-clipboard")?;
    Ok(())
}

#[test]
fn navigator_clipboard_read_text_returns_empty_string_when_clipboard_is_empty() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p class='clip-text'>keep</p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            navigator.clipboard.readText().then((clipText) => {
              document.querySelector('.clip-text').textContent = clipText;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(".clip-text", "")?;
    Ok(())
}

#[test]
fn navigator_clipboard_write_text_and_window_alias_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const same = navigator.clipboard === window.navigator.clipboard;
            window.navigator.clipboard
              .writeText('saved')
              .then(() => navigator.clipboard.readText())
              .then((clipText) => {
                document.getElementById('result').textContent = same + ':' + clipText;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:saved")?;
    assert_eq!(h.clipboard_text(), "saved");
    Ok(())
}

#[test]
fn navigator_clipboard_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          navigator.clipboard = null;
        </script>
        "#,
    )
    .expect_err("navigator.clipboard should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "navigator.clipboard is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn structured_clone_deep_copies_objects_arrays_and_dates() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const source = { nested: { value: 1 }, items: [1, 2] };
            const clone = structuredClone(source);
            const sourceNested = source.nested;
            const cloneNested = clone.nested;
            const sourceItems = source.items;
            const cloneItems = clone.items;

            cloneNested.value = 9;
            cloneItems.push(3);

            const date = new Date('2020-01-02T03:04:05Z');
            const dateClone = structuredClone(date);
            dateClone.setTime(0);

            document.getElementById('result').textContent =
              sourceNested.value + ':' + cloneNested.value + ':' +
              sourceItems.length + ':' + cloneItems.length + ':' +
              (source === clone) + ':' + (sourceNested === cloneNested) + ':' +
              (sourceItems === cloneItems) + ':' +
              (date.getTime() != dateClone.getTime()) + ':' + (date === dateClone);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:9:2:3:false:false:false:true:false")?;
    Ok(())
}

#[test]
fn structured_clone_rejects_non_cloneable_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fn = () => {};
            structuredClone(fn);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("structuredClone should reject functions");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("not cloneable")),
        other => panic!("unexpected structuredClone error: {other:?}"),
    }
    Ok(())
}

#[test]
fn request_animation_frame_and_cancel_animation_frame_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const out = document.getElementById('result');
            const canceled = requestAnimationFrame((ts) => {
              out.textContent = out.textContent + 'C' + ts;
            });
            window.cancelAnimationFrame(canceled);
            window.requestAnimationFrame((ts) => {
              out.textContent = out.textContent + 'R' + ts;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(15)?;
    h.assert_text("#result", "")?;
    h.advance_time(1)?;
    h.assert_text("#result", "R16")?;
    Ok(())
}

#[test]
fn function_constructor_uses_global_scope_while_closure_keeps_local_scope() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          var x = 10;

          function createFunction1() {
            const x = 20;
            return new Function("return x;");
          }

          function createFunction2() {
            const x = 20;
            function f() {
              return x;
            }
            return f;
          }

          document.getElementById('btn').addEventListener('click', () => {
            const f1 = createFunction1();
            const f2 = createFunction2();
            document.getElementById('result').textContent = f1() + ':' + f2();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "10:20")?;
    Ok(())
}

#[test]
fn alert_confirm_prompt_support_mocked_responses() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const accepted = confirm('continue?');
            const name = prompt('name?', 'guest');
            window.alert('hello ' + name);
            document.getElementById('result').textContent = accepted + ':' + name;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.enqueue_confirm_response(true);
    h.enqueue_prompt_response(Some("kazu"));
    h.click("#btn")?;
    h.assert_text("#result", "true:kazu")?;
    assert_eq!(h.take_alert_messages(), vec!["hello kazu".to_string()]);
    Ok(())
}

#[test]
fn prompt_uses_default_argument_when_no_mock_response() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const name = prompt('name?', 'guest');
            document.getElementById('result').textContent = name;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "guest")?;
    Ok(())
}

#[test]
fn global_function_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>encodeURI();</script>",
            "encodeURI requires exactly one argument",
        ),
        (
            "<script>window.encodeURIComponent('a', 'b');</script>",
            "encodeURIComponent requires exactly one argument",
        ),
        (
            "<script>decodeURI('a', 'b');</script>",
            "decodeURI requires exactly one argument",
        ),
        (
            "<script>window.decodeURIComponent();</script>",
            "decodeURIComponent requires exactly one argument",
        ),
        (
            "<script>escape();</script>",
            "escape requires exactly one argument",
        ),
        (
            "<script>window.unescape('a', 'b');</script>",
            "unescape requires exactly one argument",
        ),
        (
            "<script>isNaN();</script>",
            "isNaN requires exactly one argument",
        ),
        (
            "<script>window.isFinite();</script>",
            "isFinite requires exactly one argument",
        ),
        (
            "<script>atob('YQ==', 'x');</script>",
            "atob requires exactly one argument",
        ),
        (
            "<script>window.btoa();</script>",
            "btoa requires exactly one argument",
        ),
        (
            "<script>parseFloat('1', 10);</script>",
            "parseFloat requires exactly one argument",
        ),
        (
            "<script>window.parseInt('1', 10, 10);</script>",
            "parseInt requires one or two arguments",
        ),
        (
            "<script>JSON.parse();</script>",
            "JSON.parse requires exactly one argument",
        ),
        (
            "<script>window.JSON.stringify('x', 1);</script>",
            "JSON.stringify requires exactly one argument",
        ),
        (
            "<script>fetch();</script>",
            "fetch requires exactly one argument",
        ),
        (
            "<script>matchMedia();</script>",
            "matchMedia requires exactly one argument",
        ),
        (
            "<script>navigator.clipboard.readText('x');</script>",
            "navigator.clipboard.readText takes no arguments",
        ),
        (
            "<script>window.navigator.clipboard.writeText();</script>",
            "navigator.clipboard.writeText requires exactly one argument",
        ),
        (
            "<script>structuredClone();</script>",
            "structuredClone requires exactly one argument",
        ),
        (
            "<script>alert();</script>",
            "alert requires exactly one argument",
        ),
        (
            "<script>window.confirm('ok', 'ng');</script>",
            "confirm requires exactly one argument",
        ),
        (
            "<script>prompt();</script>",
            "prompt requires one or two arguments",
        ),
        (
            "<script>window.prompt('x', );</script>",
            "prompt default argument cannot be empty",
        ),
        (
            "<script>requestAnimationFrame();</script>",
            "requestAnimationFrame requires exactly one argument",
        ),
        (
            "<script>cancelAnimationFrame();</script>",
            "cancelAnimationFrame requires 1 argument",
        ),
        (
            "<script>Array.isArray();</script>",
            "Array.isArray requires exactly one argument",
        ),
        (
            "<script>Object.keys();</script>",
            "Object.keys requires exactly one argument",
        ),
        (
            "<script>window.Object.values(1, 2);</script>",
            "Object.values requires exactly one argument",
        ),
        (
            "<script>Object.entries();</script>",
            "Object.entries requires exactly one argument",
        ),
        (
            "<script>Object.hasOwn({ a: 1 });</script>",
            "Object.hasOwn requires exactly two arguments",
        ),
        (
            "<script>const obj = {}; obj.hasOwnProperty();</script>",
            "hasOwnProperty requires exactly one argument",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail to parse");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains(expected), "expected '{expected}' in '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn global_function_parser_respects_identifier_boundaries() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const escaped = escape('A B');
            const encodedValue = encodeURIComponent('x y');
            const parseIntValue = 7;
            const parseFloatValue = 1.25;
            const escapedValue = escaped;
            const round = unescape(escapedValue);
            document.getElementById('result').textContent =
              escapedValue + ':' + encodedValue + ':' + round + ':' +
              parseIntValue + ':' + parseFloatValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A%20B:x%20y:A B:7:1.25")?;
    Ok(())
}

#[test]
fn btoa_non_latin1_input_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const nonLatin1 = unescape('%u3042');
            btoa(nonLatin1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("btoa should reject non-Latin1 input");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("non-Latin1")),
        other => panic!("unexpected btoa error: {other:?}"),
    }
    Ok(())
}

#[test]
fn decode_uri_invalid_sequence_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            decodeURI('%E0%A4%A');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("decodeURI should fail for malformed input");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("malformed URI sequence")),
        other => panic!("unexpected decode URI error: {other:?}"),
    }
    Ok(())
}

#[test]
fn is_nan_and_is_finite_global_functions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = isNaN('abc');
            const b = isNaN('  ');
            const c = isNaN(undefined);
            const d = isFinite('1.5');
            const e = isFinite(Infinity);
            const f = window.isFinite(null);
            const g = window.isNaN(NaN);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' + f + ':' + g;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:true:true:false:true:true")?;
    Ok(())
}

#[test]
fn atob_and_btoa_global_functions_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const encoded = btoa('abc123!?');
            const decoded = atob(encoded);
            const viaWindow = window.atob('QQ==');
            document.getElementById('result').textContent =
              encoded + ':' + decoded + ':' + viaWindow;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "YWJjMTIzIT8=:abc123!?:A")?;
    Ok(())
}

#[test]
fn atob_invalid_input_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='atob'>atob</button>
        <script>
          document.getElementById('atob').addEventListener('click', () => {
            atob('@@@');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let atob_err = h
        .click("#atob")
        .expect_err("atob should reject invalid base64");
    match atob_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("invalid base64")),
        other => panic!("unexpected atob error: {other:?}"),
    }

    Ok(())
}

#[test]
fn parse_int_global_function_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = parseInt('42px');
            const b = parseInt('  -0x10');
            const c = parseInt('10', 2);
            const d = parseInt('10', 8);
            const e = parseInt('0x10', 16);
            const bad1 = parseInt('xyz');
            const bad2 = parseInt('10', 1);
            const f = window.parseInt('12', 10);
            document.getElementById('result').textContent =
              a + ':' + b + ':' + c + ':' + d + ':' + e + ':' +
              (bad1 === bad1) + ':' + (bad2 === bad2) + ':' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "42:-16:2:8:16:false:false:12")?;
    Ok(())
}

#[test]
fn parse_float_global_function_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = parseFloat('3.5px');
            const b = parseFloat('  -2.5e2x');
            const invalid = parseFloat('abc');
            const d = window.parseFloat('42');
            document.getElementById('result').textContent =
              a + ':' + b + ':' + (invalid === invalid) + ':' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3.5:-250:false:42")?;
    Ok(())
}

#[test]
fn json_parse_and_stringify_roundtrip_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const source = '{"a":1,"b":[true,null,"x"],"c":{"d":2}}';
            const parsed = JSON.parse(source);
            const out = JSON.stringify(parsed);
            const arr = JSON.parse('[1,2,3]');
            const viaWindow = window.JSON.stringify(window.JSON.parse('{"x":"y"}'));
            document.getElementById('result').textContent = out + '|' + JSON.stringify(arr) + '|' + viaWindow;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "{\"a\":1,\"b\":[true,null,\"x\"],\"c\":{\"d\":2}}|[1,2,3]|{\"x\":\"y\"}",
    )?;
    Ok(())
}

#[test]
fn json_stringify_handles_special_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const parsed = JSON.parse('"\\u3042\\n\\t"');
            const encoded = JSON.stringify(parsed);
            const topUndefined = JSON.stringify(undefined);
            const finite = JSON.stringify(1.5);
            const nan = JSON.stringify(NaN);
            const inf = JSON.stringify(Infinity);
            const arr = JSON.stringify([1, undefined, NaN, Infinity]);
            const obj = JSON.stringify(JSON.parse('{"a":1,"b":null}'));
            document.getElementById('result').textContent =
              encoded + '|' + topUndefined + '|' + finite + '|' + nan + '|' + inf + '|' + arr + '|' + obj;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "\"あ\\n\\t\"|undefined|1.5|null|null|[1,null,null,null]|{\"a\":1,\"b\":null}",
    )?;
    Ok(())
}

#[test]
fn json_parse_invalid_input_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            JSON.parse('{bad json}');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("JSON.parse should fail for invalid input");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("JSON.parse invalid JSON")),
        other => panic!("unexpected JSON.parse error: {other:?}"),
    }
    Ok(())
}

#[test]
fn json_stringify_circular_array_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1];
            arr.push(arr);
            JSON.stringify(arr);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("JSON.stringify should fail for circular array");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("JSON.stringify circular structure")),
        other => panic!("unexpected JSON.stringify error: {other:?}"),
    }
    Ok(())
}

#[test]
fn object_literal_property_access_and_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = { a: 1, "b": 2, a: 3 };
            obj.c = 4;
            obj['d'] = obj.a + obj.b;
            obj.value = 'v';

            const keys = Object.keys(obj);
            const values = Object.values(obj);
            const entries = Object.entries(obj);
            const firstEntry = entries[0];
            const lastEntry = entries[4];
            const ownA = Object.hasOwn(obj, 'a');
            const ownZ = window.Object.hasOwn(obj, 'z');
            const ownD = obj.hasOwnProperty('d');

            document.getElementById('result').textContent =
              obj.a + ':' + obj.b + ':' + obj.c + ':' + obj.d + ':' + obj.value + '|' +
              keys.join(',') + '|' +
              values.join(',') + '|' +
              firstEntry[0] + ':' + firstEntry[1] + ':' + lastEntry[0] + ':' + lastEntry[1] + '|' +
              ownA + ':' + ownZ + ':' + ownD;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "3:2:4:5:v|a,b,c,d,value|3,2,4,5,v|a:3:value:v|true:false:true",
    )?;
    Ok(())
}

#[test]
fn object_property_access_missing_key_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = { ok: 'yes' };
            document.getElementById('result').textContent =
              obj.missing + ':' + (typeof obj.missing) + ':' + obj.ok;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:undefined:yes")?;
    Ok(())
}

#[test]
fn member_call_expression_on_nested_object_path_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const api = {
              a: {
                b: {
                  method: (x, y) => x + y,
                  tag: 'ok'
                }
              }
            };
            const first = api.a.b.method(2, 3);
            const second = api.a.b.method(10, -4);
            document.getElementById('result').textContent =
              api.a.b.tag + ':' + first + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok:5:6")?;
    Ok(())
}

#[test]
fn member_call_expression_reports_non_function_target() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const api = { a: { b: { method: 1 } } };
            api.a.b.method('x');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("member call on non-function should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("'method' is not a function")),
        other => panic!("unexpected member call error: {other:?}"),
    }
    Ok(())
}

#[test]
fn object_method_runtime_type_errors_are_reported() -> Result<()> {
    let html = r#"
        <button id='keys'>keys</button>
        <button id='own'>own</button>
        <script>
          document.getElementById('keys').addEventListener('click', () => {
            Object.keys(1);
          });
          document.getElementById('own').addEventListener('click', () => {
            const x = 1;
            x.hasOwnProperty('a');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let keys_err = h
        .click("#keys")
        .expect_err("Object.keys should reject non-object argument");
    match keys_err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Object.keys argument must be an object"))
        }
        other => panic!("unexpected Object.keys error: {other:?}"),
    }

    let own_err = h
        .click("#own")
        .expect_err("hasOwnProperty should reject non-object receiver");
    match own_err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("is not an object")),
        other => panic!("unexpected hasOwnProperty error: {other:?}"),
    }

    Ok(())
}

#[test]
fn array_literal_and_basic_mutation_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2];
            const isArray1 = Array.isArray(arr);
            const isArray2 = window.Array.isArray('x');
            const lenBefore = arr.length;
            const first = arr[0];
            const pushed = arr.push(3, 4);
            const popped = arr.pop();
            const shifted = arr.shift();
            const unshifted = arr.unshift(9);
            document.getElementById('result').textContent =
              isArray1 + ':' + isArray2 + ':' + lenBefore + ':' + first + ':' +
              pushed + ':' + popped + ':' + shifted + ':' + unshifted + ':' + arr.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:2:1:4:4:1:3:9,2,3")?;
    Ok(())
}

#[test]
fn array_map_filter_and_reduce_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2, 3, 4];
            const mapped = arr.map((value, index) => value * 2 + index);
            const filtered = mapped.filter(value => value > 5);
            const sum = filtered.reduce((acc, value) => acc + value, 0);
            const sumNoInitial = filtered.reduce((acc, value) => acc + value);
            document.getElementById('result').textContent =
              mapped.join(',') + '|' + filtered.join(',') + '|' + sum + '|' + sumNoInitial;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2,5,8,11|8,11|19|19")?;
    Ok(())
}

#[test]
fn array_foreach_find_some_every_and_includes_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [2, 4, 6];
            let total = 0;
            arr.forEach((value, idx) => {
              total += value + idx;
            });
            const found = arr.find(value => value > 3);
            const some = arr.some(value => value === 4);
            const every = arr.every(value => value % 2 === 0);
            const includesDirect = arr.includes(4);
            const includesFrom = arr.includes(2, 1);
            document.getElementById('result').textContent =
              total + ':' + found + ':' + some + ':' + every + ':' +
              includesDirect + ':' + includesFrom;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "15:4:true:true:true:false")?;
    Ok(())
}

#[test]
fn array_slice_splice_and_join_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2, 3, 4];
            const firstSlice = arr.slice(1, 3);
            const secondSlice = arr.slice(-2);
            const removed = arr.splice(1, 2, 9, 8);
            document.getElementById('result').textContent =
              firstSlice.join(',') + '|' + secondSlice.join(',') + '|' +
              removed.join(',') + '|' + arr.join('-');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2,3|3,4|2,3|1-9-8-4")?;
    Ok(())
}

#[test]
fn reduce_empty_array_without_initial_value_returns_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [];
            arr.reduce((acc, value) => acc + value);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("reduce without initial on empty array should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("reduce of empty array with no initial value"))
        }
        other => panic!("unexpected reduce error: {other:?}"),
    }
    Ok(())
}

#[test]
fn string_trim_and_case_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const raw = '  AbC ';
            const trimmed = raw.trim();
            const trimmedStart = raw.trimStart();
            const trimmedEnd = raw.trimEnd();
            const upper = raw.toUpperCase();
            const lower = raw.toLowerCase();
            const literal = ' z '.trim();
            document.getElementById('result').textContent =
              '[' + trimmed + ']|[' + trimmedStart + ']|[' + trimmedEnd + ']|[' +
              upper + ']|[' + lower + ']|[' + literal + ']';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "[AbC]|[AbC ]|[  AbC]|[  ABC ]|[  abc ]|[z]")?;
    Ok(())
}

#[test]
fn string_includes_prefix_suffix_and_index_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = 'hello world';
            const includes1 = text.includes('lo');
            const includes2 = text.includes('lo', 4);
            const includes3 = 'abc'.includes('a', -2);
            const starts1 = text.startsWith('hello');
            const starts2 = text.startsWith('world', 6);
            const starts3 = 'abc'.startsWith('a');
            const ends1 = text.endsWith('world');
            const ends2 = text.endsWith('hello', 5);
            const index1 = text.indexOf('o');
            const index2 = text.indexOf('o', 5);
            const index3 = text.indexOf('x');
            const index4 = text.indexOf('', 2);
            document.getElementById('result').textContent =
              includes1 + ':' + includes2 + ':' + includes3 + ':' +
              starts1 + ':' + starts2 + ':' + starts3 + ':' +
              ends1 + ':' + ends2 + ':' +
              index1 + ':' + index2 + ':' + index3 + ':' + index4;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:false:true:true:true:true:true:true:4:7:-1:2",
    )?;
    Ok(())
}

#[test]
fn string_slice_substring_split_and_replace_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = '012345';
            const s1 = text.slice(1, 4);
            const s2 = text.slice(-2);
            const s3 = text.slice(4, 1);
            const sub1 = text.substring(1, 4);
            const sub2 = text.substring(4, 1);
            const sub3 = text.substring(-2, 2);
            const split1 = 'a,b,c'.split(',');
            const split2 = 'abc'.split('');
            const split3 = 'a,b,c'.split(',', 2);
            const split4 = 'abc'.split();
            const rep1 = 'foo foo'.replace('foo', 'bar');
            const rep2 = 'abc'.replace('', '-');
            document.getElementById('result').textContent =
              s1 + ':' + s2 + ':' + s3.length + ':' +
              sub1 + ':' + sub2 + ':' + sub3 + ':' +
              split1.join('-') + ':' + split2.join('|') + ':' + split3.join(':') + ':' +
              split4.length + ':' + rep1 + ':' + rep2;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "123:45:0:123:123:01:a-b-c:a|b|c:a:b:1:bar foo:-abc",
    )?;
    Ok(())
}

#[test]
fn string_constructor_and_static_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const string1 = "A string primitive";
            const string2 = String(1);
            const string3 = String(true);
            const string4 = new String("A String object");
            const types =
              typeof string1 + ':' + typeof string2 + ':' + typeof string3 + ':' + typeof string4;
            const ctor = string4.constructor === String;
            const value = string4.valueOf();
            const rendered = string4.toString();
            const fromChar = String.fromCharCode(65, 66, 67);
            const fromCode = String.fromCodePoint(0x1F600);
            const raw = String.raw({ raw: ['Hi\\n', '!'] }, 'Bob');
            const symbolText = String(Symbol('token'));
            document.getElementById('result').textContent =
              types + '|' + ctor + '|' + value + '|' + rendered + '|' +
              fromChar + '|' + (fromCode.length > 0) + '|' + raw + '|' + symbolText;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "string:string:string:object|true|A String object|A String object|ABC|true|Hi\\nBob!|Symbol(token)",
    )?;
    Ok(())
}

#[test]
fn string_extended_instance_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const text = 'cat';
            const charAt = text.charAt(1);
            const charCodeAt = text.charCodeAt(1);
            const codePointAt = text.codePointAt(1);
            const at = text.at(-1);
            const concat = text.concat('s', '!');
            const lastIndex1 = 'bananas'.lastIndexOf('an');
            const lastIndex2 = 'bananas'.lastIndexOf('an', 3);
            const searchRegex = 'abc123'.search(/[0-9]+/);
            const searchString = 'abc'.search('d');
            const replaceAll = 'foo foo'.replaceAll('foo', 'bar');
            const replaceAllRegex = 'a1b2c3'.replaceAll(/[0-9]/g, '');
            const repeated = 'ha'.repeat(3);
            const paddedStart = '5'.padStart(3, '0');
            const paddedEnd = '5'.padEnd(3, '0');
            const localeUpper = 'abc'.toLocaleUpperCase();
            const localeLower = 'ABC'.toLocaleLowerCase();
            const wellFormed = 'ok'.isWellFormed();
            const toWellFormed = 'ok'.toWellFormed();
            document.getElementById('result').textContent =
              charAt + ':' + charCodeAt + ':' + codePointAt + ':' + at + ':' +
              concat + ':' + lastIndex1 + ':' + lastIndex2 + ':' +
              searchRegex + ':' + searchString + ':' +
              replaceAll + ':' + replaceAllRegex + ':' +
              repeated + ':' + paddedStart + ':' + paddedEnd + ':' +
              localeUpper + ':' + localeLower + ':' + wellFormed + ':' + toWellFormed;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "a:97:97:t:cats!:3:1:3:-1:bar bar:abc:hahaha:005:500:ABC:abc:true:ok",
    )?;
    Ok(())
}

#[test]
fn string_locale_compare_and_character_access_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const de = 'ä'.localeCompare('z', 'de');
            const sv = 'ä'.localeCompare('z', 'sv');
            const word = 'cat';
            const charAt = word.charAt(1);
            const bracket = word[1];
            const less = 'a' < 'b';
            const eq = 'HELLO'.toLowerCase() === 'hello';
            document.getElementById('result').textContent =
              (de < 0) + ':' + (sv > 0) + ':' + charAt + ':' + bracket + ':' + less + ':' + eq;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:a:a:true:true")?;
    Ok(())
}

#[test]
fn string_method_arity_errors_have_stable_messages() {
    let cases = [
        (
            "<script>'x'.trim(1);</script>",
            "trim does not take arguments",
        ),
        (
            "<script>'x'.toUpperCase(1);</script>",
            "toUpperCase does not take arguments",
        ),
        (
            "<script>'x'.includes();</script>",
            "String.includes requires one or two arguments",
        ),
        (
            "<script>'x'.startsWith();</script>",
            "startsWith requires one or two arguments",
        ),
        (
            "<script>'x'.endsWith();</script>",
            "endsWith requires one or two arguments",
        ),
        (
            "<script>'x'.slice(, 1);</script>",
            "String.slice start cannot be empty",
        ),
        (
            "<script>'x'.substring(, 1);</script>",
            "substring start cannot be empty",
        ),
        (
            "<script>'x'.split(, 1);</script>",
            "split separator cannot be empty expression",
        ),
        (
            "<script>'x'.replace('a');</script>",
            "replace requires exactly two arguments",
        ),
        (
            "<script>'x'.indexOf();</script>",
            "indexOf requires one or two arguments",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail to parse");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains(expected), "expected '{expected}' in '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn typed_array_constructors_and_properties_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const i8 = new Int8Array([257, -129, 1.9]);
            const u8c = new Uint8ClampedArray([300, -1, 1.5, 2.5, 0.5]);
            const bi = new BigInt64Array([1n, -1n]);
            document.getElementById('result').textContent =
              i8.length + ':' +
              i8[0] + ':' +
              i8[1] + ':' +
              i8[2] + ':' +
              u8c.join(',') + ':' +
              Int8Array.BYTES_PER_ELEMENT + ':' +
              i8.BYTES_PER_ELEMENT + ':' +
              typeof Int8Array + ':' +
              typeof TypedArray + ':' +
              bi[1];
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:1:127:1:255,0,2,2,0:1:1:function:undefined:-1")?;
    Ok(())
}

#[test]
fn typed_array_static_from_of_and_constructor_errors_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = Int16Array.of(1, 2, 3);
            const b = Int16Array.from(a);
            document.getElementById('result').textContent = a.join(',') + ':' + b.join(',');
          });
        </script>
        "#;
    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1,2,3:1,2,3")?;

    let err = Harness::from_html("<script>Int8Array(2);</script>")
        .expect_err("calling typed array constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("must be called with new")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>new BigInt64Array(new Int8Array([1]));</script>")
        .expect_err("mixing bigint and number typed arrays should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("cannot mix BigInt and Number typed arrays"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn typed_array_resizable_array_buffer_view_behavior_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const buffer = new ArrayBuffer(8, { maxByteLength: 16 });
          const tracking = new Float32Array(buffer);
          const fixed = new Float32Array(buffer, 0, 2);

          document.getElementById('btn').addEventListener('click', () => {
            let out =
              tracking.byteLength + ':' + tracking.length + ':' +
              fixed.byteLength + ':' + fixed.length;

            buffer.resize(12);
            out = out + ':' +
              tracking.byteLength + ':' + tracking.length + ':' +
              fixed.byteLength + ':' + fixed.length;

            buffer.resize(7);
            out = out + ':' +
              tracking.byteLength + ':' + tracking.length + ':' +
              fixed.byteLength + ':' + fixed.length + ':' + fixed[0];

            buffer.resize(8);
            out = out + ':' + fixed.byteLength + ':' + fixed.length + ':' + fixed[0];
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "8:2:8:2:12:3:8:2:4:1:0:0:undefined:8:2:0")?;
    Ok(())
}

#[test]
fn typed_array_methods_set_subarray_copy_within_and_with_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const ta = new Uint8Array([1, 2, 3, 4]);
            ta.copyWithin(1, 2);
            const sub = ta.subarray(1, 3);
            ta.set([9, 8], 2);
            const withOne = ta.with(0, 7);
            const rev = ta.toReversed();
            const src = new Uint8Array([3, 1, 2]);
            const sorted = src.toSorted();
            document.getElementById('result').textContent =
              ta.join(',') + ':' +
              sub.join(',') + ':' +
              withOne.join(',') + ':' +
              rev.join(',') + ':' +
              sorted.join(',') + ':' +
              ta.at(-1);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1,3,9,8:3,9:7,3,9,8:8,9,3,1:1,2,3:8")?;
    Ok(())
}

#[test]
fn typed_array_abstract_constructor_and_freeze_errors_work() {
    let err = Harness::from_html("<script>new (Object.getPrototypeOf(Int8Array))();</script>")
        .expect_err("abstract TypedArray constructor should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Abstract class TypedArray not directly constructable"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err =
        Harness::from_html("<script>const i8 = Int8Array.of(1,2,3); Object.freeze(i8);</script>")
            .expect_err("freezing non-empty typed array should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Cannot freeze array buffer views with elements"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn typed_array_alignment_and_array_buffer_constructor_errors_work() {
    let cases = [
        (
            "<script>new Int32Array(new ArrayBuffer(3));</script>",
            "byte length of Int32Array should be a multiple of 4",
        ),
        (
            "<script>new Int32Array(new ArrayBuffer(4), 1);</script>",
            "start offset of Int32Array should be a multiple of 4",
        ),
        (
            "<script>ArrayBuffer(8);</script>",
            "ArrayBuffer constructor must be called with new",
        ),
    ];

    for (html, expected) in cases {
        let err = Harness::from_html(html).expect_err("script should fail");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains(expected), "expected '{expected}', got '{msg}'")
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn map_set_get_size_delete_and_iteration_order_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const map = new Map();
            map.set('a', 1);
            map.set('b', 2);
            map.set('c', 3);
            map.set('a', 97);

            const deleted = map.delete('b');
            const missing = map.delete('missing');

            let forEachOut = '';
            map.forEach((value, key, self) => {
              forEachOut = forEachOut + key + '=' + value + ':' + (self === map) + ';';
            });

            let forOfOut = '';
            for (const pair of map) {
              forOfOut = forOfOut + pair[0] + '=' + pair[1] + ';';
            }

            const entries = map.entries();
            const keys = map.keys();
            const values = map.values();

            document.getElementById('result').textContent =
              map.get('a') + ':' +
              map.size + ':' +
              deleted + ':' +
              missing + ':' +
              forEachOut + ':' +
              forOfOut + ':' +
              entries.length + ':' +
              keys.join(',') + ':' +
              values.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "97:2:true:false:a=97:true;c=3:true;:a=97;c=3;:2:a,c:97,3",
    )?;
    Ok(())
}

#[test]
fn map_same_value_zero_and_wrong_property_assignment_behavior_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const map = new Map();
            const keyArr = [];

            map.set(NaN, 'not-a-number');
            map.set(0, 'zero');
            map.set(-0, 'minus-zero');
            map.set(keyArr, 'arr');

            const wrongMap = new Map();
            wrongMap['bla'] = 'blaa';
            wrongMap['bla2'] = 'blaaa2';

            document.getElementById('result').textContent =
              map.get(Number('foo')) + ':' +
              map.get(0) + ':' +
              map.has(-0) + ':' +
              map.get([]) + ':' +
              map.get(keyArr) + ':' +
              wrongMap.has('bla') + ':' +
              wrongMap.size + ':' +
              wrongMap.bla;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "not-a-number:minus-zero:true:undefined:arr:false:0:blaa",
    )?;
    Ok(())
}

#[test]
fn map_group_by_and_get_or_insert_methods_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const grouped = Map.groupBy([1, 2, 3, 4, 5], function(value) { return value % 2; });
            const odd = grouped.get(1);
            const even = grouped.get(0);
            const map = new Map();
            const first = map.getOrInsert('count', 1);
            const second = map.getOrInsert('count', 9);
            const computed1 = map.getOrInsertComputed('lazy', function(key) { return key + '-value'; });
            const computed2 = map.getOrInsertComputed('lazy', function() { return 'ignored'; });

            document.getElementById('result').textContent =
              odd.join(',') + ':' +
              even.join(',') + ':' +
              first + ':' +
              second + ':' +
              computed1 + ':' +
              computed2 + ':' +
              map.size;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1,3,5:2,4:1:1:lazy-value:lazy-value:2")?;
    Ok(())
}

#[test]
fn map_constructor_clone_and_error_cases_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const original = new Map([[1, 'one'], [2, 'two']]);
            const clone = new Map(original);
            clone.set(1, 'uno');
            const cleared = new Map([[1, 'x']]);
            cleared.clear();
            document.getElementById('result').textContent =
              original.get(1) + ':' + clone.get(1) + ':' + (original === clone) + ':' +
              clone.size + ':' + cleared.size;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "one:uno:false:2:0")?;

    let err = Harness::from_html("<script>Map();</script>")
        .expect_err("calling Map constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Map constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const value = []; value.delete('x');</script>")
        .expect_err("Map methods on non-map value should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("is not a Map")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn set_basic_methods_and_iteration_order_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const set = new Set();
            set.add(1);
            set.add(2);
            set.add(2);
            set.add(NaN);
            set.add(Number('foo'));
            set.delete(2);
            set.add(2);

            let order = '';
            for (const item of set) {
              order = order + item + ',';
            }

            const keys = set.keys();
            const values = set.values();
            const entries = set.entries();

            let forEachOut = '';
            set.forEach((value, key, self) => {
              forEachOut = forEachOut + value + '=' + key + ':' + (self === set) + ';';
            });

            document.getElementById('result').textContent =
              set.size + ':' +
              set.has(NaN) + ':' +
              set.has(2) + ':' +
              order + ':' +
              keys.join('|') + ':' +
              values.join('|') + ':' +
              entries.length + ':' +
              forEachOut;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "3:true:true:1,NaN,2,:1|NaN|2:1|NaN|2:3:1=1:true;NaN=NaN:true;2=2:true;",
    )?;
    Ok(())
}

#[test]
fn set_composition_methods_and_map_set_like_argument_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const a = new Set([1, 2, 3, 4]);
            const b = new Set([3, 4, 5]);
            const m = new Map([[2, 'two'], [7, 'seven']]);

            const union = a.union(b);
            const intersection = a.intersection(b);
            const difference = a.difference(b);
            const symmetric = a.symmetricDifference(b);
            const unionMap = a.union(m);

            const disjoint = a.isDisjointFrom(new Set([8, 9]));
            const subsetSet = new Set([1, 2]);
            const supersetSet = new Set([1, 4]);
            const subsetMapSet = new Set([2]);
            const subset = subsetSet.isSubsetOf(a);
            const superset = a.isSupersetOf(supersetSet);
            const subsetMap = subsetMapSet.isSubsetOf(m);

            const unionValues = union.values();
            const intersectionValues = intersection.values();
            const differenceValues = difference.values();
            const symmetricValues = symmetric.values();
            const unionMapValues = unionMap.values();

            document.getElementById('result').textContent =
              unionValues.join(',') + ':' +
              intersectionValues.join(',') + ':' +
              differenceValues.join(',') + ':' +
              symmetricValues.join(',') + ':' +
              unionMapValues.join(',') + ':' +
              disjoint + ':' + subset + ':' + superset + ':' + subsetMap;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "1,2,3,4,5:3,4:1,2:1,2,5:1,2,3,4,7:true:true:true:true",
    )?;
    Ok(())
}

#[test]
fn set_constructor_iterable_and_property_assignment_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const map = new Map([[1, 'one'], [2, 'two']]);
            const fromArr = new Set([1, 1, 2, 3]);
            const fromMap = new Set(map);
            const fromMapValues = fromMap.values();

            const wrongSet = new Set();
            wrongSet['bla'] = 'x';

            const obj = {};
            fromArr.add(obj);
            fromArr.add({});

            document.getElementById('result').textContent =
              fromArr.size + ':' +
              fromArr.has(1) + ':' +
              fromArr.has(4) + ':' +
              fromMap.size + ':' +
              fromMapValues.join('|') + ':' +
              wrongSet.has('bla') + ':' +
              wrongSet.size + ':' +
              wrongSet.bla;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5:true:false:2:1,one|2,two:false:0:x")?;
    Ok(())
}

#[test]
fn set_constructor_and_composition_errors_work() {
    let err = Harness::from_html("<script>Set();</script>")
        .expect_err("calling Set constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Set constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const set = new Set([1]); set.union([1,2]);</script>")
        .expect_err("Set.union requires a set-like argument");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("set-like")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const arr = []; arr.union(new Set([1]));</script>")
        .expect_err("Set method target must be a Set");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("is not a Set")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn url_search_params_basic_methods_and_iteration_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const params = new URLSearchParams("q=URLUtils.searchParams&topic=api");

            let forOfOut = '';
            for (const pair of params) {
              forOfOut = forOfOut + pair[0] + '=' + pair[1] + ';';
            }

            const entries = params.entries();
            let entriesOut = '';
            for (const pair of entries) {
              entriesOut = entriesOut + pair[0] + '=' + pair[1] + ';';
            }

            const hasTopic = params.has('topic');
            const hasTopicFish = params.has('topic', 'fish');
            const topic = params.get('topic');
            const allTopic = params.getAll('topic');
            const missingIsNull = params.get('foo') === null;

            params.append('topic', 'webdev');
            const afterAppend = params.toString();

            params.set('topic', 'More webdev');
            const afterSet = params.toString();

            params.append('topic', 'fish');
            const hasFish = params.has('topic', 'fish');
            params.delete('topic', 'fish');
            const afterDeletePair = params.toString();

            params.delete('topic');
            const afterDelete = params.toString();

            document.getElementById('result').textContent =
              forOfOut + '|' +
              entriesOut + '|' +
              hasTopic + ':' + hasTopicFish + ':' + topic + ':' + allTopic.join(',') + ':' + missingIsNull + '|' +
              afterAppend + '|' +
              afterSet + '|' +
              hasFish + '|' +
              afterDeletePair + '|' +
              afterDelete + '|' +
              params.size;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "q=URLUtils.searchParams;topic=api;|q=URLUtils.searchParams;topic=api;|true:false:api:api:true|q=URLUtils.searchParams&topic=api&topic=webdev|q=URLUtils.searchParams&topic=More+webdev|true|q=URLUtils.searchParams&topic=More+webdev|q=URLUtils.searchParams|1",
    )?;
    Ok(())
}

#[test]
fn url_search_params_object_and_location_parsing_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const paramsObj = { foo: 'bar', baz: 'bar' };
            const fromObj = new URLSearchParams(paramsObj);

            location.href = 'https://developer.mozilla.org/en-US/docs/Web/API/URLSearchParams?foo=a';
            const fromLocation = new URLSearchParams(window.location.search);

            const full = new URLSearchParams('http://example.com/search?query=%40');
            const leading = new URLSearchParams('?query=value');
            const dup = new URLSearchParams('foo=bar&foo=baz');
            const dupAll = dup.getAll('foo');
            const emptyVal = new URLSearchParams('foo=&bar=baz');
            const noEquals = new URLSearchParams('foo&bar=baz');

            document.getElementById('result').textContent =
              fromObj.toString() + ':' +
              fromObj.has('foo') + ':' +
              fromObj.get('foo') + ':' +
              fromLocation.get('foo') + ':' +
              full.has('query') + ':' +
              full.has('http://example.com/search?query') + ':' +
              full.get('query') + ':' +
              full.get('http://example.com/search?query') + ':' +
              leading.has('query') + ':' +
              dup.get('foo') + ':' +
              dupAll.join(',') + ':' +
              emptyVal.get('foo') + ':' +
              noEquals.get('foo') + ':' +
              noEquals.toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "foo=bar&baz=bar:true:bar:a:false:true:null:@:true:bar:bar,baz:::foo=&bar=baz",
    )?;
    Ok(())
}

#[test]
fn url_search_params_percent_encoding_and_plus_behavior_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const params = new URLSearchParams('%24%25%26=%28%29%2B');
            const decoded = params.get('$%&');
            const encodedKeyMiss = params.get('%24%25%26') === null;
            params.append('$%&$#@+', '$#&*@#()+');
            const encoded = params.toString();

            const plusBrokenParams = new URLSearchParams('bin=E+AXQB+A');
            const plusBroken = plusBrokenParams.get('bin');
            const plusPreservedParams = new URLSearchParams();
            plusPreservedParams.append('bin', 'E+AXQB+A');
            const plusPreserved = plusPreservedParams.get('bin');
            const plusSerialized = plusPreservedParams.toString();

            const encodedKeyParams = new URLSearchParams();
            encodedKeyParams.append('%24%26', 'value');

            document.getElementById('result').textContent =
              decoded + ':' +
              encodedKeyMiss + ':' +
              encoded + ':' +
              plusBroken + ':' +
              plusPreserved + ':' +
              plusSerialized + ':' +
              encodedKeyParams.toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "()+:true:%24%25%26=%28%29%2B&%24%25%26%24%23%40%2B=%24%23%26*%40%23%28%29%2B:E AXQB A:E+AXQB+A:bin=E%2BAXQB%2BA:%2524%2526=value",
    )?;
    Ok(())
}

#[test]
fn url_search_params_constructor_requires_new() {
    let err = Harness::from_html("<script>URLSearchParams('a=1');</script>")
        .expect_err("calling URLSearchParams constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("URLSearchParams constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn url_constructor_properties_setters_and_methods_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://some.site/?id=123';

            const url = new URL('../cats', 'http://www.example.com/dogs');
            const initial =
              url.href + '|' +
              url.hostname + '|' +
              url.pathname + '|' +
              url.origin;

            url['hash'] = 'tabby';
            url['username'] = 'alice';
            url['password'] = 'secret';
            url['port'] = '8080';
            url['protocol'] = 'https:';
            url['pathname'] = 'démonstration.html';
            url['search'] = 'q=space value';

            const parsedLocation = new URL(window.location.href);
            const fromLocation = parsedLocation.searchParams.get('id');

            document.getElementById('result').textContent =
              initial + '|' +
              url.href + '|' +
              url.hash + '|' +
              url.search + '|' +
              url.searchParams.get('q') + '|' +
              url.toString() + '|' +
              url.toJSON() + '|' +
              (window.URL === URL) + '|' +
              (typeof URL) + '|' +
              fromLocation;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "http://www.example.com/cats|www.example.com|/cats|http://www.example.com|https://alice:secret@www.example.com:8080/d%C3%A9monstration.html?q=space%20value#tabby|#tabby|?q=space%20value|space value|https://alice:secret@www.example.com:8080/d%C3%A9monstration.html?q=space%20value#tabby|https://alice:secret@www.example.com:8080/d%C3%A9monstration.html?q=space%20value#tabby|true|function|123",
    )?;
    Ok(())
}

#[test]
fn url_search_params_live_sync_with_url_search_and_href_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const url = new URL('https://example.com/?a=b ~');
            const before = url.href;

            const params = url.searchParams;
            const appended = params.append('topic', 'web dev');
            const afterAppend = url.href;

            url['search'] = '?x=1';
            const afterSearch = url.href;

            const topicCleared = url.searchParams.get('topic') === null;
            const xValue = url.searchParams.get('x');

            document.getElementById('result').textContent =
              before + '|' +
              afterAppend + '|' +
              afterSearch + '|' +
              topicCleared + ':' + xValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/?a=b%20~|https://example.com/?a=b+%7E&topic=web+dev|https://example.com/?x=1|true:1",
    )?;
    Ok(())
}

#[test]
fn url_static_methods_and_blob_object_urls_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const blob = new Blob(['abc'], { type: 'text/plain' });

            const objectUrl1 = URL.createObjectURL(blob);
            URL.revokeObjectURL(objectUrl1);
            const objectUrl2 = window.URL.createObjectURL(blob);
            URL.revokeObjectURL('blob:bt-999');

            const canRel = URL.canParse('../cats', 'http://www.example.com/dogs');
            const canBad = URL.canParse('/cats');

            const parsed = URL.parse('../cats', 'http://www.example.com/dogs');
            const parsedHref = parsed === null ? 'null' : parsed.href;
            const parsedBad = URL.parse('/cats') === null;

            const C = URL;
            const viaAlias = C.canParse('https://example.com/path');

            document.getElementById('result').textContent =
              objectUrl1 + '|' +
              objectUrl2 + '|' +
              canRel + ':' + canBad + '|' +
              parsedHref + ':' + parsedBad + ':' + viaAlias;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "blob:bt-1|blob:bt-2|true:false|http://www.example.com/cats:true:true",
    )?;
    Ok(())
}

#[test]
fn url_constructor_requires_new() {
    let err = Harness::from_html("<script>URL('https://example.com');</script>")
        .expect_err("calling URL constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("URL constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn url_constructor_without_base_rejects_relative_urls() {
    let err = Harness::from_html("<script>new URL('/cats');</script>")
        .expect_err("relative URL without base should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Invalid URL")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn url_href_setter_rejects_relative_urls() {
    let err = Harness::from_html(
        "<script>const u = new URL('https://example.com/a'); u['href'] = '/cats';</script>",
    )
    .expect_err("setting URL.href to a relative URL should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Invalid URL")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn blob_constructor_properties_and_text_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = { hello: 'world' };
            const blob = new Blob([JSON.stringify(obj)], { type: 'Application/JSON' });
            const promise = blob.text();
            promise.then((text) => {
              document.getElementById('result').textContent =
                blob.size + ':' +
                blob.type + ':' +
                text + ':' +
                (blob.constructor === Blob);
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "17:application/json:{\"hello\":\"world\"}:true")?;
    Ok(())
}

#[test]
fn blob_array_buffer_bytes_slice_and_stream_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const source = new Uint8Array([65, 66, 67, 68]);
            const blob = new Blob([source.buffer], { type: 'text/plain' });
            const sliced = blob.slice(1, 3);
            const p1 = blob.arrayBuffer();
            const p2 = blob.bytes();
            const p3 = sliced.text();

            Promise.all([p1, p2, p3]).then((values) => {
              const fromAb = new Uint8Array(values[0]).join(',');
              const fromBytes = values[1].join(',');
              const streamObj = blob.stream();
              document.getElementById('result').textContent =
                fromAb + '|' +
                fromBytes + '|' +
                values[2] + '|' +
                (typeof streamObj) + ':' +
                (streamObj ? 'y' : 'n');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "65,66,67,68|65,66,67,68|BC|object:y")?;
    Ok(())
}

#[test]
fn blob_constructor_and_method_errors_work() {
    let err = Harness::from_html("<script>Blob(['x']);</script>")
        .expect_err("calling Blob constructor without new should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Blob constructor must be called with new"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const b = new Blob(['x']); b.text('oops');</script>")
        .expect_err("Blob.text should reject extra arguments");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Blob.text does not take arguments")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn array_buffer_properties_slice_and_is_view_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const buffer = new ArrayBuffer(8, { maxByteLength: 16 });
          const view = new Uint8Array(buffer);
          view.set([10, 20, 30, 40]);

          document.getElementById('btn').addEventListener('click', () => {
            const sliced = buffer.slice(1, 3);
            const slicedView = new Uint8Array(sliced);
            document.getElementById('result').textContent =
              buffer.byteLength + ':' +
              buffer.resizable + ':' +
              buffer.maxByteLength + ':' +
              buffer.detached + ':' +
              ArrayBuffer.isView(view) + ':' +
              ArrayBuffer.isView(buffer) + ':' +
              sliced.byteLength + ':' +
              sliced.resizable + ':' +
              sliced.maxByteLength + ':' +
              (buffer.constructor === ArrayBuffer) + ':' +
              slicedView.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "8:true:16:false:true:false:2:false:2:true:20,30")?;
    Ok(())
}

#[test]
fn array_buffer_transfer_and_transfer_to_fixed_length_detach_source() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const buffer = new ArrayBuffer(6, { maxByteLength: 12 });
            const view = new Uint8Array(buffer);
            view.set([1, 2, 3, 4, 5, 6]);

            const moved = buffer.transfer();
            const fixed = moved.transferToFixedLength();
            const fixedView = new Uint8Array(fixed);

            document.getElementById('result').textContent =
              buffer.detached + ':' +
              buffer.byteLength + ':' +
              view.byteLength + ':' +
              moved.detached + ':' +
              moved.byteLength + ':' +
              moved.resizable + ':' +
              moved.maxByteLength + ':' +
              fixed.detached + ':' +
              fixed.byteLength + ':' +
              fixed.resizable + ':' +
              fixed.maxByteLength + ':' +
              fixedView.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "true:0:0:true:0:false:0:false:6:false:6:1,2,3,4,5,6",
    )?;
    Ok(())
}

#[test]
fn array_buffer_detached_behavior_errors_work() {
    let err = Harness::from_html(
            "<script>const b = new ArrayBuffer(4, { maxByteLength: 8 }); b.transfer(); b.resize(2);</script>",
        )
        .expect_err("resize on detached ArrayBuffer should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("detached ArrayBuffer")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html(
        "<script>const b = new ArrayBuffer(4); b.transfer(); b.slice(0, 1);</script>",
    )
    .expect_err("slice on detached ArrayBuffer should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("detached ArrayBuffer")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html(
            "<script>const b = new ArrayBuffer(4); const ta = new Uint8Array(b); b.transfer(); ta.fill(1);</script>",
        )
        .expect_err("typed array methods on detached backing buffer should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("detached ArrayBuffer")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn array_buffer_is_view_arity_and_transfer_arity_errors_work() {
    let err = Harness::from_html("<script>ArrayBuffer.isView();</script>")
        .expect_err("ArrayBuffer.isView without args should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("ArrayBuffer.isView requires exactly one argument"))
        }
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>const b = new ArrayBuffer(4); b.transfer(1);</script>")
        .expect_err("ArrayBuffer.transfer with args should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("ArrayBuffer.transfer does not take arguments"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
