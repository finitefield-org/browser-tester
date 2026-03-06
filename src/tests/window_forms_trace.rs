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
fn mock_window_open_page_uses_url_for_initial_location() -> Result<()> {
    let mut win = MockWindow::new();
    win.open_page(
        "https://app.local/a",
        r#"
            <p id='result'></p>
            <script>
              document.getElementById('result').textContent =
                location.href + '|' + history.length;
            </script>
        "#,
    )?;

    win.assert_text("#result", "https://app.local/a|1")?;
    Ok(())
}

#[test]
fn mock_window_treats_page_urls_as_case_sensitive() -> Result<()> {
    let mut win = MockWindow::new();
    win.open_page("https://app.local/Path", "<p id='result'>upper</p>")?;
    win.open_page("https://app.local/path", "<p id='result'>lower</p>")?;

    assert_eq!(win.page_count(), 2);
    win.switch_to("https://app.local/Path")?;
    win.assert_text("#result", "upper")?;
    win.switch_to("https://app.local/path")?;
    win.assert_text("#result", "lower")?;
    Ok(())
}

#[test]
fn mock_window_current_url_tracks_location_navigation() -> Result<()> {
    let mut win = MockWindow::new();
    win.open_page(
        "https://app.local/start",
        r#"
            <button id='go'>go</button>
            <script>
              document.getElementById('go').addEventListener('click', () => {
                location.assign('/next');
              });
            </script>
        "#,
    )?;

    win.click("#go")?;
    assert_eq!(win.current_url()?, "https://app.local/next");
    Ok(())
}

#[test]
fn mock_window_switch_to_accepts_navigated_url() -> Result<()> {
    let mut win = MockWindow::new();
    win.open_page(
        "https://app.local/a",
        r#"
            <button id='go'>go</button>
            <script>
              document.getElementById('go').addEventListener('click', () => {
                location.assign('/a2');
              });
            </script>
        "#,
    )?;
    win.open_page("https://app.local/b", "<p id='result'>B</p>")?;

    win.switch_to("https://app.local/a")?;
    win.click("#go")?;
    win.switch_to("https://app.local/b")?;
    win.switch_to("https://app.local/a2")?;
    win.assert_exists("#go")?;
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
fn window_close_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.close = 1;
        </script>
        "#,
    )
    .expect_err("window.close should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.close is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_stop_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.stop = 1;
        </script>
        "#,
    )
    .expect_err("window.stop should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.stop is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_focus_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.focus = 1;
        </script>
        "#,
    )
    .expect_err("window.focus should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.focus is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_scroll_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.scroll = 1;
        </script>
        "#,
    )
    .expect_err("window.scroll should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.scroll is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_scroll_by_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.scrollBy = 1;
        </script>
        "#,
    )
    .expect_err("window.scrollBy should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.scrollBy is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_scroll_to_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.scrollTo = 1;
        </script>
        "#,
    )
    .expect_err("window.scrollTo should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.scrollTo is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_print_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.print = 1;
        </script>
        "#,
    )
    .expect_err("window.print should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.print is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_prompt_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.prompt = 1;
        </script>
        "#,
    )
    .expect_err("window.prompt should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.prompt is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_queue_microtask_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.queueMicrotask = 1;
        </script>
        "#,
    )
    .expect_err("window.queueMicrotask should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.queueMicrotask is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_report_error_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.reportError = 1;
        </script>
        "#,
    )
    .expect_err("window.reportError should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.reportError is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_set_interval_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.setInterval = 1;
        </script>
        "#,
    )
    .expect_err("window.setInterval should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.setInterval is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_set_timeout_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.setTimeout = 1;
        </script>
        "#,
    )
    .expect_err("window.setTimeout should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.setTimeout is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_move_by_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.moveBy = 1;
        </script>
        "#,
    )
    .expect_err("window.moveBy should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.moveBy is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_move_to_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.moveTo = 1;
        </script>
        "#,
    )
    .expect_err("window.moveTo should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.moveTo is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_resize_by_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.resizeBy = 1;
        </script>
        "#,
    )
    .expect_err("window.resizeBy should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.resizeBy is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_resize_to_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.resizeTo = 1;
        </script>
        "#,
    )
    .expect_err("window.resizeTo should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.resizeTo is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_post_message_property_is_read_only() {
    let err = Harness::from_html(
        r#"
        <script>
          window.postMessage = 1;
        </script>
        "#,
    )
    .expect_err("window.postMessage should be read-only");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "window.postMessage is read-only"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_closed_reflects_close_calls() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = window.closed;
            window.close();
            const afterFirst = window.closed;
            window.close();
            const afterSecond = window.closed;
            document.getElementById('result').textContent =
              String(before) + ':' + String(afterFirst) + ':' + String(afterSecond);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:true")?;
    Ok(())
}

