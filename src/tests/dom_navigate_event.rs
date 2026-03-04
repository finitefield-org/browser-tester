use super::*;

#[test]
fn navigate_event_constructor_populates_fields_and_methods() -> Result<()> {
    let html = r#"
        <a id='source' href='/next'>go</a>
        <p id='result'></p>
        <script>
          const source = document.getElementById('source');
          const destination = { url: 'https://app.local/articles/1' };
          const formData = { kind: 'f' };
          const info = { from: 'back' };
          const signal = { aborted: false };
          const ev = new NavigateEvent('navigate', {
            bubbles: true,
            cancelable: true,
            canIntercept: true,
            destination,
            downloadRequest: 'book.pdf',
            formData,
            hashChange: true,
            hasUAVisualTransition: true,
            info,
            navigationType: 'replace',
            signal,
            sourceElement: source,
            userInitiated: true
          });
          document.getElementById('result').textContent = [
            ev.type,
            ev.bubbles,
            ev.cancelable,
            ev.canIntercept,
            ev.destination === destination,
            ev.downloadRequest,
            ev.formData === formData,
            ev.hashChange,
            ev.hasUAVisualTransition,
            ev.info === info,
            ev.navigationType,
            ev.signal === signal,
            ev.sourceElement === source,
            ev.userInitiated,
            typeof ev.intercept,
            typeof ev.scroll
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "navigate:true:true:true:true:book.pdf:true:true:true:true:replace:true:true:true:function:function",
    )?;
    Ok(())
}

#[test]
fn navigate_event_constructor_uses_expected_defaults() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const ev = new NavigateEvent('navigate');
          document.getElementById('result').textContent = [
            ev.canIntercept,
            ev.destination === null,
            ev.downloadRequest === null,
            ev.formData === null,
            ev.hashChange,
            ev.hasUAVisualTransition,
            ev.info === undefined,
            ev.navigationType,
            typeof ev.signal,
            ev.signal && ev.signal.aborted === false,
            ev.sourceElement === null,
            ev.userInitiated
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "false:true:true:true:false:false:true:push:object:true:true:false",
    )?;
    Ok(())
}

#[test]
fn navigate_event_intercept_runs_handler_and_throws_when_not_interceptable() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ev = new NavigateEvent('navigate', { canIntercept: true });
            let handled = 0;
            ev.intercept({
              handler() {
                handled += 1;
              }
            });
            ev.scroll();

            let threw = false;
            try {
              new NavigateEvent('navigate').intercept();
            } catch (err) {
              threw = String(err).includes('InvalidStateError');
            }
            document.getElementById('result').textContent = [handled, threw].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:true")?;
    Ok(())
}

#[test]
fn dispatch_event_with_navigate_event_payload_preserves_navigate_fields() -> Result<()> {
    let html = r#"
        <a id='source' href='/next'>go</a>
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.addEventListener('navigate', (event) => {
            let handled = 0;
            event.intercept({
              handler() {
                handled += 1;
              }
            });
            event.scroll();
            document.getElementById('result').textContent = [
              event.canIntercept,
              event.destination.url,
              event.navigationType,
              event.userInitiated,
              event.sourceElement ? event.sourceElement.id : 'none',
              typeof event.signal,
              event.signal && event.signal.aborted === false,
              handled
            ].join(':');
          });

          document.getElementById('run').addEventListener('click', () => {
            const source = document.getElementById('source');
            const ev = new NavigateEvent('navigate', {
              canIntercept: true,
              destination: { url: 'https://app.local/next' },
              navigationType: 'push',
              userInitiated: true,
              sourceElement: source
            });
            target.dispatchEvent(ev);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:https://app.local/next:push:true:source:object:true:1",
    )?;
    Ok(())
}

#[test]
fn navigate_event_constructor_rejects_non_object_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let threw = false;
            try {
              new NavigateEvent('navigate', 123);
            } catch (error) {
              threw = String(error).includes('options argument must be an object');
            }
            document.getElementById('result').textContent = String(threw);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true")?;
    Ok(())
}
