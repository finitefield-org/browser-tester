use super::*;

#[test]
fn issue_102_worker_regex_exec_result_is_indexable_array() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `
            self.onmessage = (event) => {
              try {
                const regex = /\\d+/g;
                const matched = regex.exec('a1 b22 c333');
                self.postMessage({ ok: true, text: matched[0], len: matched.length });
              } catch (error) {
                self.postMessage({
                  ok: false,
                  message: String(error && (error.message || error))
                });
              }
            };
          `;
          const blob = new Blob([source], { type: 'text/javascript' });
          const worker = new Worker(URL.createObjectURL(blob));
          worker.onmessage = (ev) => {
            out.textContent = JSON.stringify(ev.data || {});
            worker.terminate();
          };
          worker.postMessage('run');
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", r#"{"ok":true,"text":"1","len":1}"#)?;
    Ok(())
}

#[test]
fn issue_119_worker_regex_exec_assignment_in_while_condition_preserves_match_array() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `
            self.onmessage = function(event) {
              try {
                const req = event.data || {};
                const regex = new RegExp(String(req.pattern || ''), String(req.flags || ''));
                const sourceText = String(req.source || '');

                let matched;
                while ((matched = regex.exec(sourceText)) !== null) {
                  const full = matched[0];
                  self.postMessage({ ok: true, full, len: matched.length });
                  return;
                }

                self.postMessage({ ok: true, full: null });
              } catch (error) {
                self.postMessage({ ok: false, raw: String(error) });
              }
            };
          `;
          const blob = new Blob([source], { type: 'text/javascript' });
          const worker = new Worker(URL.createObjectURL(blob));
          worker.onmessage = (ev) => {
            out.textContent = JSON.stringify(ev.data || {});
            worker.terminate();
          };
          worker.postMessage({ pattern: '\\d+', flags: 'g', source: 'a1 b22 c333' });
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", r#"{"ok":true,"full":"1","len":1}"#)?;
    Ok(())
}

#[test]
fn worker_object_url_is_snapshot_at_construction_and_survives_immediate_revoke() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `
            self.onmessage = () => {
              self.postMessage('ok');
            };
          `;
          const blob = new Blob([source], { type: 'text/javascript' });
          const url = URL.createObjectURL(blob);
          const worker = new Worker(url);
          URL.revokeObjectURL(url);
          worker.onmessage = (ev) => {
            out.textContent = String(ev.data);
            worker.terminate();
          };
          worker.postMessage('run');
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "ok")?;
    Ok(())
}

#[test]
fn revoked_object_url_worker_constructor_throws_not_found() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const blob = new Blob(['self.onmessage = () => {};'], { type: 'text/javascript' });
          const url = URL.createObjectURL(blob);
          URL.revokeObjectURL(url);
          try {
            new Worker(url);
            out.textContent = 'missing-error';
          } catch (error) {
            out.textContent = String(error && error.message ? error.message : error);
          }
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "Worker script source not found: blob:bt-1")?;
    Ok(())
}

#[test]
fn worker_constructor_surface_and_prototype_branding_work() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const blob = new Blob(['self.onmessage = () => {};'], { type: 'text/javascript' });
          const url = URL.createObjectURL(blob);
          const worker = new Worker(url);
          URL.revokeObjectURL(url);

          const proto = Object.getPrototypeOf(worker);
          const postDesc = Object.getOwnPropertyDescriptor(Worker.prototype, 'postMessage');
          const termDesc = Object.getOwnPropertyDescriptor(Worker.prototype, 'terminate');

          out.textContent = [
            typeof Worker,
            String(window.Worker === Worker),
            String(worker.constructor === Worker),
            String(proto === Worker.prototype),
            String(Object.getPrototypeOf(Worker.prototype) === Object.prototype),
            Object.getOwnPropertyNames(Worker.prototype).sort().join(','),
            String(Object.keys(Worker.prototype).length === 0),
            String(postDesc.enumerable),
            String(postDesc.configurable),
            String(postDesc.writable),
            String(worker.postMessage === Worker.prototype.postMessage),
            String(termDesc.enumerable),
            String(termDesc.configurable),
            String(termDesc.writable),
            String(worker.terminate === Worker.prototype.terminate),
            Object.prototype.toString.call(worker),
          ].join('|');

          worker.terminate();
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text(
        "#out",
        "function|true|true|true|true|constructor,postMessage,terminate|true|false|true|true|false|false|true|true|false|[object Worker]",
    )?;
    Ok(())
}