#[test]
fn window_close_global_alias_and_method_reference_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = window.closed;
            const closeRef = window.close;
            const ret1 = closeRef();
            const ret2 = close();
            const after = window.closed;
            document.getElementById('result').textContent =
              String(before) + ':' + String(after) + ':' +
              String(ret1 === undefined) + ':' + String(ret2 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:true:true")?;
    Ok(())
}

#[test]
fn window_stop_global_alias_and_method_reference_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = window.closed;
            const stopRef = window.stop;
            const ret1 = stopRef();
            const ret2 = stop();
            const ret3 = window.stop();
            const after = window.closed;
            document.getElementById('result').textContent =
              String(before) + ':' + String(after) + ':' +
              String(ret1 === undefined) + ':' + String(ret2 === undefined) + ':' + String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:false:true:true:true")?;
    Ok(())
}

#[test]
fn window_report_error_dispatches_global_error_event() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const err = { message: 'boom' };
            const logs = [];
            window.addEventListener('error', (event) => {
              event.preventDefault();
              logs.push(event.type);
              logs.push(event.error === err);
              logs.push(String(event.error.message || ''));
            });
            const ref = window.reportError;
            const ret = ref(err);
            document.getElementById('result').textContent =
              String(ret === undefined) + '|' + logs.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true|error|true|boom")?;
    Ok(())
}

#[test]
fn window_report_error_requires_argument() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            reportError();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("TypeError: reportError requires one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn window_report_error_supports_only_one_argument() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            reportError('a', 'b');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("reportError supports only one argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn window_focus_global_alias_and_method_reference_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const focusRef = window.focus;
            const ret1 = window.focus();
            const ret2 = focus();
            const ret3 = focusRef('extra', 1);
            document.getElementById('result').textContent =
              String(window.closed) + ':' +
              String(ret1 === undefined) + ':' +
              String(ret2 === undefined) + ':' +
              String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:true:true")?;
    Ok(())
}

#[test]
fn window_scroll_updates_document_position_and_supports_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <div id='spacer' style='height: 2000px; width: 2000px;'>
          <div id='target' style='margin-top: 300px;'>x</div>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const before = target.getBoundingClientRect().top;

            const scrollRef = window.scroll;
            const ret1 = window.scroll(0, 100);
            const afterFirst = target.getBoundingClientRect().top;
            const ret2 = scroll({ top: 120, left: 30, behavior: 'smooth' });
            const afterSecond = target.getBoundingClientRect().top;
            const ret3 = scrollRef({ top: 130, left: 15, behavior: 'instant' });
            const afterThird = target.getBoundingClientRect().top;

            document.getElementById('result').textContent =
              String(before - afterFirst) + ':' +
              String(afterFirst - afterSecond) + ':' +
              String(afterSecond - afterThird) + '|' +
              String(ret1 === undefined) + ':' +
              String(ret2 === undefined) + ':' +
              String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "100:20:10|true:true:true")?;
    Ok(())
}

#[test]
fn window_scroll_by_updates_document_position_and_supports_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <div id='spacer' style='height: 2000px; width: 2000px;'>
          <div id='target' style='margin-top: 300px;'>x</div>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const before = target.getBoundingClientRect().top;

            const scrollByRef = window.scrollBy;
            const ret1 = window.scrollBy(0, 100);
            const afterFirst = target.getBoundingClientRect().top;
            const ret2 = scrollBy({ top: 20, left: 30, behavior: 'smooth' });
            const afterSecond = target.getBoundingClientRect().top;
            const ret3 = scrollByRef({ top: -10, left: 15, behavior: 'instant' });
            const afterThird = target.getBoundingClientRect().top;

            document.getElementById('result').textContent =
              String(before - afterFirst) + ':' +
              String(afterFirst - afterSecond) + ':' +
              String(afterSecond - afterThird) + '|' +
              String(ret1 === undefined) + ':' +
              String(ret2 === undefined) + ':' +
              String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "100:20:-10|true:true:true")?;
    Ok(())
}

