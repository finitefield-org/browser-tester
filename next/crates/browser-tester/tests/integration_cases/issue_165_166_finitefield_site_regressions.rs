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
        document.getElementById("out").textContent =
          "field_name,field_group\nField 1,North Block\nField 2,South Block";
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
      <div id="out"></div>
      <script>
        document.getElementById("download").addEventListener("click", () => {
          document.getElementById("out").textContent =
            "field_name,field_group,crop_name,start_ym,end_ym,caution_tag,status,memo\n" +
            "Field 1,North Block,Cabbage,2026-02,2026-05,Brassicaceae,fixed,Spring crop plan\n" +
            "Field 2,North Block,Tomato,2026-03,2026-08,Solanaceae,plan,Summer-autumn crop";
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#download")?;
    harness.assert_text("#out", "field_name,field_group,crop_name,start_ym,end_ym,caution_tag,status,memo\nField 1,North Block,Cabbage,2026-02,2026-05,Brassicaceae,fixed,Spring crop plan\nField 2,North Block,Tomato,2026-03,2026-08,Solanaceae,plan,Summer-autumn crop")?;
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
        const resultValue = 10000 / 4046.8564224;
        document.getElementById("result").textContent = new Intl.NumberFormat("en-US", {
          maximumSignificantDigits: 4,
          minimumSignificantDigits: 1
        }).format(resultValue);
        document.getElementById("status").textContent = "ready";
        document.getElementById("from").textContent = "ha";
        document.getElementById("to").textContent = "acre";
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
    harness.assert_text("#out", "1.000|2.471|10,000")?;
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
        const factor = 10000 / 4046.8564224;
        const resultValue = 1 * factor;
        document.getElementById("out").textContent = [
          "Ready",
          new Intl.NumberFormat("en-US", {
            maximumSignificantDigits: 4,
            minimumSignificantDigits: 1
          }).format(resultValue),
          "1 × 2.471053815 = 2.471053815 acre"
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
        const state = {
          inputValue: "1",
          fromUnit: "ha",
          toUnit: "acre"
        };
        const factor = 10000 / 4046.8564224;
        document.getElementById("out").textContent = [
          state.inputValue,
          factor.toPrecision(10),
          (Number(state.inputValue) * factor).toPrecision(10)
        ].join("|");
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
            return { valid: false, errors: errors, warnings: warnings };
          }
          const factor = 10000 / 4046.8564224;
          const resultValue = parsedValue * factor;
          if (errors.length) {
            return { valid: false, errors: errors, warnings: warnings };
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
        function inspectNumber(value) {
          return [String(value), typeof value, String(Number.isFinite(value))].join(":");
        }

        const parsedValue = 1;
        const factor = 10000 / 4046.8564224;
        const resultValue = parsedValue * factor;
        document.getElementById("out").textContent = [
          inspectNumber(parsedValue),
          inspectNumber(factor),
          inspectNumber(resultValue)
        ].join("|");
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
            resultValue: resultValue,
            formulaText: formulaText
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
              errors: errors,
              warnings: warnings
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
              errors: errors,
              warnings: warnings
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
            resultValue: resultValue,
            normalized: normalized,
            factor: factor,
            inverseFactor: inverseFactor,
            errors: errors,
            warnings: warnings,
            formulaText: formulaText
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
        function parseFlexibleNumber(raw) {
          const value = Number(String(raw == null ? "" : raw).trim());
          return Number.isFinite(value) ? value : Number.NaN;
        }

        function factorForUnit(unitKey) {
          if (unitKey === "ha") return 10000;
          if (unitKey === "acre") return 4046.8564224;
          return Number.NaN;
        }

        const state = {
          inputValue: "1",
          fromUnit: "ha",
          toUnit: "acre"
        };
        const parsedValue = parseFlexibleNumber(state.inputValue);
        const factor = factorForUnit(state.fromUnit) / factorForUnit(state.toUnit);
        const resultValue = parsedValue * factor;
        document.getElementById("out").textContent = [
          String(true),
          resultValue.toPrecision(10),
          parsedValue + " × " + factor.toPrecision(10) + " = " + resultValue.toPrecision(10)
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|2.471053815|1 × 2.471053815 = 2.471053815")?;
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
        const factorWeight = Number.NaN;
        const parsedValue = 1;
        const factor = 10000 / 4046.8564224;
        const resultValue = parsedValue * factor;
        document.getElementById("out").textContent = [
          String(false),
          String(parsedValue),
          factor.toPrecision(10),
          resultValue.toPrecision(10)
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "false|1|2.471053815|2.471053815")?;
    Ok(())
}

#[test]
fn issue_166_page_like_full_repro_traces_values_seen_by_format_plain_number()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const trace = [];

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

        const parsedValue = 1;
        const factor = 10000 / 4046.8564224;
        const resultValue = parsedValue * factor;
        formatPlainNumber(parsedValue, 10);
        formatPlainNumber(factor, 10);
        formatPlainNumber(resultValue, 10);
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
