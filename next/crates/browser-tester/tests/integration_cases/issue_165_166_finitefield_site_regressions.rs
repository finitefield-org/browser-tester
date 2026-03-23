use browser_tester::Harness;

#[test]
fn issue_165_double_quoted_newline_separator_in_join_is_supported() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        document.getElementById("out").textContent = ["a", "b"].join("\n");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "a\nb")?;
    Ok(())
}

#[test]
fn issue_165_build_csv_keeps_row_breaks_before_download() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function csvLine(values) {
          return values.map((value) => {
            const text = String(value === undefined || value === null ? "" : value);
            if (/[",\n]/.test(text)) return `"${text.replace(/"/g, "\"\"")}"`;
            return text;
          }).join(",");
        }

        function buildCsv() {
          const lines = [
            ["field_name", "field_group"],
            ["Field 1", "North Block"],
            ["Field 2", "South Block"]
          ];
          return lines.map(csvLine).join("\n");
        }

        document.getElementById("out").textContent = buildCsv();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "field_name,field_group\nField 1,North Block\nField 2,South Block",
    )?;
    Ok(())
}

#[test]
fn issue_165_chained_map_join_keeps_explicit_separator() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        document.getElementById("out").textContent = ["a", "b"].map((value) => value).join("\n");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "a\nb")?;
    Ok(())
}

#[test]
fn issue_165_join_after_storing_map_result_keeps_explicit_separator() -> browser_tester::Result<()>
{
    let html = r#"
      <div id="out"></div>
      <script>
        const mapped = ["a", "b"].map((value) => value);
        document.getElementById("out").textContent = mapped.join("\n");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "a\nb")?;
    Ok(())
}

#[test]
fn issue_165_named_map_callback_followed_by_join_keeps_explicit_separator()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function identity(value) {
          return value;
        }

        function build() {
          return ["a", "b"].map(identity).join("\n");
        }

        document.getElementById("out").textContent = build();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "a\nb")?;
    Ok(())
}

#[test]
fn issue_165_nested_array_rows_map_to_strings_then_join_with_newlines() -> browser_tester::Result<()>
{
    let html = r#"
      <div id="out"></div>
      <script>
        function rowToLine(row) {
          return row.join(",");
        }

        function build() {
          const rows = [
            ["a", "b"],
            ["c", "d"]
          ];
          return rows.map(rowToLine).join("\n");
        }

        document.getElementById("out").textContent = build();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "a,b\nc,d")?;
    Ok(())
}

#[test]
fn issue_165_multiline_csv_blob_download_keeps_row_breaks() -> browser_tester::Result<()> {
    let html = r#"
      <button id="download">Download</button>
      <script>
        function csvLine(values) {
          return values.map((value) => {
            const text = String(value === undefined || value === null ? "" : value);
            if (/[",\n]/.test(text)) return `"${text.replace(/"/g, "\"\"")}"`;
            return text;
          }).join(",");
        }

        function buildCsv() {
          const lines = [
            ["field_name", "field_group", "crop_name", "start_ym", "end_ym", "caution_tag", "status", "memo"],
            ["Field 1", "North Block", "Cabbage", "2026-02", "2026-05", "Brassicaceae", "fixed", "Spring crop plan"],
            ["Field 2", "North Block", "Tomato", "2026-03", "2026-08", "Solanaceae", "plan", "Summer-autumn crop"]
          ];
          return lines.map(csvLine).join("\n");
        }

        document.getElementById("download").addEventListener("click", () => {
          const blob = new Blob([buildCsv()], { type: "text/csv" });
          const url = URL.createObjectURL(blob);
          const link = document.createElement("a");
          link.href = url;
          link.download = "sample.csv";
          document.body.appendChild(link);
          link.click();
          document.body.removeChild(link);
          URL.revokeObjectURL(url);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#download")?;
    let downloads = harness.take_downloads();
    assert_eq!(downloads.len(), 1);

    let text = String::from_utf8(downloads[0].bytes.clone()).expect("download must be UTF-8");
    assert_eq!(
        text.lines().next().unwrap_or_default(),
        "field_name,field_group,crop_name,start_ym,end_ym,caution_tag,status,memo"
    );
    Ok(())
}

#[test]
fn issue_166_invalid_query_fallback_keeps_default_area_result() -> browser_tester::Result<()> {
    let html = r#"
      <div id="result"></div>
      <div id="status"></div>
      <div id="from"></div>
      <div id="to"></div>
      <script>
        const UNIT_GROUPS = {
          area: ["ha", "acre"],
          crop: ["bushel_acre", "kg_ha"]
        };

        const DEFAULT_PAIRS = {
          area: { fromUnit: "ha", toUnit: "acre" },
          crop: { fromUnit: "bushel_acre", toUnit: "kg_ha" }
        };

        const DEFAULTS = {
          category: "area",
          inputValue: "1",
          fromUnit: "ha",
          toUnit: "acre",
          localeMode: "auto",
          roundMode: "sigfig",
          significantDigits: 4,
          fixedDecimals: "auto",
          gallonType: "us",
          cropPreset: "corn",
          testWeightLbPerBushel: "56",
          showJpCustomUnits: false
        };

        const state = {
          category: DEFAULTS.category,
          inputValue: DEFAULTS.inputValue,
          fromUnit: DEFAULTS.fromUnit,
          toUnit: DEFAULTS.toUnit,
          localeMode: DEFAULTS.localeMode,
          roundMode: DEFAULTS.roundMode,
          significantDigits: DEFAULTS.significantDigits,
          fixedDecimals: DEFAULTS.fixedDecimals,
          gallonType: DEFAULTS.gallonType,
          cropPreset: DEFAULTS.cropPreset,
          testWeightLbPerBushel: DEFAULTS.testWeightLbPerBushel,
          showJpCustomUnits: DEFAULTS.showJpCustomUnits,
          lastPairs: {
            area: Object.assign({}, DEFAULT_PAIRS.area),
            crop: Object.assign({}, DEFAULT_PAIRS.crop)
          }
        };

        function getAvailableUnits(category) {
          return (UNIT_GROUPS[category] || []).slice();
        }

        function getAutoDecimals(value) {
          const abs = Math.abs(value);
          if (abs < 1) return 4;
          if (abs < 10) return 3;
          if (abs < 100) return 2;
          if (abs < 1000) return 1;
          return 0;
        }

        function formatNumber(value, currentState, options) {
          if (!Number.isFinite(value)) return "—";
          const locale = currentState.localeMode === "ja" ? "ja-JP" : "en-US";
          const fallback = options && typeof options.fallback === "number" ? options.fallback : value;
          if (currentState.roundMode === "sigfig" && !(options && options.forceFixed)) {
            return new Intl.NumberFormat(locale, {
              maximumSignificantDigits: options && options.significantDigits ? options.significantDigits : currentState.significantDigits,
              minimumSignificantDigits: 1
            }).format(value);
          }
          const decimals = options && typeof options.decimals === "number"
            ? options.decimals
            : currentState.fixedDecimals === "auto"
              ? getAutoDecimals(fallback)
              : Number(currentState.fixedDecimals);
          return new Intl.NumberFormat(locale, {
            minimumFractionDigits: decimals,
            maximumFractionDigits: decimals
          }).format(value);
        }

        function sanitizeState(candidate) {
          const next = Object.assign({}, DEFAULTS, candidate || {});
          next.category = UNIT_GROUPS[next.category] ? next.category : DEFAULTS.category;
          next.inputValue = next.inputValue == null ? DEFAULTS.inputValue : String(next.inputValue);
          next.testWeightLbPerBushel = next.testWeightLbPerBushel == null
            ? DEFAULTS.testWeightLbPerBushel
            : String(next.testWeightLbPerBushel);
          next.lastPairs = Object.assign({
            area: Object.assign({}, DEFAULT_PAIRS.area),
            crop: Object.assign({}, DEFAULT_PAIRS.crop)
          }, next.lastPairs || {});

          const available = getAvailableUnits(next.category);
          let fromUnit = available.includes(next.fromUnit) ? next.fromUnit : null;
          let toUnit = available.includes(next.toUnit) ? next.toUnit : null;
          const remembered = next.lastPairs[next.category] || DEFAULT_PAIRS[next.category];
          if (!fromUnit) fromUnit = available.includes(remembered.fromUnit) ? remembered.fromUnit : DEFAULT_PAIRS[next.category].fromUnit;
          if (!toUnit) toUnit = available.includes(remembered.toUnit) ? remembered.toUnit : DEFAULT_PAIRS[next.category].toUnit;
          next.fromUnit = fromUnit;
          next.toUnit = toUnit;
          next.lastPairs[next.category] = { fromUnit: next.fromUnit, toUnit: next.toUnit };
          return next;
        }

        function assignState(next) {
          const sanitized = sanitizeState(Object.assign({}, state, next));
          Object.keys(sanitized).forEach((key) => {
            state[key] = sanitized[key];
          });
        }

        function factorForUnit(unitKey) {
          switch (unitKey) {
            case "ha":
              return 10000;
            case "acre":
              return 4046.8564224;
            default:
              return Number.NaN;
          }
        }

        function compute() {
          const errors = [];
          const parsedValue = Number(state.inputValue);
          if (!Number.isFinite(parsedValue) || parsedValue < 0) {
            errors.push("invalid");
          }

          const fromFactor = factorForUnit(state.fromUnit);
          const toFactor = factorForUnit(state.toUnit);
          if (!Number.isFinite(fromFactor) || !Number.isFinite(toFactor) || fromFactor <= 0 || toFactor <= 0) {
            if (!errors.length) errors.push("invalid");
          }

          if (errors.length) {
            return {
              valid: false,
              status: "invalid",
              errors: errors
            };
          }

          const normalized = parsedValue * fromFactor;
          const resultValue = normalized / toFactor;
          return {
            valid: true,
            status: "ready",
            resultValue: resultValue,
            fromUnit: state.fromUnit,
            toUnit: state.toUnit
          };
        }

        window.history.replaceState(
          null,
          "",
          "https://example.com/?cat=invalid&v=1&from=broken&to=broken&tw=oops"
        );

        const params = new URLSearchParams(window.location.search || "");
        if ([...params.keys()].length) {
          assignState({
            category: params.get("cat") || state.category,
            inputValue: params.has("v") ? params.get("v") : state.inputValue,
            fromUnit: params.get("from") || state.fromUnit,
            toUnit: params.get("to") || state.toUnit,
            testWeightLbPerBushel: params.has("tw") ? params.get("tw") : state.testWeightLbPerBushel
          });
        }

        const computed = compute();
        document.getElementById("result").textContent = computed.valid
          ? formatNumber(computed.resultValue, state, { fallback: computed.resultValue })
          : "—";
        document.getElementById("status").textContent = computed.status;
        document.getElementById("from").textContent = state.fromUnit;
        document.getElementById("to").textContent = state.toUnit;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "2.471")?;
    harness.assert_text("#status", "ready")?;
    harness.assert_text("#from", "ha")?;
    harness.assert_text("#to", "acre")?;
    Ok(())
}

#[test]
fn issue_166_number_isfinite_recognizes_nan_variable() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const value = Number.NaN;
        document.getElementById("out").textContent = String(Number.isFinite(value));
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "false")?;
    Ok(())
}

#[test]
fn issue_166_nan_factor_triggers_invalid_branch() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const errors = [];
        const fromFactor = Number.NaN;
        const toFactor = Number.NaN;
        if (!Number.isFinite(fromFactor) || !Number.isFinite(toFactor) || fromFactor <= 0 || toFactor <= 0) {
          if (!errors.length) {
            errors.push("invalid");
          }
        }
        document.getElementById("out").textContent = errors.length ? errors[0] : "ready";
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "invalid")?;
    Ok(())
}