#[test]
fn window_scroll_to_updates_document_position_and_supports_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <div id='spacer' style='height: 2000px; width: 2000px;'>
          <div id='target' style='margin-top: 300px;'>x</div>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const target = document.getElementById('target');
            const before = target.getBoundingClientRect().top;

            const scrollToRef = window.scrollTo;
            const ret1 = window.scrollTo(0, 100);
            const afterFirst = target.getBoundingClientRect().top;
            const ret2 = scrollTo({ top: 120, left: 30, behavior: 'smooth' });
            const afterSecond = target.getBoundingClientRect().top;
            const ret3 = scrollToRef({ top: 130, left: 15, behavior: 'instant' });
            const afterThird = target.getBoundingClientRect().top;

            document.getElementById('result').textContent =
              String(before - afterFirst) + ':' +
              String(afterFirst - afterSecond) + ':' +
              String(afterSecond - afterThird) + '|' +
              String(ret1 === undefined) + ':' +
              String(ret2 === undefined) + ':' +
              String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "100:20:10|true:true:true")?;
    Ok(())
}

#[test]
fn window_print_global_alias_and_method_reference_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const printRef = window.print;
            const ret1 = window.print();
            const ret2 = print();
            const ret3 = printRef('extra', 1);
            document.getElementById('result').textContent =
              String(ret1 === undefined) + ':' +
              String(ret2 === undefined) + ':' +
              String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true")?;
    assert_eq!(h.take_print_call_count(), 3);
    Ok(())
}

#[test]
fn window_move_by_updates_screen_coordinates_and_supports_alias() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = [window.screenX, window.screenY, window.screenLeft, window.screenTop].join(':');
            const moveByRef = window.moveBy;
            const ret1 = window.moveBy(10, -5);
            const mid = [window.screenX, window.screenY, window.screenLeft, window.screenTop].join(':');
            const ret2 = moveByRef(3, 4);
            const ret3 = moveBy(-8, 1);
            const after = [window.screenX, window.screenY, window.screenLeft, window.screenTop].join(':');
            document.getElementById('result').textContent =
              before + '|' + mid + '|' + after + '|' +
              String(ret1 === undefined) + ':' + String(ret2 === undefined) + ':' + String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "0:0:0:0|10:-5:10:-5|5:0:5:0|true:true:true")?;
    Ok(())
}

#[test]
fn window_move_to_sets_absolute_coordinates_and_supports_alias() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const before = [window.screenX, window.screenY, window.screenLeft, window.screenTop].join(':');
            const moveToRef = window.moveTo;
            const ret1 = window.moveBy(10, 10);
            const ret2 = window.moveTo(30, 40);
            const mid = [window.screenX, window.screenY, window.screenLeft, window.screenTop].join(':');
            const ret3 = moveToRef(-5, 10);
            const ret4 = moveTo(0, 0);
            const after = [window.screenX, window.screenY, window.screenLeft, window.screenTop].join(':');
            document.getElementById('result').textContent =
              before + '|' + mid + '|' + after + '|' +
              String(ret1 === undefined) + ':' + String(ret2 === undefined) + ':' +
              String(ret3 === undefined) + ':' + String(ret4 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "0:0:0:0|30:40:30:40|0:0:0:0|true:true:true:true")?;
    Ok(())
}

