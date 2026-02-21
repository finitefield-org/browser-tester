use browser_tester::Harness;

#[test]
fn ignores_json_ld_script_blocks_and_runs_executable_script() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result">init</div>
    <script type="application/ld+json">
      {"@context":"https://schema.org","@type":"FAQPage"}
    </script>
    <script>
      document.getElementById("result").textContent = "ok";
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn json_ld_with_escaped_quotes_does_not_break_script_end_detection() -> browser_tester::Result<()> {
    let html = r#"<!doctype html><html><body><div id="result">init</div><script type=\"application/ld+json\">{\"@context\":\"https://schema.org\"}</script><script>document.getElementById("result").textContent = "ok";</script></body></html>"#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn script_end_extractor_handles_regex_literals_with_quotes() -> browser_tester::Result<()> {
    let html = r##"
    <div id="result">init</div>
    <script>
      const sanitizer = /["]/g;
      document.getElementById("result").textContent = "ok";
    </script>
    "##;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn array_from_supports_nodelist_and_map_callback() -> browser_tester::Result<()> {
    let html = r#"
    <ul>
      <li class="item">A</li>
      <li class="item">B</li>
    </ul>
    <div id="result"></div>
    <script>
      const nodes = Array.from(document.querySelectorAll(".item"));
      const mapped = Array.from(nodes, (node, idx) => node.textContent + idx);
      document.getElementById("result").textContent = nodes.length + ":" + mapped.join(",");
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "2:A0,B1")?;
    Ok(())
}

#[test]
fn trailing_commas_in_literals_are_supported_without_allowing_sparse_entries() {
    let html = r#"
    <div id="result"></div>
    <script>
      const base = { a: 1, b: 2, };
      const merged = { ...base, c: 3, };
      const values = [merged.a, merged.b, merged.c,];
      document.getElementById("result").textContent = values.join("-");
    </script>
    "#;

    let harness = Harness::from_html(html).expect("trailing commas should be accepted");
    harness
        .assert_text("#result", "1-2-3")
        .expect("result text should match");

    let sparse = Harness::from_html("<script>const bad = [1,,2];</script>")
        .expect_err("sparse arrays should remain unsupported");
    match sparse {
        browser_tester::Error::ScriptParse(msg) => {
            assert!(msg.contains("array literal does not support empty elements"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn nested_object_path_access_on_runtime_objects_is_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const page = {
        input: {
          inventory: {
            defaultLotPrefix: "Lot"
          }
        }
      };
      document.getElementById("result").textContent = page.input.inventory.defaultLotPrefix;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "Lot")?;
    Ok(())
}

#[test]
fn function_declaration_can_be_called_before_its_definition() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const state = {
        lots: [createLotRow()],
      };

      function createLotRow(seed = {}) {
        return {
          name: seed.name || "lot-1",
        };
      }

      document.getElementById("result").textContent = String(state.lots.length);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn create_lot_row_seed_name_property_uses_object_semantics() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      function createLotRow(seed = {}) {
        return seed.name;
      }

      const fromDefault = createLotRow();
      const fromArray = createLotRow([]);
      document.getElementById("result").textContent =
        String(fromDefault === undefined) + "|" + String(fromArray === undefined);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "true|true")?;
    Ok(())
}

#[test]
fn function_reassignment_of_global_is_visible_across_functions() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      let computedAllCandidates = [];

      function buildCandidates() {
        computedAllCandidates = [{ id: "one" }];
      }

      function candidateCount() {
        return computedAllCandidates.length;
      }

      buildCandidates();
      document.getElementById("result").textContent = String(candidateCount());
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn function_can_call_global_function_declared_later() -> browser_tester::Result<()> {
    let html = r#"
    <button id="btn">run</button>
    <div id="result">init</div>
    <script>
      function openDialog() {
        closePasteDialog();
      }

      function closePasteDialog() {
        document.getElementById("result").textContent = "closed";
      }

      document.getElementById("btn").addEventListener("click", openDialog);
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#btn")?;
    harness.assert_text("#result", "closed")?;
    Ok(())
}

#[test]
fn a_then_b_reads_updated_global_binding() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      let x = 0;
      function a() {
        x = 1;
      }
      function b() {
        return x;
      }
      a();
      document.getElementById("result").textContent = String(b());
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn function_a_update_is_visible_to_function_b_in_same_event() -> browser_tester::Result<()> {
    let html = r#"
    <button id="btn">run</button>
    <div id="result"></div>
    <script>
      let shared = 0;
      function a() {
        shared = 7;
      }
      function b() {
        return shared;
      }
      document.getElementById("btn").addEventListener("click", () => {
        a();
        document.getElementById("result").textContent = String(b());
      });
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#btn")?;
    harness.assert_text("#result", "7")?;
    Ok(())
}

#[test]
fn bind_function_listener_closure_can_call_local_close() -> browser_tester::Result<()> {
    let html = r#"
    <button id="btn">run</button>
    <div id="result">init</div>
    <script>
      const btn = document.getElementById("btn");
      function bind() {
        const close = () => {
          document.getElementById("result").textContent = "closed";
        };
        btn.addEventListener("click", () => close());
      }
      bind();
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#btn")?;
    harness.assert_text("#result", "closed")?;
    Ok(())
}

#[test]
fn foreach_map_reduce_sort_callbacks_reflect_outer_updates() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const values = [3, 1, 2];
      const stats = { forEachCount: 0, mapSum: 0, reduceSum: 0, sortCalls: 0 };

      const mapped = values.map((v) => {
        stats.mapSum += v;
        return v * 2;
      });
      values.forEach(() => {
        stats.forEachCount += 1;
      });
      values.reduce((acc, v) => {
        stats.reduceSum += v;
        return acc + v;
      }, 0);
      values.sort((a, b) => {
        stats.sortCalls += 1;
        return a - b;
      });

      document.getElementById("result").textContent =
        String(stats.forEachCount) + "|" +
        String(stats.mapSum) + "|" +
        String(stats.reduceSum) + "|" +
        String(stats.sortCalls > 0) + "|" +
        values.join(",") + "|" +
        mapped.join(",");
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "3|6|6|true|1,2,3|6,2,4")?;
    Ok(())
}

