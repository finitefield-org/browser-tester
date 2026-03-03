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