#[test]
fn worker_post_message_structured_clones_payloads_work() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `
            self.onmessage = (event) => {
              try {
                event.data.nested.value = 9;
                event.data.items.push(3);
                self.postMessage({
                  fromInput: event.data.nested.value,
                  fromInputLen: event.data.items.length,
                });
              } catch (error) {
                self.postMessage({ error: String(error) });
              }
            };
          `;

          const blob = new Blob([source], { type: 'text/javascript' });
          const url = URL.createObjectURL(blob);
          const worker = new Worker(url);
          URL.revokeObjectURL(url);

          const payload = {
            nested: { value: 1 },
            items: [1, 2],
          };

          worker.onmessage = (event) => {
            if (event.data && event.data.error) {
              out.textContent = 'ERR:' + event.data.error;
              worker.terminate();
              return;
            }
            out.textContent = [
              payload.nested.value,
              payload.items.length,
              event.data.fromInput,
              event.data.fromInputLen,
            ].join(':');
            worker.terminate();
          };

          worker.postMessage(payload);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "1:2:9:3")?;
    Ok(())
}

#[test]
fn worker_messages_are_delivered_after_same_task_handler_registration_work() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `
            self.onmessage = (event) => {
              self.postMessage('ack:' + String(event.data));
            };
          `;
          const blob = new Blob([source], { type: 'text/javascript' });
          const worker = new Worker(URL.createObjectURL(blob));
          worker.postMessage('ping');
          worker.onmessage = (event) => {
            out.textContent = String(event.data);
            worker.terminate();
          };
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "ack:ping")?;
    Ok(())
}

#[test]
fn worker_boot_messages_wait_until_end_of_task_for_handler_registration_work() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `self.postMessage('boot');`;
          const blob = new Blob([source], { type: 'text/javascript' });
          const worker = new Worker(URL.createObjectURL(blob));
          worker.onmessage = (event) => {
            out.textContent = String(event.data);
            worker.terminate();
          };
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "boot")?;
    Ok(())
}

#[test]
fn worker_terminate_suppresses_queued_message_delivery_work() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `
            self.onmessage = (event) => {
              self.postMessage('ack:' + String(event.data));
            };
          `;
          const blob = new Blob([source], { type: 'text/javascript' });
          const worker = new Worker(URL.createObjectURL(blob));
          worker.onmessage = (event) => {
            out.textContent = String(event.data);
          };
          worker.postMessage('ping');
          worker.terminate();
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "")?;
    Ok(())
}

#[test]
fn worker_multiple_queued_messages_preserve_fifo_after_same_task_handler_registration_work()
-> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `
            self.onmessage = (event) => {
              self.postMessage('first:' + String(event.data));
              self.postMessage('second:' + String(event.data));
            };
          `;
          const blob = new Blob([source], { type: 'text/javascript' });
          const worker = new Worker(URL.createObjectURL(blob));
          const seen = [];
          worker.postMessage('ping');
          worker.onmessage = (event) => {
            seen.push(String(event.data));
            if (seen.length >= 2) {
              out.textContent = seen.join('|');
              worker.terminate();
            }
          };
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "first:ping|second:ping")?;
    Ok(())
}

#[test]
fn worker_boot_messages_preserve_fifo_after_end_of_task_handler_registration_work() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `
            self.postMessage('boot:1');
            self.postMessage('boot:2');
          `;
          const blob = new Blob([source], { type: 'text/javascript' });
          const worker = new Worker(URL.createObjectURL(blob));
          const seen = [];
          worker.onmessage = (event) => {
            seen.push(String(event.data));
            if (seen.length >= 2) {
              out.textContent = seen.join('|');
              worker.terminate();
            }
          };
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "boot:1|boot:2")?;
    Ok(())
}

#[test]
fn worker_post_message_structured_clone_isolated_per_queued_send_work() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <div id='out'></div>
      <script>
        const out = document.getElementById('out');
        document.getElementById('run').addEventListener('click', () => {
          const source = `
            self.onmessage = (event) => {
              self.postMessage({
                value: event.data.nested.value,
                len: event.data.items.length,
              });
            };
          `;

          const blob = new Blob([source], { type: 'text/javascript' });
          const worker = new Worker(URL.createObjectURL(blob));
          const payload = {
            nested: { value: 1 },
            items: [1],
          };
          const seen = [];

          worker.onmessage = (event) => {
            seen.push(String(event.data.value) + ':' + String(event.data.len));
            if (seen.length >= 2) {
              out.textContent = seen.join('|') + '|main:' + payload.nested.value + ':' + payload.items.length;
              worker.terminate();
            }
          };

          worker.postMessage(payload);
          payload.nested.value = 7;
          payload.items.push(2);
          worker.postMessage(payload);
          payload.nested.value = 9;
          payload.items.push(3);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", "1:1|7:2|main:9:3")?;
    Ok(())
}