#[test]
fn window_resize_by_updates_dimensions_and_supports_alias() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            window.innerWidth = 1000;
            window.innerHeight = 800;
            const before = [window.innerWidth, window.innerHeight, window.outerWidth, window.outerHeight].join(':');
            const resizeByRef = window.resizeBy;
            const ret1 = window.resizeBy(20, -10);
            const mid = [window.innerWidth, window.innerHeight, window.outerWidth, window.outerHeight].join(':');
            const ret2 = resizeByRef(5, 15);
            const ret3 = resizeBy(-10, -5);
            const after = [window.innerWidth, window.innerHeight, window.outerWidth, window.outerHeight].join(':');
            document.getElementById('result').textContent =
              before + '|' + mid + '|' + after + '|' +
              String(ret1 === undefined) + ':' + String(ret2 === undefined) + ':' + String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1000:800::|1020:790:1020:790|1015:800:1015:800|true:true:true",
    )?;
    Ok(())
}

#[test]
fn window_resize_to_sets_dimensions_absolutely_and_supports_alias() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            window.innerWidth = 1000;
            window.innerHeight = 800;
            const before = [window.innerWidth, window.innerHeight, window.outerWidth, window.outerHeight].join(':');
            const resizeToRef = window.resizeTo;
            const ret1 = window.resizeTo(320, 240);
            const mid = [window.innerWidth, window.innerHeight, window.outerWidth, window.outerHeight].join(':');
            const ret2 = resizeToRef(640, 480);
            const ret3 = resizeTo(200, 100);
            const after = [window.innerWidth, window.innerHeight, window.outerWidth, window.outerHeight].join(':');
            document.getElementById('result').textContent =
              before + '|' + mid + '|' + after + '|' +
              String(ret1 === undefined) + ':' + String(ret2 === undefined) + ':' + String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1000:800::|320:240:320:240|200:100:200:100|true:true:true",
    )?;
    Ok(())
}

#[test]
fn window_post_message_dispatches_message_events_and_supports_alias() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const payload = { kind: 'obj' };
            const log = [];
            window.addEventListener('message', (event) => {
              const value =
                typeof event.data === 'object' ? event.data.kind : String(event.data);
              const cloned =
                typeof event.data === 'object' ? String(event.data !== payload) : 'n/a';
              log.push(
                value +
                  ',' +
                  String(event.origin === window.origin) +
                  ',' +
                  String(event.source === window) +
                  ',' +
                  cloned,
              );
            });

            const postRef = window.postMessage;
            const ret1 = window.postMessage(payload, '*');
            const ret2 = postRef('text', '*');
            const ret3 = postMessage('slash', '/');

            document.getElementById('result').textContent =
              log.join('|') +
              '|' +
              String(ret1 === undefined) +
              ':' +
              String(ret2 === undefined) +
              ':' +
              String(ret3 === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "obj,true,true,true|text,true,true,n/a|slash,true,true,n/a|true:true:true",
    )?;
    Ok(())
}

#[test]
fn window_post_message_honors_target_origin_and_options_overload() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const seen = [];
            window.addEventListener('message', () => {
              seen.push('x');
            });
            window.postMessage('default');
            window.postMessage('drop', 'https://evil.example');
            window.postMessage('exact', window.origin);
            window.postMessage('options', { targetOrigin: window.origin });
            window.postMessage('wildcard', { targetOrigin: '*' });
            window.postMessage('legacy', window.origin, [1, 2, 3]);
            window.postMessage('drop-options', { targetOrigin: 'https://evil.example' });
            document.getElementById('result').textContent = String(seen.length);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "5")?;
    Ok(())
}

