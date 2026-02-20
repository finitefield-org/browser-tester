use browser_tester::Harness;

#[test]
fn regex_charclass_quote_does_not_break_balancer() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      "<a&b>".replace(/[&<>"']/g, (ch) => ch);
      document.getElementById("result").textContent = "ok";
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn unary_not_regex_test_condition_parses() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const value = "abc";
      if (!/^[0-9]+$/.test(value)) {
        document.getElementById("result").textContent = "non-numeric";
      } else {
        document.getElementById("result").textContent = "numeric";
      }
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "non-numeric")?;
    Ok(())
}

#[test]
fn window_unknown_property_falls_back_to_undefined() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const value = window.lucide;
      document.getElementById("result").textContent = String(value === undefined);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn document_element_lang_is_readable() -> browser_tester::Result<()> {
    let html = r#"
    <html lang="ja">
      <body>
        <div id="result"></div>
        <script>
          document.getElementById("result").textContent = document.documentElement.lang;
        </script>
      </body>
    </html>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ja")?;
    Ok(())
}

#[test]
fn object_literal_shorthand_and_spread_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const seed = { count: 1 };
      const name = "lot";
      const merged = { ...seed, name };
      document.getElementById("result").textContent = merged.count + ":" + merged.name;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "1:lot")?;
    Ok(())
}

#[test]
fn object_property_compound_assignment_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const obj = { count: 1 };
      obj.count += 1;
      document.getElementById("result").textContent = String(obj.count);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "2")?;
    Ok(())
}

#[test]
fn nested_object_property_assignment_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const state = { settings: { allowOpenCase: false } };
      state.settings.allowOpenCase = true;
      document.getElementById("result").textContent = String(state.settings.allowOpenCase);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn nested_object_property_compound_assignment_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const state = { settings: { count: 1 } };
      state.settings.count += 2;
      document.getElementById("result").textContent = String(state.settings.count);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn logical_or_returns_operand_value_for_object_defaulting() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      function createLotRow(seed) {
        seed = seed || {};
        return seed.name;
      }
      document.getElementById("result").textContent = String(createLotRow() === undefined);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn logical_and_returns_last_truthy_operand_value() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const value = "x" && 7;
      document.getElementById("result").textContent = String(value);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "7")?;
    Ok(())
}

#[test]
fn array_destructured_callback_params_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const obj = { a: "x", b: "y" };
      const result = document.getElementById("result");
      Object.entries(obj).forEach(([k, v]) => {
        result.textContent = result.textContent + k + v;
      });
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "axby")?;
    Ok(())
}

#[test]
fn nodelist_foreach_member_call_is_supported() -> browser_tester::Result<()> {
    let html = r#"
    <ul>
      <li class="item">A</li>
      <li class="item">B</li>
    </ul>
    <div id="result"></div>
    <script>
      let out = "";
      document.querySelectorAll(".item").forEach((node, index) => {
        out += node.textContent + String(index);
      });
      document.getElementById("result").textContent = out;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "A0B1")?;
    Ok(())
}

#[test]
fn array_member_calls_via_object_path_are_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const state = { lots: [{ qty: 2 }, { qty: 3 }] };
      const total = state.lots
        .map((lot) => lot.qty)
        .reduce((sum, value) => sum + value, 0);
      state.lots.push({ qty: 4 });
      const filtered = state.lots.filter((lot) => lot.qty >= 3);
      document.getElementById("result").textContent =
        String(total) + "|" + String(filtered.length);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "5|2")?;
    Ok(())
}

#[test]
fn array_filter_boolean_is_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const values = [0, 1, "", 2, null, 3, false];
      const filtered = values.filter(Boolean);
      document.getElementById("result").textContent = filtered.join(",");
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "1,2,3")?;
    Ok(())
}

#[test]
fn chained_filter_boolean_is_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const text = "a\n\n b \n";
      const lines = text
        .split(/\r?\n/)
        .map((line) => line.trim())
        .filter(Boolean);
      document.getElementById("result").textContent = lines.join("|");
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "a|b")?;
    Ok(())
}

#[test]
fn add_event_listener_accepts_named_function_reference_callback() -> browser_tester::Result<()> {
    let html = r#"
    <button id="open-tool">open</button>
    <div id="result"></div>
    <script>
      const el = {
        openToolBtn: document.getElementById("open-tool"),
      };
      function openDialog() {
        document.getElementById("result").textContent = "opened";
      }
      el.openToolBtn.addEventListener("click", openDialog);
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#open-tool")?;
    harness.assert_text("#result", "opened")?;
    Ok(())
}

