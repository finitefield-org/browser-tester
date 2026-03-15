use browser_tester::{Harness, KeyboardEventInit};

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

#[test]
fn dispatch_keyboard_completes_async_keydown_handlers_waiting_for_animation_frame()
-> browser_tester::Result<()> {
    let html = r#"
    <textarea id="input"></textarea>
    <textarea id="result"></textarea>
    <script>
      (() => {
        const input = document.getElementById("input");
        const result = document.getElementById("result");

        function nextFrame() {
          return new Promise((resolve) => {
            window.requestAnimationFrame(() => resolve());
          });
        }

        async function runDedupe() {
          await nextFrame();
          const seen = new Set();
          const lines = [];
          for (const rawLine of input.value.split(/\r?\n/)) {
            if (rawLine === "" || seen.has(rawLine)) {
              continue;
            }
            seen.add(rawLine);
            lines.push(rawLine);
          }
          result.value = lines.join("\n");
        }

        document.addEventListener("keydown", (event) => {
          if ((event.ctrlKey || event.metaKey) && !event.shiftKey && event.key === "Enter") {
            event.preventDefault();
            runDedupe();
          }
        });
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.type_text("#input", "A\nA\nB")?;
    harness.dispatch_keyboard(
        "document",
        "keydown",
        KeyboardEventInit {
            key: "Enter".to_string(),
            ctrl_key: true,
            ..KeyboardEventInit::default()
        },
    )?;

    harness.assert_value("#result", "")?;

    harness.flush()?;

    harness.assert_value("#result", "A\nB")?;
    Ok(())
}

#[test]
fn iife_helper_listener_reads_live_outer_let_after_sibling_render() -> browser_tester::Result<()> {
    let html = r#"
    <button id="open">Open</button>
    <button id="copy">Copy</button>
    <p id="result"></p>
    <script>
      (() => {
        let lastComputation = null;

        function bindActions() {
          document.getElementById("open").addEventListener("click", () => {
            document.body.setAttribute("data-opened", "yes");
          });
          document.getElementById("copy").addEventListener("click", () => {
            document.getElementById("result").textContent =
              lastComputation ? lastComputation.value : "null";
          });
        }

        function render() {
          lastComputation = { value: "1.23 mg/L", canCopy: true };
        }

        bindActions();
        render();
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#open")?;
    harness.click("#copy")?;
    harness.assert_text("#result", "1.23 mg/L")?;
    Ok(())
}

#[test]
fn nested_call_preserves_caller_local_close_binding() -> browser_tester::Result<()> {
    let html = r#"
    <button id="go" type="button">go</button>
    <div id="out"></div>
    <script>
      (() => {
        function make(source) {
          let index = 0;

          function current() {
            return source[index] || "";
          }

          function consume() {
            const char = source[index] || "";
            index += 1;
            return char;
          }

          function parseSequence(stopChar) {
            let seen = "";
            while (index < source.length && current() !== stopChar) {
              seen += consume();
            }
            return "seen=" + seen + "|stop=" + stopChar + "|curr=" + (current() || "<eof>") + "|index=" + index;
          }

          function parseBracketGroup() {
            const open = consume();
            const close = open === "(" ? ")" : "]";
            const inner = parseSequence(close);
            return "after=" + (current() || "<eof>") + "|close=" + close + "|index=" + index + "|" + inner;
          }

          return parseBracketGroup();
        }

        document.getElementById("go").addEventListener("click", () => {
          document.getElementById("out").textContent = make("(SO4)3");
        });
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text(
        "#out",
        "after=)|close=)|index=4|seen=SO4|stop=)|curr=)|index=4",
    )?;
    Ok(())
}

#[test]
fn nested_call_keeps_caller_local_binding_before_follow_up_calls() -> browser_tester::Result<()> {
    let html = r#"
    <button id="go" type="button">go</button>
    <div id="out"></div>
    <script>
      (() => {
        function make(source) {
          let index = 0;

          function consume() {
            const char = source[index] || "";
            index += 1;
            return char;
          }

          function parseSequence(stopChar) {
            let seen = "";
            while (index < source.length && source[index] !== stopChar) {
              seen += consume();
            }
            return seen;
          }

          function parseBracketGroup() {
            const open = consume();
            const close = open === "(" ? ")" : "]";
            const inner = parseSequence(close);
            return "close=" + close + "|inner=" + inner + "|index=" + index;
          }

          return parseBracketGroup();
        }

        document.getElementById("go").addEventListener("click", () => {
          document.getElementById("out").textContent = make("(SO4)3");
        });
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "close=)|inner=SO4|index=4")?;
    Ok(())
}

#[test]
fn nested_call_keeps_caller_local_binding_after_sibling_call() -> browser_tester::Result<()> {
    let html = r#"
    <button id="go" type="button">go</button>
    <div id="out"></div>
    <script>
      (() => {
        function make(source) {
          let index = 0;

          function current() {
            return source[index] || "";
          }

          function consume() {
            const char = source[index] || "";
            index += 1;
            return char;
          }

          function parseSequence(stopChar) {
            let seen = "";
            while (index < source.length && current() !== stopChar) {
              seen += consume();
            }
            return seen;
          }

          function parseBracketGroup() {
            const open = consume();
            const close = open === "(" ? ")" : "]";
            const inner = parseSequence(close);
            const after = current();
            return "close=" + close + "|after=" + after + "|inner=" + inner + "|index=" + index;
          }

          return parseBracketGroup();
        }

        document.getElementById("go").addEventListener("click", () => {
          document.getElementById("out").textContent = make("(SO4)3");
        });
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "close=)|after=)|inner=SO4|index=4")?;
    Ok(())
}

#[test]
fn trivial_nested_call_does_not_replace_local_close_with_window_close() -> browser_tester::Result<()>
{
    let html = r#"
    <button id="go" type="button">go</button>
    <div id="out"></div>
    <script>
      (() => {
        function make() {
          function noop() {
            return "ok";
          }

          function parseBracketGroup() {
            const close = ")";
            const inner = noop();
            return "close=" + close + "|inner=" + inner;
          }

          return parseBracketGroup();
        }

        document.getElementById("go").addEventListener("click", () => {
          document.getElementById("out").textContent = make();
        });
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "close=)|inner=ok")?;
    Ok(())
}

#[test]
fn nested_call_keeps_captured_index_visible_to_bare_reads() -> browser_tester::Result<()> {
    let html = r#"
    <button id="go" type="button">go</button>
    <div id="out"></div>
    <script>
      (() => {
        function make(source) {
          let index = 0;

          function current() {
            return source[index] || "";
          }

          function consume() {
            const char = source[index] || "";
            index += 1;
            return char;
          }

          function parseDigits() {
            const start = index;
            while (/[0-9]/.test(current())) {
              consume();
            }
            return "start=" + start + "|index=" + index + "|curr=" + current() + "|raw=" + source.slice(start, index);
          }

          consume();
          consume();
          return parseDigits();
        }

        document.getElementById("go").addEventListener("click", () => {
          document.getElementById("out").textContent = make("Al2(SO4)3");
        });
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "start=2|index=3|curr=(|raw=2")?;
    Ok(())
}

#[test]
fn nested_parse_number_keeps_outer_progress_visible() -> browser_tester::Result<()> {
    let html = r#"
    <button id="go" type="button">go</button>
    <div id="out"></div>
    <script>
      (() => {
        function createParser(source) {
          let index = 0;

          function current() {
            return source[index] || "";
          }

          function consume() {
            const char = source[index] || "";
            index += 1;
            return char;
          }

          function isDigit(char) {
            return /[0-9]/.test(char);
          }

          function isUpper(char) {
            return /[A-Z]/.test(char);
          }

          function isLower(char) {
            return /[a-z]/.test(char);
          }

          function parseNumber() {
            const start = index;
            while (isDigit(current())) {
              consume();
            }
            return source.slice(start, index);
          }

          function parseOptionalMultiplier() {
            if (isDigit(current())) return parseNumber();
            return "";
          }

          function parseElementSymbol() {
            const first = current();
            if (!isUpper(first)) {
              throw new Error("invalid symbol");
            }
            let symbol = consume();
            if (isLower(current())) symbol += consume();
            return symbol;
          }

          function parseElementGroup() {
            const symbol = parseElementSymbol();
            const count = parseOptionalMultiplier();
            return symbol + count + "|index=" + index + "|curr=" + current();
          }

          return parseElementGroup();
        }

        document.getElementById("go").addEventListener("click", () => {
          try {
            document.getElementById("out").textContent = createParser("Al2(SO4)3");
          } catch (error) {
            document.getElementById("out").textContent =
              error && error.message ? error.message : "unknown";
          }
        });
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "Al2|index=3|curr=(")?;
    Ok(())
}

#[test]
fn plain_formula_parser_accepts_parenthesized_groups() -> browser_tester::Result<()> {
    let html = r#"
    <div>
      <input id="formula" value="Al2(SO4)3" />
      <button id="go" type="button">go</button>
      <div id="out"></div>
    </div>
    <script>
    (() => {
      const weights = { Al: 26.982, S: 32.06, O: 15.999 };
      const input = document.getElementById("formula");
      const out = document.getElementById("out");

      function parserError(message) {
        return { message };
      }

      function multiplyCounts(map, factor) {
        const out = {};
        Object.keys(map).forEach((key) => {
          out[key] = map[key] * factor;
        });
        return out;
      }

      function mergeCounts(target, source) {
        Object.keys(source).forEach((key) => {
          target[key] = (target[key] || 0) + source[key];
        });
      }

      function createParser(source) {
        let index = 0;

        function current() {
          return source[index] || "";
        }

        function consume() {
          const char = source[index] || "";
          index += 1;
          return char;
        }

        function isDigit(char) {
          return /[0-9]/.test(char);
        }

        function isUpper(char) {
          return /[A-Z]/.test(char);
        }

        function isLower(char) {
          return /[a-z]/.test(char);
        }

        function parseNumber() {
          const start = index;
          let sawDigit = false;

          while (isDigit(current())) {
            sawDigit = true;
            consume();
          }

          const raw = source.slice(start, index);
          if (!sawDigit) {
            throw parserError("invalid number");
          }

          return { raw, value: Number(raw) };
        }

        function parseOptionalMultiplier() {
          if (isDigit(current())) return parseNumber();
          return { raw: "", value: 1 };
        }

        function parseElementSymbol() {
          const first = current();
          if (!isUpper(first)) {
            throw parserError("invalid symbol");
          }
          let symbol = consume();
          if (isLower(current())) symbol += consume();
          if (!weights[symbol]) {
            throw parserError("unknown element");
          }
          return symbol;
        }

        function parseBracketGroup() {
          const open = consume();
          const close = open === "(" ? ")" : "]";
          const inner = parseSequence(close, 1);
          if (current() !== close) {
            throw parserError("Bracket mismatch detected.");
          }
          consume();
          const multiplier = parseOptionalMultiplier();
          return {
            counts: multiplyCounts(inner.counts, multiplier.value),
            order: inner.order.slice(),
            normalized: open + inner.normalized + close + multiplier.raw
          };
        }

        function parseElementGroup() {
          const symbol = parseElementSymbol();
          const count = parseOptionalMultiplier();
          return {
            counts: { [symbol]: count.value },
            order: [symbol],
            normalized: symbol + count.raw
          };
        }

        function parseGroup(nesting) {
          const char = current();
          if (char === "(" || char === "[") {
            return parseBracketGroup();
          }
          return parseElementGroup();
        }

        function parseSequence(stopChar, nesting) {
          const counts = {};
          const order = [];
          let normalized = "";

          while (index < source.length && current() !== stopChar) {
            if (current() === ")" || current() === "]") {
              throw parserError("unexpected close");
            }
            const group = parseGroup(nesting);
            mergeCounts(counts, group.counts);
            group.order.forEach((item) => {
              if (!order.includes(item)) order.push(item);
            });
            normalized += group.normalized;
          }

          return { counts, order, normalized };
        }

        function parseFragment() {
          const body = parseSequence("", 1);
          if (index !== source.length) {
            throw parserError("invalid tail");
          }
          return {
            counts: multiplyCounts(body.counts, 1),
            order: body.order.slice(),
            normalized: body.normalized
          };
        }

        return { parseFragment };
      }

      document.getElementById("go").addEventListener("click", () => {
        try {
          const parsed = createParser(input.value).parseFragment();
          out.textContent = parsed.normalized + "|" + JSON.stringify(parsed.counts);
        } catch (error) {
          out.textContent = error && error.message ? error.message : "unknown";
        }
      });
    })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    let actual = harness.dump_dom("#out")?;
    assert!(
        actual.contains("Al2(SO4)3|")
            && actual.contains("\"Al\":2")
            && actual.contains("\"S\":3")
            && actual.contains("\"O\":12"),
        "expected parenthesized formula parser to succeed; actual: {actual}"
    );
    Ok(())
}

#[test]
fn foreach_attached_click_handler_reassigns_outer_state() -> browser_tester::Result<()> {
    let html = r#"
    <button id="reset-a" type="button">Reset A</button>
    <button id="reset-b" type="button">Reset B</button>
    <input id="status" value="" />
    <script>
      let state = { value: "1.2" };
      function createDefaultState() {
        return { value: "" };
      }
      function renderControls() {
        document.getElementById("status").value = state.value;
      }
      const els = {
        resetButtons: [
          document.getElementById("reset-a"),
          document.getElementById("reset-b")
        ]
      };
      renderControls();
      els.resetButtons.forEach((button) => button?.addEventListener("click", () => {
        state = createDefaultState();
        renderControls();
      }));
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#reset-a")?;
    harness.assert_value("#status", "")?;
    Ok(())
}

#[test]
fn foreach_attached_click_handler_reassigns_outer_state_in_iife_page_flow()
-> browser_tester::Result<()> {
    let html = r#"
    <button id="open" type="button">Open</button>
    <button id="reset" type="button">Reset</button>
    <input id="burn" value="" />
    <input id="price" value="" />
    <input id="idle" value="" />
    <p id="status"></p>
    <script>
      (() => {
        const messages = { reset: "Inputs reset." };
        const els = {
          openButton: document.getElementById("open"),
          resetButtons: [document.getElementById("reset")],
          burn: document.getElementById("burn"),
          price: document.getElementById("price"),
          idle: document.getElementById("idle"),
          status: document.getElementById("status"),
        };

        function createDefaultState() {
          return {
            presetKind: "small",
            fuelBurnValue: "20",
            priceValue: "",
            idleMinutes: "",
          };
        }

        let state = createDefaultState();

        function setStatus(text) {
          els.status.textContent = text || "";
        }

        function renderAll() {
          els.burn.value = state.fuelBurnValue;
          els.price.value = state.priceValue;
          els.idle.value = state.idleMinutes;
        }

        function attachEvents() {
          els.openButton.addEventListener("click", () => {
            state.priceValue = "155";
            state.idleMinutes = "300";
            renderAll();
          });
          els.resetButtons.forEach((button) => button?.addEventListener("click", () => {
            state = createDefaultState();
            renderAll();
            setStatus(messages.reset);
          }));
        }

        renderAll();
        attachEvents();
      })();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#open")?;
    harness.assert_value("#price", "155")?;
    harness.assert_value("#idle", "300")?;

    harness.click("#reset")?;
    harness.assert_value("#burn", "20")?;
    harness.assert_value("#price", "")?;
    harness.assert_value("#idle", "")?;
    harness.assert_text("#status", "Inputs reset.")?;
    Ok(())
}

#[test]
fn for_of_loop_supports_array_destructuring_binding() -> browser_tester::Result<()> {
    let html = r#"
    <div id="out"></div>
    <script>
      const entries = Object.entries({ mode: "mooring" });
      for (const [key, value] of entries) {
        document.getElementById("out").textContent = key + ":" + value;
      }
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "mode:mooring")?;
    Ok(())
}

#[test]
fn append_child_syncs_select_value_for_preselected_option() -> browser_tester::Result<()> {
    let html = r#"
    <select id="s"></select>
    <p id="out"></p>
    <script>
      const s = document.getElementById("s");
      ["g", "kg", "ml"].forEach((value) => {
        const option = document.createElement("option");
        option.value = value;
        option.textContent = value;
        if (value === "ml") {
          option.selected = true;
        }
        s.appendChild(option);
      });
      document.getElementById("out").textContent = "value:" + s.value;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "value:ml")?;
    harness.assert_value("#s", "ml")?;
    Ok(())
}