#[test]
fn run_calculation_pipeline_keeps_candidates_for_render_outputs() -> browser_tester::Result<()> {
    let html = r#"
    <button id="run">run</button>
    <div id="result"></div>
    <script>
      let computedAllCandidates = [];
      const state = { selectedCandidate: 0 };

      function buildCandidates(requiredQty, unitsPerCase) {
        if (requiredQty === 125 && unitsPerCase === 24) {
          return [{ caseTotal: 4, eachTotal: 29, totalUnits: 125 }];
        }
        return [];
      }

      function renderOutputs() {
        if (!computedAllCandidates.length) {
          document.getElementById("result").textContent = "No candidate";
          return;
        }
        const candidate = computedAllCandidates[state.selectedCandidate];
        document.getElementById("result").textContent =
          String(candidate.caseTotal) + ":" + String(candidate.eachTotal) + ":" + String(candidate.totalUnits);
      }

      function runCalculation() {
        computedAllCandidates = buildCandidates(125, 24);
        state.selectedCandidate = 0;
        renderOutputs();
      }

      document.getElementById("run").addEventListener("click", runCalculation);
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#result", "4:29:125")?;
    Ok(())
}

#[test]
fn window_url_static_properties_are_object_like_and_assignable() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const beforeType = typeof window.URL.createObjectURL;
      window.URL.createObjectURL = function() { return "patched"; };
      const afterType = typeof window.URL.createObjectURL;
      const called = window.URL.createObjectURL(new Blob(["x"]));
      document.getElementById("result").textContent = beforeType + "|" + afterType + "|" + called;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "function|function|patched")?;
    Ok(())
}

