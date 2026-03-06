use super::*;

#[test]
fn error_event_constructor_populates_properties() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const e = new ErrorEvent('error', {
            message: 'boom',
            filename: 'worker.js',
            lineno: 12,
            colno: 34,
            error: { code: 'E_BANG' },
            bubbles: true,
            cancelable: true
          });
          document.getElementById('result').textContent = [
            e.type,
            e.message,
            e.filename,
            e.lineno,
            e.colno,
            e.error.code,
            e.bubbles,
            e.cancelable,
            e.isTrusted
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "error:boom:worker.js:12:34:E_BANG:true:true:false",
    )?;
    Ok(())
}

#[test]
fn dispatch_event_with_error_event_payload_preserves_fields() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.addEventListener('error', (event) => {
            document.getElementById('result').textContent = [
              event.message,
              event.filename,
              event.lineno,
              event.colno,
              event.error.code,
              event.isTrusted
            ].join(':');
          });

          document.getElementById('run').addEventListener('click', () => {
            const ev = new ErrorEvent('error', {
              message: 'from-dispatch',
              filename: 'app.js',
              lineno: 7,
              colno: 9,
              error: { code: 'E_DISPATCH' }
            });
            target.dispatchEvent(ev);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "from-dispatch:app.js:7:9:E_DISPATCH:false")?;
    Ok(())
}

#[test]
fn error_event_constructor_rejects_non_object_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let threw = false;
            try {
              new ErrorEvent('error', 123);
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

#[test]
fn error_event_constructor_is_available_in_worker_context() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const result = document.getElementById('result');
          document.getElementById('run').addEventListener('click', () => {
            const source = `
              self.onmessage = () => {
                try {
                  const ev = new ErrorEvent('error', {
                    message: 'worker-boom',
                    filename: 'worker.js',
                    lineno: 5,
                    colno: 8,
                    error: { code: 'E_WORKER' }
                  });
                  self.postMessage([
                    ev.type,
                    ev.message,
                    ev.filename,
                    ev.lineno,
                    ev.colno,
                    ev.error.code,
                    ev.isTrusted
                  ].join(':'));
                } catch (error) {
                  self.postMessage('ERR:' + String(error));
                }
              };
            `;
            const blob = new Blob([source], { type: 'text/javascript' });
            const worker = new Worker(URL.createObjectURL(blob));
            worker.onmessage = (event) => {
              result.textContent = String(event.data);
              worker.terminate();
            };
            worker.postMessage('run');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "error:worker-boom:worker.js:5:8:E_WORKER:false")?;
    Ok(())
}