#[test]
fn window_move_by_requires_two_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          moveBy(1);
        </script>
        "#,
    )
    .expect_err("moveBy with one argument should fail");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "moveBy requires exactly two arguments"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_move_to_requires_two_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          moveTo(1);
        </script>
        "#,
    )
    .expect_err("moveTo with one argument should fail");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "moveTo requires exactly two arguments"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_resize_by_requires_two_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          resizeBy(1);
        </script>
        "#,
    )
    .expect_err("resizeBy with one argument should fail");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "resizeBy requires exactly two arguments"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_resize_to_requires_two_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          resizeTo(1);
        </script>
        "#,
    )
    .expect_err("resizeTo with one argument should fail");

    match err {
        Error::ScriptRuntime(msg) => assert_eq!(msg, "resizeTo requires exactly two arguments"),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_post_message_requires_one_to_three_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          postMessage();
        </script>
        "#,
    )
    .expect_err("postMessage with no arguments should fail");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "postMessage requires one to three arguments")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_scroll_supports_zero_one_or_two_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          scroll(1, 2, 3);
        </script>
        "#,
    )
    .expect_err("scroll with three arguments should fail");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "scroll supports zero, one, or two arguments")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_scroll_by_supports_zero_one_or_two_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          scrollBy(1, 2, 3);
        </script>
        "#,
    )
    .expect_err("scrollBy with three arguments should fail");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "scrollBy supports zero, one, or two arguments")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn window_scroll_to_supports_zero_one_or_two_arguments() {
    let err = Harness::from_html(
        r#"
        <script>
          scrollTo(1, 2, 3);
        </script>
        "#,
    )
    .expect_err("scrollTo with three arguments should fail");

    match err {
        Error::ScriptRuntime(msg) => {
            assert_eq!(msg, "scrollTo supports zero, one, or two arguments")
        }
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
fn form_data_get_returns_first_value_and_null_for_missing_key() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const formData = new FormData();
            formData.append('username', 'Chris');
            formData.append('username', 'Bob');
            document.getElementById('result').textContent =
              formData.get('username') + ':' + (formData.get('missing') === null);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Chris:true")?;
    Ok(())
}

#[test]
fn form_data_has_is_false_before_append_and_true_after_append() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const formData = new FormData();
            const before = formData.has('username');
            formData.append('username', 'Chris');
            const after = formData.has('username');
            document.getElementById('result').textContent = before + ':' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true")?;
    Ok(())
}

#[test]
fn form_data_constructor_without_arguments_creates_empty_form_data() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData();
            fd.append('username', 'Chris');
            document.getElementById('result').textContent =
              fd.get('username') + ':' +
              fd.has('missing') + ':' +
              fd.getAll('username').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Chris:false:1")?;
    Ok(())
}

#[test]
fn form_data_constructor_with_submitter_includes_submitter_data() -> Result<()> {
    let html = r#"
        <form id='form'>
          <input type='text' name='text1' value='foo'>
          <input type='text' name='text2' value='bar'>
          <input type='text' name='text3' value='baz'>
          <input type='checkbox' name='check' checked disabled>
          <button name='intent' value='save'>Save</button>
          <button name='intent' value='saveAsCopy'>Save As Copy</button>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('form');
            const submitter = document.querySelector('button[value=save]');
            const fd = new FormData(form, submitter);
            document.getElementById('result').textContent =
              fd.get('text1') + ':' +
              fd.get('text2') + ':' +
              fd.get('text3') + ':' +
              fd.has('check') + ':' +
              fd.get('intent') + ':' +
              fd.getAll('intent').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "foo:bar:baz:false:save:1")?;
    Ok(())
}

#[test]
fn form_data_inline_constructor_without_form_supports_methods() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              new FormData().get('username') + ':' +
              new FormData().has('username') + ':' +
              new FormData().getAll('username').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "null:false:0")?;
    Ok(())
}

#[test]
fn form_data_inline_constructor_with_submitter_supports_methods() -> Result<()> {
    let html = r#"
        <form id='form'>
          <input type='text' name='text1' value='foo'>
          <button name='intent' value='save'>Save</button>
          <button name='intent' value='saveAsCopy'>Save As Copy</button>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('form');
            const submitter = document.querySelector('button[value=save]');
            document.getElementById('result').textContent =
              new FormData(form, submitter).get('intent') + ':' +
              new FormData(form, submitter).getAll('intent').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "save:1")?;
    Ok(())
}