#[test]
fn object_entries_foreach_updates_outer_variable() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      function interpolate(template, params = {}) {
        let out = String(template || "");
        Object.entries(params).forEach(([key, value]) => {
          out = out.replaceAll(`{${key}}`, String(value));
        });
        return out;
      }
      document.getElementById("result").textContent = interpolate("Shortage {value}", { value: 7 });
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "Shortage 7")?;
    Ok(())
}

#[test]
fn expression_foreach_listener_keeps_item_variable_scope() -> browser_tester::Result<()> {
    let html = r#"
    <button id="a">A</button>
    <button id="b">B</button>
    <div id="result"></div>
    <script>
      document.querySelectorAll("button").forEach((btn, idx) => {
        btn.addEventListener("click", () => {
          document.getElementById("result").textContent = btn.id + ":" + idx;
        });
      });
    </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#b")?;
    harness.assert_text("#result", "b:1")?;
    Ok(())
}

#[test]
fn real_world_picking_breakdown_smoke() -> browser_tester::Result<()> {
    let html = include_str!("fixtures/picking-breakdown-minimal.html");
    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ja|true|true|2")?;
    Ok(())
}

#[test]
fn array_push_trailing_comma_is_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      const lines = [];
      lines.push("x",);
      document.getElementById("r").textContent = lines.join(",");
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "x")?;
    Ok(())
}

#[test]
fn nested_call_with_trailing_commas_is_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      function interpolate(v, obj) { return v + ":" + obj.n; }
      const lines = [];
      lines.push(interpolate("total", { n: 1, },),);
      document.getElementById("r").textContent = lines[0];
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "total:1")?;
    Ok(())
}

#[test]
fn empty_call_argument_is_still_rejected() {
    let err = Harness::from_html("<script>const a=[]; a.push(1,,2);</script>").unwrap_err();
    match err {
        browser_tester::Error::ScriptParse(msg) => assert!(msg.contains("empty")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn lone_comma_argument_is_rejected() {
    let err = Harness::from_html("<script>function f(x){} f(,);</script>").unwrap_err();
    match err {
        browser_tester::Error::ScriptParse(msg) => assert!(msg.contains("empty")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn function_call_spread_arguments_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      function sum(x, y, z) { return x + y + z; }
      const numbers = [1, 2, 3];
      document.getElementById("r").textContent = String(sum(...numbers));
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "6")?;
    Ok(())
}

#[test]
fn mixed_call_spread_arguments_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      function myFunction(v, w, x, y, z) { return v + "," + w + "," + x + "," + y + "," + z; }
      const args = [0, 1];
      document.getElementById("r").textContent = myFunction(-1, ...args, 2, ...[3]);
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "-1,0,1,2,3")?;
    Ok(())
}

#[test]
fn array_push_spread_arguments_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      const lines = ["a"];
      const next = ["b", "c"];
      lines.push(...next,);
      document.getElementById("r").textContent = lines.join(",");
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "a,b,c")?;
    Ok(())
}

#[test]
fn object_spread_primitives_follow_js_behavior() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      const obj = { ...true, ..."test", ...10 };
      document.getElementById("r").textContent =
        obj["0"] + obj["1"] + obj["2"] + obj["3"] + "|" + String(obj["4"] === undefined);
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "test|true")?;
    Ok(())
}

#[test]
fn non_iterable_spread_in_call_is_rejected() {
    let err = Harness::from_html("<script>function f(a){} const obj={a:1}; f(...obj);</script>")
        .unwrap_err();
    match err {
        browser_tester::Error::ScriptRuntime(msg) => assert!(msg.contains("iterable")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn non_iterable_spread_in_array_literal_is_rejected() {
    let err =
        Harness::from_html("<script>const obj={a:1}; const arr=[...obj];</script>").unwrap_err();
    match err {
        browser_tester::Error::ScriptRuntime(msg) => assert!(msg.contains("iterable")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn division_by_zero_follows_js_infinity_and_nan_semantics() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const pos = 1 / 0;
      const neg = -1 / 0;
      const nan = 0 / 0;
      document.getElementById("result").textContent =
        String(pos) + "|" + String(neg) + "|" + String(nan) + "|" + String(pos > 0);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "Infinity|-Infinity|NaN|true")?;
    Ok(())
}

#[test]
fn array_from_map_values_is_usable_with_array_map() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const state = { totalsMap: new Map() };
      state.totalsMap.set("a", { n: 1 });
      state.totalsMap.set("b", { n: 2 });

      const totals = Array.from(state.totalsMap.values());
      const rendered = totals.map((row) => `${row.n}`).join(",");
      document.getElementById("result").textContent = rendered;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "1,2")?;
    Ok(())
}

#[test]
fn parenthesized_arrow_parameter_list_in_const_assignment_parses() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const toCSV = (rows) => rows.map((r) => r).join(",");
      document.getElementById("result").textContent = toCSV(["a", "b"]);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "a,b")?;
    Ok(())
}