#[test]
fn issue_166_number_isfinite_accepts_numeric_function_parameter_named_value()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function check(value) {
          return String(Number.isFinite(value));
        }
        document.getElementById("out").textContent = check(1) + "|" + check(Number.NaN);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|false")?;
    Ok(())
}

#[test]
fn issue_166_number_isfinite_accepts_numeric_function_parameter_with_other_name()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function check(input) {
          return String(Number.isFinite(input));
        }
        document.getElementById("out").textContent = check(1) + "|" + check(Number.NaN);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|false")?;
    Ok(())
}

#[test]
fn issue_166_format_plain_number_formats_valid_numbers() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        document.getElementById("out").textContent = [
          formatPlainNumber(1, 10),
          formatPlainNumber(10000, 10),
          formatPlainNumber(2.471053814671653, 10)
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|10000|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_format_number_formats_valid_numbers() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function resolveLocale(currentState) {
          if (currentState.localeMode === "ja") return "ja-JP";
          if (currentState.localeMode === "en") return "en-US";
          return navigator.language || "en-US";
        }

        function getAutoDecimals(value) {
          const abs = Math.abs(value);
          if (abs < 1) return 4;
          if (abs < 10) return 3;
          if (abs < 100) return 2;
          if (abs < 1000) return 1;
          return 0;
        }

        function formatNumber(value, currentState, options) {
          if (!Number.isFinite(value)) return "—";
          const locale = resolveLocale(currentState);
          const fallback = options && typeof options.fallback === "number" ? options.fallback : value;
          try {
            if (currentState.roundMode === "sigfig" && !(options && options.forceFixed)) {
              return new Intl.NumberFormat(locale, {
                maximumSignificantDigits: options && options.significantDigits ? options.significantDigits : currentState.significantDigits,
                minimumSignificantDigits: 1
              }).format(value);
            }
            const decimals = options && typeof options.decimals === "number"
              ? options.decimals
              : currentState.fixedDecimals === "auto"
                ? getAutoDecimals(fallback)
                : Number(currentState.fixedDecimals);
            return new Intl.NumberFormat(locale, {
              minimumFractionDigits: decimals,
              maximumFractionDigits: decimals
            }).format(value);
          } catch (error) {
            return String(value);
          }
        }

        const state = {
          localeMode: "en",
          roundMode: "sigfig",
          significantDigits: 4,
          fixedDecimals: "auto"
        };

        document.getElementById("out").textContent = [
          formatNumber(1, state, { fallback: 1 }),
          formatNumber(2.471053814671653, state, { fallback: 2.471053814671653 }),
          formatNumber(10000, state, { fallback: 10000 })
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|2.471|10,000")?;
    Ok(())
}

#[test]
fn issue_166_inner_formatter_accepts_outer_local_numeric_variables() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function build() {
          const parsedValue = 1;
          const factor = 10000;
          const resultValue = 2.471053814671653;

          function formatPlainNumber(value, digits) {
            if (!Number.isFinite(value)) return "";
            if (value === 0) return "0";
            let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
              ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
              : value.toPrecision(digits || 10);
            if (text.indexOf("e") === -1) {
              text = String(Number(text));
            }
            return text;
          }

          return [
            formatPlainNumber(parsedValue, 10),
            formatPlainNumber(factor, 10),
            formatPlainNumber(resultValue, 10)
          ].join("|");
        }

        document.getElementById("out").textContent = build();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|10000|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_sibling_formatter_functions_receive_numeric_arguments() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function formatPlainNumber(value, digits) {
            if (!Number.isFinite(value)) return "";
            if (value === 0) return "0";
            let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
              ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
              : value.toPrecision(digits || 10);
            if (text.indexOf("e") === -1) {
              text = String(Number(text));
            }
            return text;
          }

          function renderFormula() {
            const parsedValue = 1;
            const factor = 10000;
            const resultValue = 2.471053814671653;
            return [
              formatPlainNumber(parsedValue, 10),
              formatPlainNumber(factor, 10),
              formatPlainNumber(resultValue, 10)
            ].join("|");
          }

          document.getElementById("out").textContent = renderFormula();
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|10000|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_page_like_compute_and_format_path_survives_invalid_query_fallback()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const pageRaw = {
          tool: {
            result: { empty: "—" },
            status: {
              needInput: "Need input",
              invalid: "Invalid",
              needFactor: "Need factor",
              ready: "Ready"
            },
            errors: {
              valueInvalid: "valueInvalid",
              testWeightRequired: "testWeightRequired",
              testWeightInvalid: "testWeightInvalid"
            },
            warnings: {
              factor: "factor",
              area: "area",
              spray: "spray",
              fertilizer: "fertilizer"
            }
          }
        };

        const UNIT_GROUPS = {
          area: ["ha", "acre", "m2", "a", "10a", "tan", "se", "tsubo"],
          spray: ["L_ha", "L_10a", "gal_acre"],
          fertilizer: ["kg_ha", "kg_10a", "lb_acre", "g_m2"],
          crop: ["bushel_acre", "kg_ha", "kg_10a", "lb_acre", "g_m2"]
        };

        const DEFAULT_PAIRS = {
          area: { fromUnit: "ha", toUnit: "acre" },
          spray: { fromUnit: "L_ha", toUnit: "gal_acre" },
          fertilizer: { fromUnit: "kg_ha", toUnit: "lb_acre" },
          crop: { fromUnit: "bushel_acre", toUnit: "kg_ha" }
        };

        const DEFAULTS = {
          category: "area",
          inputValue: "1",
          fromUnit: "ha",
          toUnit: "acre",
          localeMode: "en",
          roundMode: "sigfig",
          significantDigits: 4,
          fixedDecimals: "auto",
          gallonType: "us",
          alwaysShowFormula: false,
          showJpCustomUnits: false,
          historyEnabled: true,
          restoreLastState: true,
          cropPreset: "corn",
          testWeightLbPerBushel: "56",
          mobileTab: "input",
          favoritesOnly: false
        };

        const state = {
          category: DEFAULTS.category,
          inputValue: DEFAULTS.inputValue,
          fromUnit: DEFAULTS.fromUnit,
          toUnit: DEFAULTS.toUnit,
          localeMode: DEFAULTS.localeMode,
          roundMode: DEFAULTS.roundMode,
          significantDigits: DEFAULTS.significantDigits,
          fixedDecimals: DEFAULTS.fixedDecimals,
          gallonType: DEFAULTS.gallonType,
          alwaysShowFormula: DEFAULTS.alwaysShowFormula,
          showJpCustomUnits: DEFAULTS.showJpCustomUnits,
          historyEnabled: DEFAULTS.historyEnabled,
          restoreLastState: DEFAULTS.restoreLastState,
          cropPreset: DEFAULTS.cropPreset,
          testWeightLbPerBushel: DEFAULTS.testWeightLbPerBushel,
          mobileTab: DEFAULTS.mobileTab,
          favoritesOnly: DEFAULTS.favoritesOnly,
          formulaExpanded: false,
          isOffline: false,
          favorites: [],
          history: [],
          lastPairs: {
            area: Object.assign({}, DEFAULT_PAIRS.area),
            spray: Object.assign({}, DEFAULT_PAIRS.spray),
            fertilizer: Object.assign({}, DEFAULT_PAIRS.fertilizer),
            crop: Object.assign({}, DEFAULT_PAIRS.crop)
          }
        };

        function getAvailableUnits(category, currentState) {
          let units = (UNIT_GROUPS[category] || []).slice();
          if (category === "area" && !currentState.showJpCustomUnits) {
            units = units.filter((unit) => !["tan", "se", "tsubo"].includes(unit));
          }
          return units;
        }

        function parseFlexibleNumber(raw) {
          const text = String(raw == null ? "" : raw).trim();
          if (!text) return null;
          let normalized = text.replace(/\s+/g, "");
          const commaCount = (normalized.match(/,/g) || []).length;
          const dotCount = (normalized.match(/\./g) || []).length;
          if (commaCount && dotCount) {
            if (normalized.lastIndexOf(",") > normalized.lastIndexOf(".")) {
              normalized = normalized.replace(/\./g, "").replace(",", ".");
            } else {
              normalized = normalized.replace(/,/g, "");
            }
          } else if (commaCount && !dotCount) {
            normalized = commaCount === 1 ? normalized.replace(",", ".") : normalized.replace(/,/g, "");
          }
          const value = Number(normalized);
          return Number.isFinite(value) ? value : Number.NaN;
        }

        function resolveLocale(currentState) {
          if (currentState.localeMode === "ja") return "ja-JP";
          if (currentState.localeMode === "en") return "en-US";
          return navigator.language || "en-US";
        }

        function getAutoDecimals(value) {
          const abs = Math.abs(value);
          if (abs < 1) return 4;
          if (abs < 10) return 3;
          if (abs < 100) return 2;
          if (abs < 1000) return 1;
          return 0;
        }

        function formatNumber(value, currentState, options) {
          if (!Number.isFinite(value)) return pageRaw.tool.result.empty;
          const locale = resolveLocale(currentState);
          const fallback = options && typeof options.fallback === "number" ? options.fallback : value;
          try {
            if (currentState.roundMode === "sigfig" && !(options && options.forceFixed)) {
              return new Intl.NumberFormat(locale, {
                maximumSignificantDigits: options && options.significantDigits ? options.significantDigits : currentState.significantDigits,
                minimumSignificantDigits: 1
              }).format(value);
            }
            const decimals = options && typeof options.decimals === "number"
              ? options.decimals
              : currentState.fixedDecimals === "auto"
                ? getAutoDecimals(fallback)
                : Number(currentState.fixedDecimals);
            return new Intl.NumberFormat(locale, {
              minimumFractionDigits: decimals,
              maximumFractionDigits: decimals
            }).format(value);
          } catch (error) {
            return String(value);
          }
        }

        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function factorForUnit(unitKey, currentState) {
          switch (unitKey) {
            case "ha":
              return 10000;
            case "acre":
              return 4046.8564224;
            case "m2":
              return 1;
            case "a":
              return 100;
            case "10a":
              return 1000;
            case "tsubo":
              return 400 / 121;
            case "se":
              return 30 * (400 / 121);
            case "tan":
              return 300 * (400 / 121);
            case "L_ha":
              return 1;
            case "L_10a":
              return 10;
            case "gal_acre":
              return currentState.gallonType === "imp" ? 11.233633036340657 : 9.353956228956229;
            case "kg_ha":
              return 1;
            case "kg_10a":
              return 10;
            case "lb_acre":
              return 1.120851156194456;
            case "g_m2":
              return 10;
            case "bushel_acre": {
              const factorWeight = parseFlexibleNumber(currentState.testWeightLbPerBushel);
              if (!Number.isFinite(factorWeight) || factorWeight <= 0) return Number.NaN;
              return factorWeight * 1.120851156194456;
            }
            default:
              return Number.NaN;
          }
        }

        function sanitizeState(candidate) {
          const next = Object.assign({}, DEFAULTS, candidate || {});
          next.category = UNIT_GROUPS[next.category] ? next.category : DEFAULTS.category;
          next.localeMode = ["auto", "ja", "en"].includes(next.localeMode) ? next.localeMode : DEFAULTS.localeMode;
          next.roundMode = ["sigfig", "fixed"].includes(next.roundMode) ? next.roundMode : DEFAULTS.roundMode;
          next.significantDigits = [2, 3, 4, 5, 6, 8].includes(Number(next.significantDigits)) ? Number(next.significantDigits) : DEFAULTS.significantDigits;
          next.fixedDecimals = ["auto", "0", "1", "2", "3", "4", "5", "6", 0, 1, 2, 3, 4, 5, 6].includes(next.fixedDecimals)
            ? String(next.fixedDecimals)
            : DEFAULTS.fixedDecimals;
          next.gallonType = ["us", "imp"].includes(next.gallonType) ? next.gallonType : DEFAULTS.gallonType;
          next.alwaysShowFormula = Boolean(next.alwaysShowFormula);
          next.showJpCustomUnits = Boolean(next.showJpCustomUnits);
          next.historyEnabled = next.historyEnabled !== false;
          next.restoreLastState = next.restoreLastState !== false;
          next.cropPreset = ["corn", "wheat", "custom"].includes(next.cropPreset) ? next.cropPreset : DEFAULTS.cropPreset;
          next.inputValue = next.inputValue == null ? DEFAULTS.inputValue : String(next.inputValue);
          next.testWeightLbPerBushel = next.testWeightLbPerBushel == null ? DEFAULTS.testWeightLbPerBushel : String(next.testWeightLbPerBushel);
          next.mobileTab = next.mobileTab === "output" ? "output" : "input";
          next.favoritesOnly = Boolean(next.favoritesOnly);
          next.formulaExpanded = Boolean(next.formulaExpanded);
          next.isOffline = Boolean(next.isOffline);
          next.favorites = Array.isArray(next.favorites) ? next.favorites : [];
          next.history = Array.isArray(next.history) ? next.history : [];
          next.lastPairs = Object.assign({
            area: Object.assign({}, DEFAULT_PAIRS.area),
            spray: Object.assign({}, DEFAULT_PAIRS.spray),
            fertilizer: Object.assign({}, DEFAULT_PAIRS.fertilizer),
            crop: Object.assign({}, DEFAULT_PAIRS.crop)
          }, next.lastPairs || {});

          if (next.cropPreset === "corn" && (!next.testWeightLbPerBushel || next.testWeightLbPerBushel === DEFAULTS.testWeightLbPerBushel)) {
            next.testWeightLbPerBushel = "56";
          }
          if (next.cropPreset === "wheat" && (!next.testWeightLbPerBushel || next.testWeightLbPerBushel === DEFAULTS.testWeightLbPerBushel)) {
            next.testWeightLbPerBushel = "60";
          }

          const usesJpUnits = ["tan", "se", "tsubo"].includes(next.fromUnit) || ["tan", "se", "tsubo"].includes(next.toUnit);
          if (usesJpUnits && !next.showJpCustomUnits) {
            next.showJpCustomUnits = true;
          }

          const available = getAvailableUnits(next.category, next);
          let fromUnit = available.includes(next.fromUnit) ? next.fromUnit : null;
          let toUnit = available.includes(next.toUnit) ? next.toUnit : null;
          const remembered = next.lastPairs[next.category] || DEFAULT_PAIRS[next.category];
          if (!fromUnit) fromUnit = available.includes(remembered.fromUnit) ? remembered.fromUnit : DEFAULT_PAIRS[next.category].fromUnit;
          if (!toUnit) toUnit = available.includes(remembered.toUnit) ? remembered.toUnit : DEFAULT_PAIRS[next.category].toUnit;
          next.fromUnit = fromUnit;
          next.toUnit = toUnit;
          next.lastPairs[next.category] = { fromUnit: next.fromUnit, toUnit: next.toUnit };
          return next;
        }

        function assignState(next) {
          const sanitized = sanitizeState(Object.assign({}, state, next));
          Object.keys(sanitized).forEach((key) => {
            state[key] = sanitized[key];
          });
        }

        function restoreFromQuery() {
          const params = new URLSearchParams(window.location.search || "");
          if (![...params.keys()].length) return false;
          assignState({
            category: params.get("cat") || state.category,
            inputValue: params.has("v") ? params.get("v") : state.inputValue,
            fromUnit: params.get("from") || state.fromUnit,
            toUnit: params.get("to") || state.toUnit,
            cropPreset: params.get("crop") || state.cropPreset,
            testWeightLbPerBushel: params.has("tw") ? params.get("tw") : state.testWeightLbPerBushel,
            localeMode: params.get("loc") || state.localeMode,
            roundMode: params.get("rm") || state.roundMode,
            significantDigits: params.get("sig") || state.significantDigits,
            fixedDecimals: params.get("dec") || state.fixedDecimals,
            gallonType: params.get("gal") || state.gallonType,
            alwaysShowFormula: params.get("sf") === "1",
            showJpCustomUnits: params.get("jp") === "1" || state.showJpCustomUnits,
            favoritesOnly: params.get("fo") === "1",
            mobileTab: params.get("tab") || state.mobileTab
          });
          return true;
        }

        function compute() {
          const errors = [];
          const warnings = [];
          const parsedValue = parseFlexibleNumber(state.inputValue);
          if (state.inputValue.trim() === "") {
            return {
              valid: false,
              needsInput: true,
              status: pageRaw.tool.status.needInput,
              errors,
              warnings
            };
          }
          if (!Number.isFinite(parsedValue) || parsedValue < 0) {
            errors.push(pageRaw.tool.errors.valueInvalid);
          }
          const needsFactor = state.category === "crop" && (state.fromUnit === "bushel_acre" || state.toUnit === "bushel_acre");
          const factorWeight = parseFlexibleNumber(state.testWeightLbPerBushel);
          if (needsFactor) {
            if (state.testWeightLbPerBushel.trim() === "") {
              errors.push(pageRaw.tool.errors.testWeightRequired);
            } else if (!Number.isFinite(factorWeight) || factorWeight <= 0) {
              errors.push(pageRaw.tool.errors.testWeightInvalid);
            } else if (factorWeight < 20 || factorWeight > 100) {
              warnings.push(pageRaw.tool.warnings.factor);
            }
          }

          const fromFactor = factorForUnit(state.fromUnit, state);
          const toFactor = factorForUnit(state.toUnit, state);
          if (!Number.isFinite(fromFactor) || !Number.isFinite(toFactor) || fromFactor <= 0 || toFactor <= 0) {
            if (!errors.length) {
              errors.push(needsFactor ? pageRaw.tool.errors.testWeightRequired : pageRaw.tool.errors.valueInvalid);
            }
          }

          if (errors.length) {
            return {
              valid: false,
              needsInput: false,
              status: needsFactor ? pageRaw.tool.status.needFactor : pageRaw.tool.status.invalid,
              errors,
              warnings
            };
          }

          const normalized = parsedValue * fromFactor;
          const resultValue = normalized / toFactor;
          const factor = fromFactor / toFactor;
          const inverseFactor = toFactor / fromFactor;

          if (state.category === "area" && normalized / 10000 > 100000) {
            warnings.push(pageRaw.tool.warnings.area);
          }
          if (state.category === "spray" && normalized > 10000) {
            warnings.push(pageRaw.tool.warnings.spray);
          }
          if ((state.category === "fertilizer" || state.category === "crop") && normalized > 5000) {
            warnings.push(pageRaw.tool.warnings.fertilizer);
          }

          const formulaText = [
            formatPlainNumber(parsedValue, 10),
            " × ",
            formatPlainNumber(factor, 10),
            " = ",
            formatPlainNumber(resultValue, 10),
            " acre"
          ].join("");

          return {
            valid: true,
            needsInput: false,
            status: pageRaw.tool.status.ready,
            inputValue: parsedValue,
            resultValue,
            normalized,
            factor,
            inverseFactor,
            errors,
            warnings,
            formulaText,
            coefficientLines: []
          };
        }

        window.history.replaceState(
          null,
          "",
          "https://example.com/?cat=invalid&v=1&from=broken&to=broken&tw=oops"
        );

        restoreFromQuery();
        const computed = compute();
        document.getElementById("out").textContent = [
          computed.status,
          formatNumber(computed.resultValue, state, { fallback: computed.resultValue }),
          computed.formulaText
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "Ready|2.471|1 × 2.471053815 = 2.471053815 acre")?;
    Ok(())
}