#[test]
fn form_data_constructor_rejects_non_submit_button_submitter() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='field' name='field' value='x'>
          <button id='ok' name='intent' value='save'>Save</button>
        </form>
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('f');
            const submitter = document.getElementById('field');
            new FormData(form, submitter);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("non-submit control must throw a TypeError");
    match err {
        Error::ScriptRuntime(message) => {
            assert!(message.contains("TypeError"));
            assert!(message.contains("submitter must be a submit button"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_data_constructor_rejects_submitter_from_another_form() -> Result<()> {
    let html = r#"
        <form id='left'>
          <input name='text1' value='foo'>
          <button name='intent' value='save'>Save</button>
        </form>
        <form id='right'>
          <button id='foreign' name='intent' value='saveAsCopy'>Save As Copy</button>
        </form>
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const form = document.getElementById('left');
            const submitter = document.getElementById('foreign');
            new FormData(form, submitter);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("foreign submitter must throw NotFoundError");
    match err {
        Error::ScriptRuntime(message) => {
            assert!(message.contains("NotFoundError"));
            assert!(message.contains("member of the form"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
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
    h.assert_text("#result", "Hanako:false:null")?;
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
fn form_data_append_returns_undefined_and_coerces_non_string_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData();
            const returned = fd.append('name', true);
            fd.append('name', 72);
            document.getElementById('result').textContent =
              String(returned) + ':' + fd.getAll('name').join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:true|72")?;
    Ok(())
}

#[test]
fn form_data_append_with_blob_and_filename_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData();
            const blob = new Blob(['abc'], { type: 'text/plain' });
            fd.append('user-pic', blob, 'chris.jpg');
            fd.append('raw-blob', blob);
            document.getElementById('result').textContent =
              fd.get('user-pic') + ':' +
              fd.get('raw-blob') + ':' +
              fd.getAll('user-pic').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "chris.jpg:blob:1")?;
    Ok(())
}

#[test]
fn form_data_append_three_argument_expression_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData();
            const blob = new Blob(['abc'], { type: 'text/plain' });
            const returned = fd.append('photo', blob, 'photo.jpg');
            document.getElementById('result').textContent =
              String(returned) + ':' + fd.get('photo');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:photo.jpg")?;
    Ok(())
}

#[test]
fn form_data_append_with_file_uses_default_and_override_filenames() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData();
            const f1 = new File(['ab'], 'sample.csv', { type: 'text/csv' });
            const f2 = new File(['cd'], 'report.csv', { type: 'text/csv' });
            fd.append('doc', f1);
            fd.append('doc', f2, 'renamed.csv');
            document.getElementById('result').textContent =
              fd.getAll('doc').join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "sample.csv|renamed.csv")?;
    Ok(())
}

#[test]
fn form_data_set_overwrites_existing_values_and_returns_undefined() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData();
            fd.append('username', 'Chris');
            fd.append('username', 'Bob');
            const returned = fd.set('username', 'Alice');
            document.getElementById('result').textContent =
              String(returned) + ':' +
              fd.get('username') + ':' +
              fd.getAll('username').length + ':' +
              fd.getAll('username').join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:Alice:1:Alice")?;
    Ok(())
}

#[test]
fn form_data_set_with_blob_filename_and_default_name_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData();
            const blob = new Blob(['abc'], { type: 'text/plain' });
            fd.set('user-pic', blob, 'chris.jpg');
            fd.set('raw-blob', blob);
            document.getElementById('result').textContent =
              fd.get('user-pic') + ':' +
              fd.get('raw-blob') + ':' +
              fd.getAll('user-pic').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "chris.jpg:blob:1")?;
    Ok(())
}

#[test]
fn form_data_set_coerces_non_string_value_and_adds_when_missing() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData();
            fd.set('name', 72);
            document.getElementById('result').textContent =
              fd.has('name') + ':' + fd.get('name') + ':' + fd.getAll('name').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:72:1")?;
    Ok(())
}

#[test]
fn form_data_set_inline_constructor_and_arity_validation_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              String(new FormData().set('name', 'Chris'));
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined")?;

    let err = Harness::from_html(
        r#"
        <script>
          const fd = new FormData();
          fd.set('name');
        </script>
        "#,
    )
    .expect_err("FormData.set should reject invalid arity");
    match err {
        Error::ScriptRuntime(message) => {
            assert!(message.contains("FormData.set requires two or three arguments"));
        }
        other => panic!("unexpected error: {other:?}"),
    }

    Ok(())
}

