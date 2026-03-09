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