#[test]
fn issue_166_page_like_context_formats_direct_numeric_literals() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function formatPlainNumber(value, digits) {
            if (!Number.isFinite(value)) return "";
            if (value === 0) return "0";
            let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
              ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
              : value.toPrecision(digits || 10);
            if (text.indexOf("e") === -1) {
              text = String(Number(text));
            }
            return text;
          }

          function compute() {
            return [
              formatPlainNumber(1, 10),
              formatPlainNumber(10000, 10),
              formatPlainNumber(2.471053814671653, 10)
            ].join("|");
          }

          document.getElementById("out").textContent = compute();
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|10000|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_page_like_context_formats_numeric_locals_from_expressions()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function parseFlexibleNumber(raw) {
            const value = Number(String(raw == null ? "" : raw).trim());
            return Number.isFinite(value) ? value : Number.NaN;
          }

          function formatPlainNumber(value, digits) {
            if (!Number.isFinite(value)) return "";
            if (value === 0) return "0";
            let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
              ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
              : value.toPrecision(digits || 10);
            if (text.indexOf("e") === -1) {
              text = String(Number(text));
            }
            return text;
          }

          function compute() {
            const parsedValue = parseFlexibleNumber("1");
            const fromFactor = 10000;
            const toFactor = 4046.8564224;
            const factor = fromFactor / toFactor;
            const resultValue = parsedValue * factor;
            return [
              formatPlainNumber(parsedValue, 10),
              formatPlainNumber(factor, 10),
              formatPlainNumber(resultValue, 10)
            ].join("|");
          }

          document.getElementById("out").textContent = compute();
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|2.471053815|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_page_like_context_formats_numeric_locals_from_state_properties()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          const state = {
            inputValue: "1",
            fromUnit: "ha",
            toUnit: "acre",
            gallonType: "us"
          };

          function parseFlexibleNumber(raw) {
            const value = Number(String(raw == null ? "" : raw).trim());
            return Number.isFinite(value) ? value : Number.NaN;
          }

          function factorForUnit(unitKey, currentState) {
            switch (unitKey) {
              case "ha":
                return 10000;
              case "acre":
                return 4046.8564224;
              case "gal_acre":
                return currentState.gallonType === "imp" ? 11.233633036340657 : 9.353956228956229;
              default:
                return Number.NaN;
            }
          }

          function formatPlainNumber(value, digits) {
            if (!Number.isFinite(value)) return "";
            if (value === 0) return "0";
            let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
              ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
              : value.toPrecision(digits || 10);
            if (text.indexOf("e") === -1) {
              text = String(Number(text));
            }
            return text;
          }

          function compute() {
            const parsedValue = parseFlexibleNumber(state.inputValue);
            const fromFactor = factorForUnit(state.fromUnit, state);
            const toFactor = factorForUnit(state.toUnit, state);
            const factor = fromFactor / toFactor;
            const resultValue = parsedValue * factor;
            return [
              formatPlainNumber(parsedValue, 10),
              formatPlainNumber(factor, 10),
              formatPlainNumber(resultValue, 10)
            ].join("|");
          }

          document.getElementById("out").textContent = compute();
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|2.471053815|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_top_level_compute_calls_top_level_formatter_with_numeric_locals()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function compute() {
          const parsedValue = 1;
          const factor = 10000 / 4046.8564224;
          const resultValue = parsedValue * factor;
          return [
            formatPlainNumber(parsedValue, 10),
            formatPlainNumber(factor, 10),
            formatPlainNumber(resultValue, 10)
          ].join("|");
        }

        document.getElementById("out").textContent = compute();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|2.471053815|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_top_level_compute_keeps_formatter_args_after_dead_error_paths()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function compute() {
          const errors = [];
          const warnings = [];
          const parsedValue = 1;
          if (false) {
            return { valid: false, errors, warnings };
          }
          const factor = 10000 / 4046.8564224;
          const resultValue = parsedValue * factor;
          if (errors.length) {
            return { valid: false, errors, warnings };
          }
          return [
            formatPlainNumber(parsedValue, 10),
            formatPlainNumber(factor, 10),
            formatPlainNumber(resultValue, 10)
          ].join("|");
        }

        document.getElementById("out").textContent = compute();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|2.471053815|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_page_like_compute_passes_finite_numbers_into_formula_formatter()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const UNIT_GROUPS = {
          area: ["ha", "acre", "m2", "a", "10a", "tan", "se", "tsubo"]
        };
        const DEFAULT_PAIRS = {
          area: { fromUnit: "ha", toUnit: "acre" }
        };
        const DEFAULTS = {
          category: "area",
          inputValue: "1",
          fromUnit: "ha",
          toUnit: "acre",
          testWeightLbPerBushel: "56",
          showJpCustomUnits: false
        };
        const state = {
          category: DEFAULTS.category,
          inputValue: DEFAULTS.inputValue,
          fromUnit: DEFAULTS.fromUnit,
          toUnit: DEFAULTS.toUnit,
          testWeightLbPerBushel: DEFAULTS.testWeightLbPerBushel,
          showJpCustomUnits: DEFAULTS.showJpCustomUnits,
          lastPairs: {
            area: Object.assign({}, DEFAULT_PAIRS.area)
          }
        };

        function getAvailableUnits(category, currentState) {
          let units = (UNIT_GROUPS[category] || []).slice();
          if (category === "area" && !currentState.showJpCustomUnits) {
            units = units.filter((unit) => !["tan", "se", "tsubo"].includes(unit));
          }
          return units;
        }

        function parseFlexibleNumber(raw) {
          const value = Number(String(raw == null ? "" : raw).trim());
          return Number.isFinite(value) ? value : Number.NaN;
        }

        function factorForUnit(unitKey) {
          switch (unitKey) {
            case "ha":
              return 10000;
            case "acre":
              return 4046.8564224;
            default:
              return Number.NaN;
          }
        }

        function inspectNumber(value) {
          return [String(value), typeof value, String(Number.isFinite(value))].join(":");
        }

        function sanitizeState(candidate) {
          const next = Object.assign({}, DEFAULTS, candidate || {});
          next.category = UNIT_GROUPS[next.category] ? next.category : DEFAULTS.category;
          const available = getAvailableUnits(next.category, next);
          let fromUnit = available.includes(next.fromUnit) ? next.fromUnit : null;
          let toUnit = available.includes(next.toUnit) ? next.toUnit : null;
          const remembered = next.lastPairs[next.category] || DEFAULT_PAIRS[next.category];
          if (!fromUnit) fromUnit = available.includes(remembered.fromUnit) ? remembered.fromUnit : DEFAULT_PAIRS[next.category].fromUnit;
          if (!toUnit) toUnit = available.includes(remembered.toUnit) ? remembered.toUnit : DEFAULT_PAIRS[next.category].toUnit;
          next.fromUnit = fromUnit;
          next.toUnit = toUnit;
          next.lastPairs[next.category] = { fromUnit: next.fromUnit, toUnit: next.toUnit };
          return next;
        }

        function assignState(next) {
          const sanitized = sanitizeState(Object.assign({}, state, next));
          Object.keys(sanitized).forEach((key) => {
            state[key] = sanitized[key];
          });
        }

        function restoreFromQuery() {
          const params = new URLSearchParams(window.location.search || "");
          assignState({
            category: params.get("cat") || state.category,
            inputValue: params.has("v") ? params.get("v") : state.inputValue,
            fromUnit: params.get("from") || state.fromUnit,
            toUnit: params.get("to") || state.toUnit
          });
        }

        function compute() {
          const parsedValue = parseFlexibleNumber(state.inputValue);
          const fromFactor = factorForUnit(state.fromUnit, state);
          const toFactor = factorForUnit(state.toUnit, state);
          const factor = fromFactor / toFactor;
          const resultValue = parsedValue * factor;
          return [
            inspectNumber(parsedValue),
            inspectNumber(factor),
            inspectNumber(resultValue)
          ].join("|");
        }

        window.history.replaceState(
          null,
          "",
          "https://example.com/?cat=invalid&v=1&from=broken&to=broken"
        );

        restoreFromQuery();
        document.getElementById("out").textContent = compute();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "1:number:true|2.471053814671653:number:true|2.471053814671653:number:true",
    )?;
    Ok(())
}