#[test]
fn local_storage_basic_methods_are_available() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      localStorage.removeItem("x");
      const initialMissing = localStorage.getItem("x") === null;
      localStorage.setItem("x", "42");
      const fromMethod = localStorage.getItem("x");
      const fromIndex = localStorage["x"];
      const lenAfterOne = localStorage.length;
      localStorage.setItem("y", "99");
      const firstKey = localStorage.key(0);
      localStorage.removeItem("x");
      const missingAgain = localStorage.getItem("x") === null;
      localStorage.clear();
      const lenAfterClear = localStorage.length;
      document.getElementById("result").textContent =
        String(initialMissing) + "|" +
        fromMethod + "|" +
        fromIndex + "|" +
        String(lenAfterOne) + "|" +
        firstKey + "|" +
        String(missingAgain) + "|" +
        String(lenAfterClear);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "true|42|42|1|x|true|0")?;
    Ok(())
}

#[test]
fn window_local_storage_is_assignable_for_stub_usage() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      window.localStorage = {
        getItem: (key) => key === "token" ? "stubbed" : null
      };
      document.getElementById("result").textContent =
        String(localStorage.getItem("token")) + "|" + String(window.localStorage.getItem("token"));
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "stubbed|stubbed")?;
    Ok(())
}

#[test]
fn from_html_with_local_storage_seeds_values_before_script_execution() -> browser_tester::Result<()>
{
    let html = r#"
    <div id="result"></div>
    <script>
      document.getElementById("result").textContent =
        String(localStorage.getItem("token")) + "|" + String(localStorage.getItem("mode"));
    </script>
    "#;

    let harness =
        Harness::from_html_with_local_storage(html, &[("token", "seed"), ("mode", "debug")])?;
    harness.assert_text("#result", "seed|debug")?;
    Ok(())
}

#[test]
fn document_member_calls_with_dynamic_arguments_are_supported() -> browser_tester::Result<()> {
    let html = r#"
    <input name="mode" value="piece" checked>
    <div id="result"></div>
    <script>
      const targetId = "result";
      const root = document.getElementById(targetId);
      const field = "mode";
      const input = document.querySelector(`input[name="${field}"]:checked`);
      root.textContent = root.id + "|" + input.value;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "result|piece")?;
    Ok(())
}

#[test]
fn get_attribute_returns_null_for_missing_attribute_in_delegated_click_handler()
-> browser_tester::Result<()> {
    let html = r#"
    <div id="root">
      <button id="cta-sample">CTA</button>
      <button id="row-del" data-order-delete="1">DEL</button>
    </div>
    <div id="result"></div>
    <script>
      const root = document.getElementById("root");
      root.addEventListener("click", (event) => {
        const button = event.target.closest("button");
        if (!button) return;
        const orderDelete = button.getAttribute("data-order-delete");
        if (orderDelete !== null) {
          document.getElementById("result").textContent = "delete";
          return;
        }
        if (button.id === "cta-sample") {
          document.getElementById("result").textContent = "cta";
        }
      });
      document.getElementById("cta-sample").click();
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "cta")?;
    Ok(())
}

#[test]
fn async_click_handler_observes_updated_let_capture_for_clipboard() -> browser_tester::Result<()> {
    let html = r#"
    <button id="b">copy</button>
    <script>
      let last = null;
      function render() {
        last = { ok: true, text: "hello" };
      }
      render();
      document.getElementById("b").addEventListener("click", async () => {
        if (!last || !last.ok) return;
        await navigator.clipboard.writeText(last.text);
      });
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#b")?;
    assert_eq!(harness.clipboard_text(), "hello");
    Ok(())
}

#[test]
fn dom_expando_properties_round_trip_on_nodes() -> browser_tester::Result<()> {
    let html = r#"
    <div id="root"></div>
    <div id="result"></div>
    <script>
      const root = document.getElementById("root");
      root.__state = { score: 19 };
      document.getElementById("result").textContent = String(root.__state.score);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "19")?;
    Ok(())
}

#[test]
fn regex_lookahead_in_replace_parses_and_runs() -> browser_tester::Result<()> {
    let html = r#"
    <p id="result"></p>
    <script>
      const format = (v) => String(v).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
      document.getElementById("result").textContent = format(810000);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "810,000")?;
    Ok(())
}

#[test]
fn utf8_script_assigned_text_is_preserved() -> browser_tester::Result<()> {
    let html = r#"
    <p id="result"></p>
    <script>
      document.getElementById("result").textContent = "A 〜 B";
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "A 〜 B")?;
    Ok(())
}
