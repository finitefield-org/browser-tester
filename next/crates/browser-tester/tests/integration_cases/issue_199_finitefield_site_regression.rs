use browser_tester::Harness;
use std::fmt::Write as _;

#[test]
fn repeated_type_text_with_large_id_heavy_dom_and_history_storage_sync_completes()
-> browser_tester::Result<()> {
    let mut filler = String::new();
    for index in 0..1800 {
        let _ = write!(
            filler,
            "<div id=\"seed-{index}\"><span id=\"seed-text-{index}\">seed</span></div>"
        );
    }

    let html = format!(
        r#"
      <div id="fixture">{filler}</div>
      <input id="cargo-w" type="text" inputmode="decimal" />
      <span id="cargo-w-hint"></span>
      <input id="cargo-d" type="text" inputmode="decimal" />
      <span id="cargo-d-hint"></span>
      <input id="cargo-h" type="text" inputmode="decimal" />
      <span id="cargo-h-hint"></span>
      <input id="box-w" type="text" inputmode="decimal" />
      <span id="box-w-hint"></span>
      <input id="box-d" type="text" inputmode="decimal" />
      <span id="box-d-hint"></span>
      <input id="box-h" type="text" inputmode="decimal" />
      <span id="box-h-hint"></span>
      <input id="stack" type="text" inputmode="numeric" />
      <input id="margin" type="text" inputmode="decimal" />
      <p id="total"></p>
      <p id="summary"></p>
      <p id="status"></p>
      <script>
        const state = {{
          displayUnit: "mm",
          restoreLastState: true,
          querySyncEnabled: true
        }};

        const el = {{
          cargoW: document.getElementById("cargo-w"),
          cargoD: document.getElementById("cargo-d"),
          cargoH: document.getElementById("cargo-h"),
          boxW: document.getElementById("box-w"),
          boxD: document.getElementById("box-d"),
          boxH: document.getElementById("box-h"),
          stack: document.getElementById("stack"),
          margin: document.getElementById("margin"),
          cargoWHint: document.getElementById("cargo-w-hint"),
          cargoDHint: document.getElementById("cargo-d-hint"),
          cargoHHint: document.getElementById("cargo-h-hint"),
          boxWHint: document.getElementById("box-w-hint"),
          boxDHint: document.getElementById("box-d-hint"),
          boxHHint: document.getElementById("box-h-hint"),
          total: document.getElementById("total"),
          summary: document.getElementById("summary"),
          status: document.getElementById("status")
        }};

        function joinValues(values) {{
          let out = "";
          for (let index = 0; index < values.length; index += 1) {{
            if (index > 0) {{
              out += ",";
            }}
            out += values[index];
          }}
          return out;
        }}

        function sync() {{
          const cargoW = el.cargoW.value;
          const cargoD = el.cargoD.value;
          const cargoH = el.cargoH.value;
          const boxW = el.boxW.value;
          const boxD = el.boxD.value;
          const boxH = el.boxH.value;
          const stack = el.stack.value;
          const margin = el.margin.value;

          let total = 0;
          if (cargoW) total += 1;
          if (cargoD) total += 1;
          if (cargoH) total += 1;
          if (boxW) total += 1;
          if (boxD) total += 1;
          if (boxH) total += 1;

          el.total.textContent = String(total);
          el.summary.textContent = joinValues([cargoW, cargoD, cargoH, boxW, boxD, boxH]);

          let query = "";

          function appendParam(name, value) {{
            if (!value) return;
            if (query) {{
              query += "&";
            }} else {{
              query += "?";
            }}
            query += name + "=" + value.replace(/,/g, "%2C");
          }}

          appendParam("cargoW", cargoW);
          appendParam("cargoD", cargoD);
          appendParam("cargoH", cargoH);
          appendParam("boxW", boxW);
          appendParam("boxD", boxD);
          appendParam("boxH", boxH);
          appendParam("stack", stack);
          appendParam("margin", margin);

          const next = query ? window.location.pathname + query : window.location.pathname;
          window.history.replaceState(null, "", next);
          window.localStorage.setItem(
            "tool.fishery.boxLoading.lastState.v1",
            joinValues([cargoW, cargoD, cargoH, boxW, boxD, boxH, stack, margin])
          );
          el.status.textContent = next;
        }}

        function bindInputs() {{
          const inputs = [el.cargoW, el.cargoD, el.cargoH, el.boxW, el.boxD, el.boxH, el.stack, el.margin];
          for (let index = 0; index < inputs.length; index += 1) {{
            inputs[index].addEventListener("input", sync);
            inputs[index].addEventListener("blur", sync);
          }}
        }}

        bindInputs();
        sync();
      </script>
    "#
    );

    let mut harness = Harness::from_html_with_url(
        "https://example.com/tools/fishery/box-loading-calculator/",
        &html,
    )?;
    harness.type_text("#cargo-w", "47.2in")?;
    harness.type_text("#cargo-d", "35.4in")?;
    harness.type_text("#cargo-h", "31,5in")?;
    harness.type_text("#box-w", "23.6in")?;
    harness.type_text("#box-d", "15.7in")?;
    harness.type_text("#box-h", "11,8in")?;
    harness.type_text("#stack", "2")?;
    harness.type_text("#margin", "3,0")?;

    harness.assert_text("#total", "6")?;
    let summary = harness.dump_dom("#summary")?;
    assert!(
        summary.contains("47.2in")
            && summary.contains("35.4in")
            && summary.contains("31,5in")
            && summary.contains("23.6in"),
        "expected typed values to survive repeated type_text flow, got: {summary}"
    );
    let status = harness.dump_dom("#status")?;
    assert!(
        status.contains("?cargoW=47.2in")
            && status.contains("&amp;stack=2")
            && status.contains("&amp;margin=3%2C0"),
        "expected URL sync to complete during repeated typing, got: {status}"
    );
    Ok(())
}