#[test]
fn issue_166_preceding_format_number_definition_does_not_break_format_plain_number()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function resolveLocale(currentState) {
          if (currentState.localeMode === "ja") return "ja-JP";
          if (currentState.localeMode === "en") return "en-US";
          return navigator.language || "en-US";
        }

        function getAutoDecimals(value) {
          const abs = Math.abs(value);
          if (abs < 1) return 4;
          if (abs < 10) return 3;
          if (abs < 100) return 2;
          if (abs < 1000) return 1;
          return 0;
        }

        function formatNumber(value, currentState, options) {
          if (!Number.isFinite(value)) return "—";
          const locale = resolveLocale(currentState);
          const fallback = options && typeof options.fallback === "number" ? options.fallback : value;
          try {
            if (currentState.roundMode === "sigfig" && !(options && options.forceFixed)) {
              return new Intl.NumberFormat(locale, {
                maximumSignificantDigits: options && options.significantDigits ? options.significantDigits : currentState.significantDigits,
                minimumSignificantDigits: 1
              }).format(value);
            }
            const decimals = options && typeof options.decimals === "number"
              ? options.decimals
              : currentState.fixedDecimals === "auto"
                ? getAutoDecimals(fallback)
                : Number(currentState.fixedDecimals);
            return new Intl.NumberFormat(locale, {
              minimumFractionDigits: decimals,
              maximumFractionDigits: decimals
            }).format(value);
          } catch (error) {
            return String(value);
          }
        }

        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function compute() {
          const parsedValue = 1;
          const factor = 10000 / 4046.8564224;
          const resultValue = parsedValue * factor;
          return [
            formatPlainNumber(parsedValue, 10),
            formatPlainNumber(factor, 10),
            formatPlainNumber(resultValue, 10)
          ].join("|");
        }

        document.getElementById("out").textContent = compute();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|2.471053815|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_compute_can_return_object_after_formatting_numeric_locals()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function compute() {
          const parsedValue = 1;
          const factor = 10000 / 4046.8564224;
          const resultValue = parsedValue * factor;
          const formulaText = [
            formatPlainNumber(parsedValue, 10),
            formatPlainNumber(factor, 10),
            formatPlainNumber(resultValue, 10)
          ].join("|");
          return {
            valid: true,
            resultValue,
            formulaText
          };
        }

        const computed = compute();
        document.getElementById("out").textContent = [
          String(computed.valid),
          String(computed.resultValue),
          computed.formulaText
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|2.471053814671653|1|2.471053815|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_compute_control_flow_does_not_poison_formatter_args() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function compute() {
          const errors = [];
          const warnings = [];
          const parsedValue = 1;
          if (false) {
            return {
              valid: false,
              needsInput: true,
              status: "Need input",
              errors,
              warnings
            };
          }
          if (!Number.isFinite(parsedValue) || parsedValue < 0) {
            errors.push("valueInvalid");
          }
          const needsFactor = false;
          const factorWeight = 56;
          if (needsFactor) {
            if (false) {
              errors.push("testWeightRequired");
            } else if (!Number.isFinite(factorWeight) || factorWeight <= 0) {
              errors.push("testWeightInvalid");
            } else if (factorWeight < 20 || factorWeight > 100) {
              warnings.push("factor");
            }
          }

          const fromFactor = 10000;
          const toFactor = 4046.8564224;
          if (!Number.isFinite(fromFactor) || !Number.isFinite(toFactor) || fromFactor <= 0 || toFactor <= 0) {
            if (!errors.length) {
              errors.push(needsFactor ? "testWeightRequired" : "valueInvalid");
            }
          }

          if (errors.length) {
            return {
              valid: false,
              needsInput: false,
              status: needsFactor ? "Need factor" : "Invalid",
              errors,
              warnings
            };
          }

          const normalized = parsedValue * fromFactor;
          const resultValue = normalized / toFactor;
          const factor = fromFactor / toFactor;
          const inverseFactor = toFactor / fromFactor;

          if (false) {
            warnings.push("area");
          }
          if (false) {
            warnings.push("spray");
          }
          if (false) {
            warnings.push("fertilizer");
          }

          const formulaText = [
            formatPlainNumber(parsedValue, 10),
            " × ",
            formatPlainNumber(factor, 10),
            " = ",
            formatPlainNumber(resultValue, 10)
          ].join("");

          return {
            valid: true,
            needsInput: false,
            status: "Ready",
            inputValue: parsedValue,
            resultValue,
            normalized,
            factor,
            inverseFactor,
            errors,
            warnings,
            formulaText
          };
        }

        const computed = compute();
        document.getElementById("out").textContent = [
          computed.status,
          String(computed.resultValue),
          computed.formulaText
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "Ready|2.471053814671653|1 × 2.471053815 = 2.471053815",
    )?;
    Ok(())
}

