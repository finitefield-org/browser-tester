use super::*;

fn find_function<'a>(stmts: &'a [Stmt], target: &str) -> &'a ScriptHandler {
    stmts.iter()
        .find_map(|stmt| match stmt {
            Stmt::FunctionDecl { name, handler, .. } if name == target => Some(handler),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing function declaration: {target}"))
}

#[test]
fn page_like_format_plain_number_keeps_numeric_guards_and_precision_calls_in_ast() -> Result<()> {
    let stmts = test_support::parse_block_statements(
        r#"
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
        "#,
    )?;

    let handler = find_function(&stmts, "formatPlainNumber");
    let debug = format!("{handler:#?}");
    assert!(debug.contains("IsFinite"));
    assert!(debug.contains("ToExponential"));
    assert!(debug.contains("ToPrecision"));
    assert!(debug.contains("text"));
    Ok(())
}

#[test]
fn page_like_compute_keeps_trace_push_before_formula_text_in_ast() -> Result<()> {
    let stmts = test_support::parse_block_statements(
        r#"
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

            trace.push([
              "locals",
              String(parsedValue),
              String(factor),
              String(resultValue)
            ].join(":"));
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
        "#,
    )?;

    let handler = find_function(&stmts, "compute");
    let debug = format!("{handler:#?}");
    assert!(debug.contains("target: \"trace\""));
    assert!(debug.contains("name: \"formulaText\""));
    Ok(())
}