#[test]
fn form_data_delete_removes_all_values_for_key_and_returns_undefined() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='t1' name='tag' value='A'>
          <input id='t2' name='tag' value='B'>
          <input id='o1' name='other' value='Z'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData(document.getElementById('f'));
            fd.append('tag', 'C');
            const returned = fd.delete('tag');
            document.getElementById('result').textContent =
              String(returned) + ':' +
              fd.has('tag') + ':' +
              fd.get('tag') + ':' +
              fd.getAll('tag').length + ':' +
              fd.get('other');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:false:null:0:Z")?;
    Ok(())
}

#[test]
fn form_data_delete_missing_key_is_noop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData();
            fd.append('name', 'A');
            fd.delete('missing');
            document.getElementById('result').textContent =
              fd.get('name') + ':' + fd.getAll('name').length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A:1")?;
    Ok(())
}

#[test]
fn form_data_entries_returns_all_pairs_in_order() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='k1' name='key1' value='value1'>
          <input id='k2' name='key2' value='value2'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = new FormData(document.getElementById('f'));
            fd.append('key1', 'value3');
            let out = '';
            for (const pair of fd.entries()) {
              out = out + pair[0] + '=' + pair[1] + ';';
            }
            document.getElementById('result').textContent =
              out + '|' + fd.entries().length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "key1=value1;key2=value2;key1=value3;|3")?;
    Ok(())
}

#[test]
fn form_data_entries_rejects_arguments() -> Result<()> {
    let err = Harness::from_html(
        r#"
        <script>
          const fd = new FormData();
          fd.entries('extra');
        </script>
        "#,
    )
    .expect_err("FormData.entries should reject arguments");

    match err {
        Error::ScriptRuntime(message) => {
            assert!(message.contains("FormData.entries does not take arguments"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_data_keys_returns_all_keys_in_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const formData = new FormData();
            formData.append('key1', 'value1');
            formData.append('key2', 'value2');
            formData.append('key1', 'value3');
            let out = '';
            for (const key of formData.keys()) {
              out = out + key + ';';
            }
            document.getElementById('result').textContent =
              out + '|' + formData.keys().length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "key1;key2;key1;|3")?;
    Ok(())
}

#[test]
fn form_data_keys_rejects_arguments() -> Result<()> {
    let err = Harness::from_html(
        r#"
        <script>
          const fd = new FormData();
          fd.keys('extra');
        </script>
        "#,
    )
    .expect_err("FormData.keys should reject arguments");

    match err {
        Error::ScriptRuntime(message) => {
            assert!(message.contains("FormData.keys does not take arguments"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_data_keys_inline_constructor_returns_keys() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='k1' name='key1' value='value1'>
          <input id='k2' name='key2' value='value2'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const keys = new FormData(document.getElementById('f')).keys();
            document.getElementById('result').textContent =
              keys.length + ':' + keys.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:key1|key2")?;
    Ok(())
}

#[test]
fn form_data_values_returns_all_values_in_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const formData = new FormData();
            formData.append('key1', 'value1');
            formData.append('key2', 'value2');
            formData.append('key1', 'value3');
            let out = '';
            for (const value of formData.values()) {
              out = out + value + ';';
            }
            document.getElementById('result').textContent =
              out + '|' + formData.values().length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "value1;value2;value3;|3")?;
    Ok(())
}

#[test]
fn form_data_values_rejects_arguments() -> Result<()> {
    let err = Harness::from_html(
        r#"
        <script>
          const fd = new FormData();
          fd.values('extra');
        </script>
        "#,
    )
    .expect_err("FormData.values should reject arguments");

    match err {
        Error::ScriptRuntime(message) => {
            assert!(message.contains("FormData.values does not take arguments"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn form_data_values_inline_constructor_returns_values() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='k1' name='key1' value='value1'>
          <input id='k2' name='key2' value='value2'>
        </form>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const values = new FormData(document.getElementById('f')).values();
            document.getElementById('result').textContent =
              values.length + ':' + values.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:value1|value2")?;
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
fn form_data_get_all_returns_all_values_for_same_name() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const formData = new FormData();
            formData.append('username', 'Chris');
            formData.append('username', 'Bob');
            const users = formData.getAll('username');
            const missing = formData.getAll('missing');
            document.getElementById('result').textContent =
              users.length + ':' + users.join('|') + ':' + missing.length;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:Chris|Bob:0")?;
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
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const fd = 1;
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
