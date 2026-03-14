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

        const numberFormat = new Intl.NumberFormat("en", {{ maximumFractionDigits: 0 }});

        function safeString(value) {{
          return String(value == null ? "" : value);
        }}

        function normalizeDigits(value) {{
          return safeString(value)
            .replace(/[\uFF10-\uFF19]/g, (s) => String.fromCharCode(s.charCodeAt(0) - 65248))
            .replace(/[\uFF0E\u3002]/g, ".")
            .replace(/[\uFF0C\u3001]/g, ",")
            .replace(/[\uFF0B]/g, "+")
            .replace(/[\u30FC\uFF0D\u2015]/g, "-")
            .trim();
        }}

        function parseFlexibleNumber(value) {{
          const normalized = normalizeDigits(value).replace(/[\s_\u00A0]/g, "");
          if (!normalized) return null;
          const sign = normalized.startsWith("-") ? -1 : 1;
          const unsigned = normalized.replace(/^[+-]/, "");
          if (!unsigned) return null;

          const commaCount = (unsigned.match(/,/g) || []).length;
          const dotCount = (unsigned.match(/\./g) || []).length;
          let candidate = unsigned;

          if (commaCount > 0 && dotCount > 0) {{
            const lastComma = unsigned.lastIndexOf(",");
            const lastDot = unsigned.lastIndexOf(".");
            const decimalIndex = Math.max(lastComma, lastDot);
            const intPart = unsigned.slice(0, decimalIndex).replace(/[.,]/g, "");
            const fracPart = unsigned.slice(decimalIndex + 1).replace(/[.,]/g, "");
            candidate = fracPart ? `${{intPart}}.${{fracPart}}` : intPart;
          }} else if (commaCount === 1) {{
            const parts = unsigned.split(",");
            candidate = parts[1].length === 3 ? parts.join("") : `${{parts[0]}}.${{parts[1]}}`;
          }} else if (dotCount === 1) {{
            const parts = unsigned.split(".");
            candidate = parts[1].length === 3 ? parts.join("") : `${{parts[0]}}.${{parts[1]}}`;
          }}

          const parsed = Number(candidate);
          if (!Number.isFinite(parsed)) return null;
          return parsed * sign;
        }}

        function parseDimensionToMm(value, fallbackUnit) {{
          const raw = normalizeDigits(value);
          if (!raw) return {{ mm: null, unit: fallbackUnit, error: null }};
          const compact = raw.replace(/[\s_\u00A0]/g, "");
          const match = compact.match(/^([+-]?[0-9.,]+)(mm|cm|m|in|inch|ft|["'])?$/i);
          if (!match) return {{ mm: null, unit: fallbackUnit, error: "format" }};
          const numeric = parseFlexibleNumber(match[1]);
          if (numeric == null) return {{ mm: null, unit: fallbackUnit, error: "format" }};
          const unit = (match[2] || fallbackUnit).toLowerCase();
          const factor =
            unit === "in" || unit === "inch" ? 25.4 :
            unit === "cm" ? 10 :
            unit === "m" ? 1000 :
            unit === "ft" ? 304.8 :
            1;
          const mm = numeric * factor;
          if (!Number.isFinite(mm) || mm <= 0) return {{ mm: null, unit, error: "positive" }};
          return {{ mm, unit, error: null }};
        }}

        function updateFieldHint(input, hintEl, parsed) {{
          if (!hintEl) return;
          if (!input.value) {{
            hintEl.textContent = "";
            hintEl.className = "hint";
            return;
          }}
          if (parsed.error) {{
            hintEl.textContent = "Invalid";
            hintEl.className = "hint error";
            return;
          }}
          hintEl.textContent = `${{numberFormat.format(Math.floor(parsed.mm))}} mm`;
          hintEl.className = "hint";
        }}

        function collectPersistedState() {{
          return {{
            cargo: [
              safeString(Math.round(parseDimensionToMm(el.cargoW.value, state.displayUnit).mm || 0)),
              safeString(Math.round(parseDimensionToMm(el.cargoD.value, state.displayUnit).mm || 0)),
              safeString(Math.round(parseDimensionToMm(el.cargoH.value, state.displayUnit).mm || 0))
            ].join(","),
            box: [
              safeString(Math.round(parseDimensionToMm(el.boxW.value, state.displayUnit).mm || 0)),
              safeString(Math.round(parseDimensionToMm(el.boxD.value, state.displayUnit).mm || 0)),
              safeString(Math.round(parseDimensionToMm(el.boxH.value, state.displayUnit).mm || 0))
            ].join(","),
            stack: normalizeDigits(el.stack.value),
            margin: normalizeDigits(el.margin.value)
          }};
        }}

        function syncUrl() {{
          if (!state.querySyncEnabled) return;
          const persisted = collectPersistedState();
          const params = new URLSearchParams();
          Object.entries(persisted).forEach(([key, value]) => {{
            if (value == null || value === "" || value === "0,0,0") return;
            params.set(key, String(value));
          }});
          const next = params.toString()
            ? `${{window.location.pathname}}?${{params.toString()}}`
            : window.location.pathname;
          window.history.replaceState(null, "", next);
        }}

        function saveLastState() {{
          if (!state.restoreLastState) return;
          window.localStorage.setItem(
            "tool.fishery.boxLoading.lastState.v1",
            JSON.stringify(collectPersistedState())
          );
        }}

        function recalc() {{
          const cargoW = parseDimensionToMm(el.cargoW.value, state.displayUnit);
          const cargoD = parseDimensionToMm(el.cargoD.value, state.displayUnit);
          const cargoH = parseDimensionToMm(el.cargoH.value, state.displayUnit);
          const boxW = parseDimensionToMm(el.boxW.value, state.displayUnit);
          const boxD = parseDimensionToMm(el.boxD.value, state.displayUnit);
          const boxH = parseDimensionToMm(el.boxH.value, state.displayUnit);

          updateFieldHint(el.cargoW, el.cargoWHint, cargoW);
          updateFieldHint(el.cargoD, el.cargoDHint, cargoD);
          updateFieldHint(el.cargoH, el.cargoHHint, cargoH);
          updateFieldHint(el.boxW, el.boxWHint, boxW);
          updateFieldHint(el.boxD, el.boxDHint, boxD);
          updateFieldHint(el.boxH, el.boxHHint, boxH);

          const allDims = [cargoW, cargoD, cargoH, boxW, boxD, boxH];
          if (allDims.every((item) => Number.isFinite(item.mm))) {{
            const stackLimit = Number(normalizeDigits(el.stack.value) || "1");
            const total = stackLimit > 1 ? "6" : "3";
            el.total.textContent = total;
            el.summary.textContent = [
              el.cargoWHint.textContent,
              el.cargoDHint.textContent,
              el.boxWHint.textContent,
              el.boxHHint.textContent
            ].join("|");
          }} else {{
            el.total.textContent = "";
            el.summary.textContent = "";
          }}

          syncUrl();
          saveLastState();
          el.status.textContent = window.location.search;
        }}

        [
          el.cargoW,
          el.cargoD,
          el.cargoH,
          el.boxW,
          el.boxD,
          el.boxH,
          el.stack,
          el.margin
        ].forEach((input) => {{
          input.addEventListener("input", recalc);
          input.addEventListener("blur", recalc);
        }});
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
        summary.contains("1,198 mm")
            && summary.contains("899 mm")
            && summary.contains("599 mm")
            && summary.contains("299 mm"),
        "expected rendered hints to survive repeated type_text flow, got: {summary}"
    );
    let status = harness.dump_dom("#status")?;
    assert!(
        status.contains(
            "?cargo=1199%2C899%2C800&amp;box=599%2C399%2C300&amp;stack=2&amp;margin=3%2C0"
        ),
        "expected URL sync to complete during repeated typing, got: {status}"
    );
    Ok(())
}