#[test]
fn parenthesized_arrow_parameter_list_with_nested_callbacks_and_template_literal_parses()
-> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const toCSV = (rows) => rows
        .map((row) => row.map((cell) => {
          const s = String(cell ?? "");
          if (/[",\n]/.test(s)) return `"${s.replace(/"/g, '""')}"`;
          return s;
        }).join(","))
        .join("\n");
      document.getElementById("result").textContent = toCSV([["a", "x,y"], ["b", "z"]]);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "a,\"x,y\"\nb,z")?;
    Ok(())
}

#[test]
fn regex_char_class_with_escaped_newline_in_test_call_parses() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const s = "x,y";
      document.getElementById("result").textContent = String(/[",\n]/.test(s));
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn template_literal_expression_with_regex_replace_parses() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const s = "x,y";
      document.getElementById("result").textContent = `"${s.replace(/"/g, '""')}"`;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "\"x,y\"")?;
    Ok(())
}

#[test]
fn parenthesized_arrow_parameter_list_with_multiline_replace_chain_parses()
-> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const encodeSharePayload = (payload) => btoa(unescape(encodeURIComponent(JSON.stringify(payload))))
        .replace(/\+/g, "-")
        .replace(/\//g, "_")
        .replace(/=+$/g, "");
      document.getElementById("result").textContent = encodeSharePayload({ a: 1 });
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "eyJhIjoxfQ")?;
    Ok(())
}

#[test]
fn regex_with_escaped_slash_parses_in_replace_call() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      document.getElementById("result").textContent = "a/b".replace(/\//g, "_");
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "a_b")?;
    Ok(())
}

#[test]
fn regex_with_end_anchor_quantifier_parses_in_replace_call() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      document.getElementById("result").textContent = "abc==".replace(/=+$/g, "");
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "abc")?;
    Ok(())
}

#[test]
fn regex_lookahead_in_ternary_condition_parses() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const input = "810000";
      const out = /\B(?=(\d{3})+(?!\d))/.test(input)
        ? input.replace(/\B(?=(\d{3})+(?!\d))/g, ",")
        : "ng";
      document.getElementById("result").textContent = out;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "810,000")?;
    Ok(())
}

#[test]
fn object_literal_colon_detection_ignores_regex_lookahead() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const model = {
        formatter: /\B(?=(\d{3})+(?!\d))/g,
        formatted: "810000".replace(/\B(?=(\d{3})+(?!\d))/g, ",")
      };
      document.getElementById("result").textContent = model.formatted;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "810,000")?;
    Ok(())
}

#[test]
fn nullish_coalescing_inside_ternary_condition_parses() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const maybe = null;
      const out = (maybe ?? "x") ? "ok" : "ng";
      document.getElementById("result").textContent = out;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn for_and_while_allow_single_statement_bodies() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const values = [];
      for (let i = 0; i < 3; i += 1) values.push(i);

      let j = 0;
      while (j < 3) j += 1;

      document.getElementById("result").textContent = values.join(",") + "|" + String(j);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "0,1,2|3")?;
    Ok(())
}

#[test]
fn ternary_with_object_literal_branches_parses() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const page = {
        units: {
          areaUs: "acre",
          rateUs: "usd",
          areaMetric: "ha",
          rateMetric: "eur",
        }
      };
      const snapshot = { unitSystem: "us" };
      const labels = snapshot.unitSystem === "us"
        ? { area: page.units.areaUs, rate: page.units.rateUs }
        : { area: page.units.areaMetric, rate: page.units.rateMetric };
      document.getElementById("result").textContent = labels.area + "|" + labels.rate;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "acre|usd")?;
    Ok(())
}

#[test]
fn nested_parenthesized_division_in_multiplication_parses() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const row = { valueNum: 2 };
      const batchL = 100;
      const x = batchL * (row.valueNum / 100);
      document.getElementById("result").textContent = String(x);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "2")?;
    Ok(())
}