#[test]
fn issue_166_real_helper_calls_do_not_poison_formatter_args() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const state = {
          category: "area",
          inputValue: "1",
          fromUnit: "ha",
          toUnit: "acre",
          localeMode: "en",
          roundMode: "sigfig",
          significantDigits: 4,
          fixedDecimals: "auto",
          gallonType: "us",
          testWeightLbPerBushel: "56"
        };

        function parseFlexibleNumber(raw) {
          const text = String(raw == null ? "" : raw).trim();
          if (!text) return null;
          let normalized = text.replace(/\s+/g, "");
          const commaCount = (normalized.match(/,/g) || []).length;
          const dotCount = (normalized.match(/\./g) || []).length;
          if (commaCount && dotCount) {
            if (normalized.lastIndexOf(",") > normalized.lastIndexOf(".")) {
              normalized = normalized.replace(/\./g, "").replace(",", ".");
            } else {
              normalized = normalized.replace(/,/g, "");
            }
          } else if (commaCount && !dotCount) {
            normalized = commaCount === 1 ? normalized.replace(",", ".") : normalized.replace(/,/g, "");
          }
          const value = Number(normalized);
          return Number.isFinite(value) ? value : Number.NaN;
        }

        function factorForUnit(unitKey, currentState) {
          switch (unitKey) {
            case "ha":
              return 10000;
            case "acre":
              return 4046.8564224;
            case "m2":
              return 1;
            case "a":
              return 100;
            case "10a":
              return 1000;
            case "tsubo":
              return 400 / 121;
            case "se":
              return 30 * (400 / 121);
            case "tan":
              return 300 * (400 / 121);
            case "L_ha":
              return 1;
            case "L_10a":
              return 10;
            case "gal_acre":
              return currentState.gallonType === "imp" ? 11.233633036340657 : 9.353956228956229;
            case "kg_ha":
              return 1;
            case "kg_10a":
              return 10;
            case "lb_acre":
              return 1.120851156194456;
            case "g_m2":
              return 10;
            case "bushel_acre": {
              const factorWeight = parseFlexibleNumber(currentState.testWeightLbPerBushel);
              if (!Number.isFinite(factorWeight) || factorWeight <= 0) return Number.NaN;
              return factorWeight * 1.120851156194456;
            }
            default:
              return Number.NaN;
          }
        }

        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function compute() {
          const errors = [];
          const warnings = [];
          const parsedValue = parseFlexibleNumber(state.inputValue);
          const fromFactor = factorForUnit(state.fromUnit, state);
          const toFactor = factorForUnit(state.toUnit, state);
          const normalized = parsedValue * fromFactor;
          const resultValue = normalized / toFactor;
          const factor = fromFactor / toFactor;
          return {
            valid: !errors.length,
            warnings,
            resultValue,
            formulaText: [
              formatPlainNumber(parsedValue, 10),
              " × ",
              formatPlainNumber(factor, 10),
              " = ",
              formatPlainNumber(resultValue, 10)
            ].join("")
          };
        }

        const computed = compute();
        document.getElementById("out").textContent = [
          String(computed.valid),
          String(computed.resultValue),
          computed.formulaText
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "true|2.471053814671653|1 × 2.471053815 = 2.471053815",
    )?;
    Ok(())
}

