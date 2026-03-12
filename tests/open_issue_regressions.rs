use browser_tester::Harness;

#[test]
fn click_toggles_button_inside_open_dialog() -> browser_tester::Result<()> {
    let html = r#"
    <button id="open">Open</button>
    <div id="dialog" class="hidden" role="dialog" aria-modal="true">
      <button id="settings-toggle" type="button" aria-expanded="false">Settings</button>
      <div id="settings-panel" class="hidden">Panel</div>
    </div>
    <p id="status"></p>
    <p id="trace"></p>
    <script>
      (() => {
        const el = {
          open: document.getElementById("open"),
          dialog: document.getElementById("dialog"),
          settingsToggle: document.getElementById("settings-toggle"),
          settingsPanel: document.getElementById("settings-panel"),
          status: document.getElementById("status"),
          trace: document.getElementById("trace"),
        };

        let settingsOpen = false;

        function setHiddenClass(node, hidden) {
          node.classList.toggle("hidden", hidden);
        }

        function syncStatus() {
          el.status.textContent = [
            String(settingsOpen),
            el.settingsToggle.getAttribute("aria-expanded"),
            String(el.settingsPanel.classList.contains("hidden")),
          ].join("|");
        }

        function render() {
          el.trace.textContent += "render>";
          setHiddenClass(el.settingsPanel, !settingsOpen);
          el.settingsToggle.setAttribute("aria-expanded", settingsOpen ? "true" : "false");
          syncStatus();
        }

        el.open.addEventListener("click", () => {
          el.trace.textContent += "open>";
          el.dialog.classList.remove("hidden");
          syncStatus();
        });

        el.settingsToggle.addEventListener("click", () => {
          el.trace.textContent += "toggle>";
          settingsOpen = !settingsOpen;
          render();
        });

        syncStatus();
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#open")?;
    harness.click("#settings-toggle")?;

    harness.assert_text("#trace", "open>toggle>render>")?;
    harness.assert_text("#status", "true|true|false")?;
    Ok(())
}

#[test]
fn click_preserves_pre_request_animation_frame_processing_state() -> browser_tester::Result<()> {
    let html = r#"
    <button id="run" type="button">Run</button>
    <div id="processing" class="hidden">Processing</div>
    <p id="status"></p>
    <script>
      (() => {
        const el = {
          run: document.getElementById("run"),
          processing: document.getElementById("processing"),
          status: document.getElementById("status"),
        };

        function setProcessing(processing) {
          el.processing.classList.toggle("hidden", !processing);
          el.run.disabled = processing;
          el.status.textContent = [
            String(el.run.disabled),
            String(el.processing.classList.contains("hidden")),
          ].join("|");
        }

        function nextFrame() {
          return new Promise((resolve) => {
            window.requestAnimationFrame(() => resolve());
          });
        }

        async function runTask() {
          setProcessing(true);
          await nextFrame();
          setProcessing(false);
        }

        el.run.addEventListener("click", runTask);
        setProcessing(false);
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;

    harness.assert_text("#status", "true|false")?;

    harness.flush()?;

    harness.assert_text("#status", "false|true")?;
    Ok(())
}
