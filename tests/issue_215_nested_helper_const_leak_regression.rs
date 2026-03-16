use browser_tester::Harness;

#[test]
fn issue_215_nested_helper_local_index_does_not_poison_later_const_index()
-> browser_tester::Result<()> {
    let html = r#"
      <button id="go" type="button">go</button>
      <div id="out"></div>
      <script>
        (() => {
          const state = {
            stack: {
              steps: [
                { id: "step-1", config: { type: "percent", percent: { rateRaw: "10" } } },
                { id: "step-2", config: { type: "fixed", fixed: { amountRaw: "10" } } }
              ]
            }
          };

          function setDeepValue(obj, path, value) {
            if (!obj || !path) return;
            const parts = path.split(".");
            let current = obj;
            for (let index = 0; index < parts.length - 1; index += 1) {
              const part = parts[index];
              if (!current[part] || typeof current[part] !== "object") {
                current[part] = {};
              }
              current = current[part];
            }
            current[parts[parts.length - 1]] = value;
          }

          function describeStep(step) {
            if (step.config.type === "percent") {
              return step.id + ":" + step.config.percent.rateRaw;
            }
            return step.id + ":" + step.config.fixed.amountRaw;
          }

          function render() {
            document.getElementById("out").textContent = state.stack.steps
              .map((step) => describeStep(step))
              .join("|");
          }

          document.getElementById("go").addEventListener("click", () => {
            const edited = state.stack.steps.find((item) => item.id === "step-1");
            if (!edited) return;
            setDeepValue(edited.config, "percent.rateRaw", "20");

            const index = state.stack.steps.findIndex((item) => item.id === "step-2");
            const moved = state.stack.steps.splice(index, 1)[0];
            state.stack.steps.splice(index - 1, 0, moved);
            render();
          });

          render();
        })();
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "step-2:10|step-1:20")?;
    Ok(())
}

#[test]
fn issue_215_nested_helper_local_index_does_not_poison_plain_const_declaration()
-> browser_tester::Result<()> {
    let html = r#"
      <button id="go" type="button">go</button>
      <div id="out"></div>
      <script>
        (() => {
          const state = { nested: {} };

          function setDeepValue(obj, path, value) {
            const parts = path.split(".");
            let current = obj;
            for (let index = 0; index < parts.length - 1; index += 1) {
              const part = parts[index];
              if (!current[part] || typeof current[part] !== "object") {
                current[part] = {};
              }
              current = current[part];
            }
            current[parts[parts.length - 1]] = value;
          }

          document.getElementById("go").addEventListener("click", () => {
            setDeepValue(state.nested, "percent.rateRaw", "20");
            const index = 1;
            document.getElementById("out").textContent =
              String(index) + ":" + state.nested.percent.rateRaw;
          });
        })();
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "1:20")?;
    Ok(())
}