#[test]
fn issue_166_unrelated_nan_local_does_not_poison_formatter_args() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function compute() {
          const errors = [];
          const warnings = [];
          const parsedValue = 1;
          const needsFactor = false;
          const factorWeight = Number.NaN;
          if (needsFactor) {
            if (!Number.isFinite(factorWeight) || factorWeight <= 0) {
              errors.push("testWeightInvalid");
            }
          }
          const fromFactor = 10000;
          const toFactor = 4046.8564224;
          const normalized = parsedValue * fromFactor;
          const resultValue = normalized / toFactor;
          const factor = fromFactor / toFactor;
          return [
            formatPlainNumber(parsedValue, 10),
            formatPlainNumber(factor, 10),
            formatPlainNumber(resultValue, 10)
          ].join("|");
        }

        document.getElementById("out").textContent = compute();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1|2.471053815|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_prior_global_call_with_nan_result_does_not_poison_formatter_params()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function parseFlexibleNumber(raw) {
          const text = String(raw == null ? "" : raw).trim();
          if (!text) return null;
          const value = Number(text);
          return Number.isFinite(value) ? value : Number.NaN;
        }

        function formatPlainNumber(value, digits) {
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function compute() {
          const factorWeight = parseFlexibleNumber("oops");
          const parsedValue = 1;
          const factor = 10000 / 4046.8564224;
          const resultValue = parsedValue * factor;
          return [
            String(Number.isNaN(factorWeight)),
            formatPlainNumber(parsedValue, 10),
            formatPlainNumber(factor, 10),
            formatPlainNumber(resultValue, 10)
          ].join("|");
        }

        document.getElementById("out").textContent = compute();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|1|2.471053815|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_page_like_full_repro_traces_values_seen_by_format_plain_number()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const pageRaw = {
          tool: {
            result: { empty: "—" },
            status: {
              needInput: "Need input",
              invalid: "Invalid",
              needFactor: "Need factor",
              ready: "Ready"
            },
            errors: {
              valueInvalid: "valueInvalid",
              testWeightRequired: "testWeightRequired",
              testWeightInvalid: "testWeightInvalid"
            },
            warnings: {
              factor: "factor",
              area: "area",
              spray: "spray",
              fertilizer: "fertilizer"
            }
          }
        };

        const UNIT_GROUPS = {
          area: ["ha", "acre", "m2", "a", "10a", "tan", "se", "tsubo"],
          spray: ["L_ha", "L_10a", "gal_acre"],
          fertilizer: ["kg_ha", "kg_10a", "lb_acre", "g_m2"],
          crop: ["bushel_acre", "kg_ha", "kg_10a", "lb_acre", "g_m2"]
        };

        const DEFAULT_PAIRS = {
          area: { fromUnit: "ha", toUnit: "acre" },
          spray: { fromUnit: "L_ha", toUnit: "gal_acre" },
          fertilizer: { fromUnit: "kg_ha", toUnit: "lb_acre" },
          crop: { fromUnit: "bushel_acre", toUnit: "kg_ha" }
        };

        const DEFAULTS = {
          category: "area",
          inputValue: "1",
          fromUnit: "ha",
          toUnit: "acre",
          localeMode: "en",
          roundMode: "sigfig",
          significantDigits: 4,
          fixedDecimals: "auto",
          gallonType: "us",
          alwaysShowFormula: false,
          showJpCustomUnits: false,
          historyEnabled: true,
          restoreLastState: true,
          cropPreset: "corn",
          testWeightLbPerBushel: "56",
          mobileTab: "input",
          favoritesOnly: false
        };

        const state = {
          category: DEFAULTS.category,
          inputValue: DEFAULTS.inputValue,
          fromUnit: DEFAULTS.fromUnit,
          toUnit: DEFAULTS.toUnit,
          localeMode: DEFAULTS.localeMode,
          roundMode: DEFAULTS.roundMode,
          significantDigits: DEFAULTS.significantDigits,
          fixedDecimals: DEFAULTS.fixedDecimals,
          gallonType: DEFAULTS.gallonType,
          alwaysShowFormula: DEFAULTS.alwaysShowFormula,
          showJpCustomUnits: DEFAULTS.showJpCustomUnits,
          historyEnabled: DEFAULTS.historyEnabled,
          restoreLastState: DEFAULTS.restoreLastState,
          cropPreset: DEFAULTS.cropPreset,
          testWeightLbPerBushel: DEFAULTS.testWeightLbPerBushel,
          mobileTab: DEFAULTS.mobileTab,
          favoritesOnly: DEFAULTS.favoritesOnly,
          formulaExpanded: false,
          isOffline: false,
          favorites: [],
          history: [],
          lastPairs: {
            area: Object.assign({}, DEFAULT_PAIRS.area),
            spray: Object.assign({}, DEFAULT_PAIRS.spray),
            fertilizer: Object.assign({}, DEFAULT_PAIRS.fertilizer),
            crop: Object.assign({}, DEFAULT_PAIRS.crop)
          }
        };
        const trace = [];

        function getAvailableUnits(category, currentState) {
          let units = (UNIT_GROUPS[category] || []).slice();
          if (category === "area" && !currentState.showJpCustomUnits) {
            units = units.filter((unit) => !["tan", "se", "tsubo"].includes(unit));
          }
          return units;
        }

        function parseFlexibleNumber(raw) {
          const text = String(raw == null ? "" : raw).trim();
          if (!text) return null;
          let normalized = text.replace(/\s+/g, "");
          const commaCount = (normalized.match(/,/g) || []).length;
          const dotCount = (normalized.match(/\./g) || []).length;
          if (commaCount && dotCount) {
            if (normalized.lastIndexOf(",") > normalized.lastIndexOf(".")) {
              normalized = normalized.replace(/\./g, "").replace(",", ".");
            } else {
              normalized = normalized.replace(/,/g, "");
            }
          } else if (commaCount && !dotCount) {
            normalized = commaCount === 1 ? normalized.replace(",", ".") : normalized.replace(/,/g, "");
          }
          const value = Number(normalized);
          return Number.isFinite(value) ? value : Number.NaN;
        }

        function resolveLocale(currentState) {
          if (currentState.localeMode === "ja") return "ja-JP";
          if (currentState.localeMode === "en") return "en-US";
          return navigator.language || "en-US";
        }

        function getAutoDecimals(value) {
          const abs = Math.abs(value);
          if (abs < 1) return 4;
          if (abs < 10) return 3;
          if (abs < 100) return 2;
          if (abs < 1000) return 1;
          return 0;
        }

        function formatNumber(value, currentState, options) {
          if (!Number.isFinite(value)) return pageRaw.tool.result.empty;
          const locale = resolveLocale(currentState);
          const fallback = options && typeof options.fallback === "number" ? options.fallback : value;
          try {
            if (currentState.roundMode === "sigfig" && !(options && options.forceFixed)) {
              return new Intl.NumberFormat(locale, {
                maximumSignificantDigits: options && options.significantDigits ? options.significantDigits : currentState.significantDigits,
                minimumSignificantDigits: 1
              }).format(value);
            }
            const decimals = options && typeof options.decimals === "number"
              ? options.decimals
              : currentState.fixedDecimals === "auto"
                ? getAutoDecimals(fallback)
                : Number(currentState.fixedDecimals);
            return new Intl.NumberFormat(locale, {
              minimumFractionDigits: decimals,
              maximumFractionDigits: decimals
            }).format(value);
          } catch (error) {
            return String(value);
          }
        }

        function formatPlainNumber(value, digits) {
          trace.push([String(value), typeof value, String(Number.isFinite(value))].join(":"));
          if (!Number.isFinite(value)) return "";
          if (value === 0) return "0";
          let text = Math.abs(value) < 1e-4 || Math.abs(value) >= 1e9
            ? value.toExponential(Math.min(Math.max((digits || 10) - 1, 1), 8))
            : value.toPrecision(digits || 10);
          if (text.indexOf("e") === -1) {
            text = String(Number(text));
          }
          return text;
        }

        function factorForUnit(unitKey, currentState) {
          switch (unitKey) {
            case "ha":
              return 10000;
            case "acre":
              return 4046.8564224;
            case "m2":
              return 1;
            case "a":
              return 100;
            case "10a":
              return 1000;
            case "tsubo":
              return 400 / 121;
            case "se":
              return 30 * (400 / 121);
            case "tan":
              return 300 * (400 / 121);
            case "L_ha":
              return 1;
            case "L_10a":
              return 10;
            case "gal_acre":
              return currentState.gallonType === "imp" ? 11.233633036340657 : 9.353956228956229;
            case "kg_ha":
              return 1;
            case "kg_10a":
              return 10;
            case "lb_acre":
              return 1.120851156194456;
            case "g_m2":
              return 10;
            case "bushel_acre": {
              const factorWeight = parseFlexibleNumber(currentState.testWeightLbPerBushel);
              if (!Number.isFinite(factorWeight) || factorWeight <= 0) return Number.NaN;
              return factorWeight * 1.120851156194456;
            }
            default:
              return Number.NaN;
          }
        }

        function sanitizeState(candidate) {
          const next = Object.assign({}, DEFAULTS, candidate || {});
          next.category = UNIT_GROUPS[next.category] ? next.category : DEFAULTS.category;
          next.localeMode = ["auto", "ja", "en"].includes(next.localeMode) ? next.localeMode : DEFAULTS.localeMode;
          next.roundMode = ["sigfig", "fixed"].includes(next.roundMode) ? next.roundMode : DEFAULTS.roundMode;
          next.significantDigits = [2, 3, 4, 5, 6, 8].includes(Number(next.significantDigits)) ? Number(next.significantDigits) : DEFAULTS.significantDigits;
          next.fixedDecimals = ["auto", "0", "1", "2", "3", "4", "5", "6", 0, 1, 2, 3, 4, 5, 6].includes(next.fixedDecimals)
            ? String(next.fixedDecimals)
            : DEFAULTS.fixedDecimals;
          next.gallonType = ["us", "imp"].includes(next.gallonType) ? next.gallonType : DEFAULTS.gallonType;
          next.alwaysShowFormula = Boolean(next.alwaysShowFormula);
          next.showJpCustomUnits = Boolean(next.showJpCustomUnits);
          next.historyEnabled = next.historyEnabled !== false;
          next.restoreLastState = next.restoreLastState !== false;
          next.cropPreset = ["corn", "wheat", "custom"].includes(next.cropPreset) ? next.cropPreset : DEFAULTS.cropPreset;
          next.inputValue = next.inputValue == null ? DEFAULTS.inputValue : String(next.inputValue);
          next.testWeightLbPerBushel = next.testWeightLbPerBushel == null ? DEFAULTS.testWeightLbPerBushel : String(next.testWeightLbPerBushel);
          next.mobileTab = next.mobileTab === "output" ? "output" : "input";
          next.favoritesOnly = Boolean(next.favoritesOnly);
          next.formulaExpanded = Boolean(next.formulaExpanded);
          next.isOffline = Boolean(next.isOffline);
          next.favorites = Array.isArray(next.favorites) ? next.favorites : [];
          next.history = Array.isArray(next.history) ? next.history : [];
          next.lastPairs = Object.assign({
            area: Object.assign({}, DEFAULT_PAIRS.area),
            spray: Object.assign({}, DEFAULT_PAIRS.spray),
            fertilizer: Object.assign({}, DEFAULT_PAIRS.fertilizer),
            crop: Object.assign({}, DEFAULT_PAIRS.crop)
          }, next.lastPairs || {});

          if (next.cropPreset === "corn" && (!next.testWeightLbPerBushel || next.testWeightLbPerBushel === DEFAULTS.testWeightLbPerBushel)) {
            next.testWeightLbPerBushel = "56";
          }
          if (next.cropPreset === "wheat" && (!next.testWeightLbPerBushel || next.testWeightLbPerBushel === DEFAULTS.testWeightLbPerBushel)) {
            next.testWeightLbPerBushel = "60";
          }

          const usesJpUnits = ["tan", "se", "tsubo"].includes(next.fromUnit) || ["tan", "se", "tsubo"].includes(next.toUnit);
          if (usesJpUnits && !next.showJpCustomUnits) {
            next.showJpCustomUnits = true;
          }

          const available = getAvailableUnits(next.category, next);
          let fromUnit = available.includes(next.fromUnit) ? next.fromUnit : null;
          let toUnit = available.includes(next.toUnit) ? next.toUnit : null;
          const remembered = next.lastPairs[next.category] || DEFAULT_PAIRS[next.category];
          if (!fromUnit) fromUnit = available.includes(remembered.fromUnit) ? remembered.fromUnit : DEFAULT_PAIRS[next.category].fromUnit;
          if (!toUnit) toUnit = available.includes(remembered.toUnit) ? remembered.toUnit : DEFAULT_PAIRS[next.category].toUnit;
          next.fromUnit = fromUnit;
          next.toUnit = toUnit;
          next.lastPairs[next.category] = { fromUnit: next.fromUnit, toUnit: next.toUnit };
          return next;
        }

        function assignState(next) {
          const sanitized = sanitizeState(Object.assign({}, state, next));
          Object.keys(sanitized).forEach((key) => {
            state[key] = sanitized[key];
          });
        }

        function restoreFromQuery() {
          const params = new URLSearchParams(window.location.search || "");
          if (![...params.keys()].length) return false;
          assignState({
            category: params.get("cat") || state.category,
            inputValue: params.has("v") ? params.get("v") : state.inputValue,
            fromUnit: params.get("from") || state.fromUnit,
            toUnit: params.get("to") || state.toUnit,
            cropPreset: params.get("crop") || state.cropPreset,
            testWeightLbPerBushel: params.has("tw") ? params.get("tw") : state.testWeightLbPerBushel,
            localeMode: params.get("loc") || state.localeMode,
            roundMode: params.get("rm") || state.roundMode,
            significantDigits: params.get("sig") || state.significantDigits,
            fixedDecimals: params.get("dec") || state.fixedDecimals,
            gallonType: params.get("gal") || state.gallonType,
            alwaysShowFormula: params.get("sf") === "1",
            showJpCustomUnits: params.get("jp") === "1" || state.showJpCustomUnits,
            favoritesOnly: params.get("fo") === "1",
            mobileTab: params.get("tab") || state.mobileTab
          });
          return true;
        }

        function compute() {
          const errors = [];
          const warnings = [];
          const parsedValue = parseFlexibleNumber(state.inputValue);
          if (state.inputValue.trim() === "") {
            return {
              valid: false,
              needsInput: true,
              status: pageRaw.tool.status.needInput,
              errors,
              warnings
            };
          }
          if (!Number.isFinite(parsedValue) || parsedValue < 0) {
            errors.push(pageRaw.tool.errors.valueInvalid);
          }
          const needsFactor = state.category === "crop" && (state.fromUnit === "bushel_acre" || state.toUnit === "bushel_acre");
          const factorWeight = parseFlexibleNumber(state.testWeightLbPerBushel);
          if (needsFactor) {
            if (state.testWeightLbPerBushel.trim() === "") {
              errors.push(pageRaw.tool.errors.testWeightRequired);
            } else if (!Number.isFinite(factorWeight) || factorWeight <= 0) {
              errors.push(pageRaw.tool.errors.testWeightInvalid);
            } else if (factorWeight < 20 || factorWeight > 100) {
              warnings.push(pageRaw.tool.warnings.factor);
            }
          }

          const fromFactor = factorForUnit(state.fromUnit, state);
          const toFactor = factorForUnit(state.toUnit, state);
          if (!Number.isFinite(fromFactor) || !Number.isFinite(toFactor) || fromFactor <= 0 || toFactor <= 0) {
            if (!errors.length) {
              errors.push(needsFactor ? pageRaw.tool.errors.testWeightRequired : pageRaw.tool.errors.valueInvalid);
            }
          }

          if (errors.length) {
            return {
              valid: false,
              needsInput: false,
              status: needsFactor ? pageRaw.tool.status.needFactor : pageRaw.tool.status.invalid,
              errors,
              warnings
            };
          }

          const normalized = parsedValue * fromFactor;
          const resultValue = normalized / toFactor;
          const factor = fromFactor / toFactor;
          const inverseFactor = toFactor / fromFactor;

          if (state.category === "area" && normalized / 10000 > 100000) {
            warnings.push(pageRaw.tool.warnings.area);
          }
          if (state.category === "spray" && normalized > 10000) {
            warnings.push(pageRaw.tool.warnings.spray);
          }
          if ((state.category === "fertilizer" || state.category === "crop") && normalized > 5000) {
            warnings.push(pageRaw.tool.warnings.fertilizer);
          }

          const formulaText = [
            formatPlainNumber(parsedValue, 10),
            " × ",
            formatPlainNumber(factor, 10),
            " = ",
            formatPlainNumber(resultValue, 10),
            " acre"
          ].join("");

          return {
            valid: true,
            needsInput: false,
            status: pageRaw.tool.status.ready,
            inputValue: parsedValue,
            resultValue,
            normalized,
            factor,
            inverseFactor,
            errors,
            warnings,
            formulaText,
            coefficientLines: []
          };
        }

        window.history.replaceState(
          null,
          "",
          "https://example.com/?cat=invalid&v=1&from=broken&to=broken&tw=oops"
        );

        restoreFromQuery();
        const computed = compute();
        document.getElementById("out").textContent = trace.join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "1:number:true|2.471053814671653:number:true|2.471053814671653:number:true",
    )?;
    Ok(())
}
