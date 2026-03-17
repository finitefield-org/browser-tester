use browser_tester::Harness;

#[test]
fn issue_218_bulk_mapping_and_summary_callbacks_keep_outer_bindings_isolated_and_accumulating()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="mapping"></div>
      <div id="summary"></div>
      <div id="preview"></div>
      <script>
        (() => {
          function inferMappings(headers) {
            const roles = ["name", "cost", "price", "extra", "target"];
            const mappings = new Array(headers.length).fill("unused");
            const normalizedHeaders = headers.map((value) => String(value || "").toLowerCase());

            roles.forEach((role) => {
              const index = normalizedHeaders.indexOf(role);
              if (index >= 0) {
                mappings[index] = role;
              }
            });

            return mappings;
          }

          function computeBulkResult(rows) {
            let calculated = 0;
            const resultRows = rows.map((row, rowIndex) => {
              const status = row.price > 0 ? "calculated" : "missing";
              if (status === "calculated") {
                calculated += 1;
              }
              return {
                label: row.name || `row-${rowIndex + 1}`,
                status,
              };
            });

            return {
              summary: `total=${resultRows.length};calculated=${calculated}`,
              preview: resultRows.map((row) => `${row.label}:${row.status}`).join("|"),
            };
          }

          const mapping = inferMappings(["name", "cost", "price", "extra", "target"]);
          const result = computeBulkResult([
            { name: "SKU-A", cost: 1200, price: 2200 },
            { name: "SKU-B", cost: 900, price: 1680 },
            { name: "SKU-C", cost: 600, price: 990 }
          ]);

          document.getElementById("mapping").textContent = mapping.join("|");
          document.getElementById("summary").textContent = result.summary;
          document.getElementById("preview").textContent = result.preview;
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#mapping", "name|cost|price|extra|target")?;
    harness.assert_text("#summary", "total=3;calculated=3")?;
    harness.assert_text(
        "#preview",
        "SKU-A:calculated|SKU-B:calculated|SKU-C:calculated",
    )?;
    Ok(())
}

#[test]
fn issue_218_bulk_result_map_accumulates_mixed_price_and_target_rows()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="summary"></div>
      <div id="preview"></div>
      <script>
        (() => {
          const page = {
            results: {
              noValue: "-",
            },
            bulk: {
              statusCalculated: "calculated",
              statusNotCalculated: "not-calculated",
              summary: "total={total};calculated={calculated}",
            },
          };

          function formatMessage(template, values) {
            return String(template || "").replace(/\{(\w+)\}/g, (_, key) => {
              return Object.prototype.hasOwnProperty.call(values, key) ? values[key] : "";
            });
          }

          function valueOrZero(value) {
            const numeric = Number(value);
            return Number.isFinite(numeric) ? numeric : 0;
          }

          function parseLooseNumber(value) {
            if (value == null || value === "") return null;
            const numeric = Number(value);
            return Number.isFinite(numeric) ? numeric : null;
          }

          function calculateMetrics(price, baseCost, feeRate) {
            const grossProfit = price * (1 - feeRate) - baseCost;
            const margin = price > 0 ? grossProfit / price : Number.NaN;
            const markup = baseCost > 0 ? grossProfit / baseCost : Number.NaN;
            return { grossProfit, margin, markup };
          }

          function formatMoney(value) {
            return value == null || !Number.isFinite(value) ? "-" : value.toFixed(0);
          }

          function formatPlain(value, digits) {
            return Number(value).toFixed(digits);
          }

          function formatPercent(value) {
            return `${(value * 100).toFixed(1)}%`;
          }

          function computeBulkResult(rows) {
            const feeRate = valueOrZero("3.6") / 100;
            const feeFixed = valueOrZero("10");
            const defaultTarget = parseLooseNumber("40");
            let calculated = 0;
            const resultRows = rows.map((row, rowIndex) => {
              const name = row.name || `row-${rowIndex + 1}`;
              const cost = parseLooseNumber(row.cost);
              const price = parseLooseNumber(row.price);
              const extra = valueOrZero(row.extra);
              const parsedTarget = parseLooseNumber(row.target);
              const target = parsedTarget == null ? defaultTarget : parsedTarget;

              const baseCost = cost + extra + feeFixed;
              let recommendedPrice = Number.NaN;
              let grossProfit = Number.NaN;
              let margin = Number.NaN;
              let markup = Number.NaN;
              let status = page.bulk.statusNotCalculated;

              if (price != null && Number.isFinite(price) && price > 0) {
                const metrics = calculateMetrics(price, baseCost, feeRate);
                grossProfit = metrics.grossProfit;
                margin = metrics.margin;
                markup = metrics.markup;
                status = page.bulk.statusCalculated;
                calculated += 1;
              }

              if (target != null && Number.isFinite(target) && target >= 0 && target < 100 && feeRate + target / 100 < 1) {
                recommendedPrice = baseCost / (1 - feeRate - target / 100);
                if (status !== page.bulk.statusCalculated) {
                  status = page.bulk.statusCalculated;
                  calculated += 1;
                }
              }

              return {
                name,
                cost: formatMoney(cost),
                price: price != null && Number.isFinite(price) && price > 0 ? formatMoney(price) : page.results.noValue,
                extra: formatMoney(extra),
                target: target == null ? page.results.noValue : `${formatPlain(target, 1)}%`,
                recommendedPrice: Number.isFinite(recommendedPrice) ? formatMoney(recommendedPrice) : page.results.noValue,
                grossProfit: Number.isFinite(grossProfit) ? formatMoney(grossProfit) : page.results.noValue,
                margin: Number.isFinite(margin) ? formatPercent(margin) : page.results.noValue,
                markup: Number.isFinite(markup) ? formatPercent(markup) : page.results.noValue,
                status,
              };
            });

            return {
              summary: formatMessage(page.bulk.summary, {
                total: String(resultRows.length),
                calculated: String(calculated),
              }),
              preview: resultRows.map((row) => `${row.name}:${row.status}:${row.recommendedPrice}`).join("|"),
            };
          }

          const result = computeBulkResult([
            { name: "定番A", cost: "1200", price: "1980", extra: "80", target: "40" },
            { name: "セットB", cost: "2400", price: "", extra: "230", target: "45" },
            { name: "SKU-C", cost: "600", price: "990", extra: "50", target: "" }
          ]);

          document.getElementById("summary").textContent = result.summary;
          document.getElementById("preview").textContent = result.preview;
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#summary", "total=3;calculated=3")?;
    harness.assert_text(
        "#preview",
        "定番A:calculated:2287|セットB:calculated:5136|SKU-C:calculated:1170",
    )?;
    Ok(())
}

#[test]
fn issue_218_nested_bulk_parser_helpers_keep_outer_rows_and_cells_live()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function parseDelimitedTable(text) {
            const rows = [];
            let row = [];
            let cell = "";
            let inQuotes = false;

            const pushCell = () => {
              row.push(cell);
              cell = "";
            };

            const pushRow = () => {
              if (row.length || cell) {
                pushCell();
              }
              const normalized = row.map((value) => value.trim());
              const hasAny = normalized.some((value) => value !== "");
              if (hasAny) rows.push(normalized);
              row = [];
            };

            const input = String(text || "");
            for (let index = 0; index < input.length; index += 1) {
              const char = input[index];
              const next = input[index + 1];
              if (char === "\"") {
                if (inQuotes && next === "\"") {
                  cell += "\"";
                  index += 1;
                } else {
                  inQuotes = !inQuotes;
                }
                continue;
              }
              if (!inQuotes && char === "\t") {
                pushCell();
                continue;
              }
              if (!inQuotes && (char === "\n" || char === "\r")) {
                if (char === "\r" && next === "\n") {
                  index += 1;
                }
                pushRow();
                continue;
              }
              cell += char;
            }
            pushRow();
            return rows;
          }

          const rows = parseDelimitedTable("商品名\t原価\t売価\n定番A\t1200\t1980\nSKU-C\t600\t990");
          document.getElementById("out").textContent = rows.map((row) => row.join("|")).join(" / ");
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text(
        "#out",
        "商品名|原価|売価 / 定番A|1200|1980 / SKU-C|600|990",
    )?;
    Ok(())
}

#[test]
fn issue_218_nested_helper_updates_stay_visible_after_direct_helper_call()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function buildRows() {
            const rows = [];
            let row = [];
            let cell = "A";

            const pushCell = () => {
              row.push(cell);
              cell = "";
            };

            const pushRow = () => {
              pushCell();
              rows.push(row.slice());
              row = [];
            };

            pushRow();
            return `${rows.length}:${rows[0] ? rows[0].join("|") : "-"}:${cell}:${row.length}`;
          }

          document.getElementById("out").textContent = buildRows();
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1:A::0")?;
    Ok(())
}

#[test]
fn issue_218_array_callback_counter_survives_plain_helper_calls()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function formatPlain(value) {
            return String(value);
          }

          function compute() {
            let calculated = 0;
            const labels = [1, 2, 3].map(() => {
              calculated += 1;
              return formatPlain(calculated);
            });
            return `${calculated}:${labels.join("|")}`;
          }

          document.getElementById("out").textContent = compute();
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "3:1|2|3")?;
    Ok(())
}

#[test]
fn issue_218_array_callback_updates_remain_visible_to_later_plain_helper_calls()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function formatPlain(value) {
            return String(value);
          }

          function compute() {
            let calculated = 0;
            const rows = [1, 2, 3].map(() => {
              calculated += 1;
              return "ok";
            });
            const summary = formatPlain(calculated);
            return `${calculated}:${summary}:${rows.length}`;
          }

          document.getElementById("out").textContent = compute();
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "3:3:3")?;
    Ok(())
}

#[test]
fn issue_218_helper_updates_are_visible_to_later_array_map_in_same_function()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function buildRow() {
            let row = [];
            let cell = " A ";

            const pushCell = () => {
              row.push(cell);
              cell = "";
            };

            if (row.length === 0) {
              pushCell();
            }

            const normalized = row.map((value) => value.trim());
            return `${normalized.join("|")}:${cell}`;
          }

          document.getElementById("out").textContent = buildRow();
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "A:")?;
    Ok(())
}

#[test]
fn issue_218_repeated_push_cell_updates_feed_parser_style_push_row()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function buildRows() {
            const rows = [];
            let row = [];
            let cell = "";

            const pushCell = () => {
              row.push(cell);
              cell = "";
            };

            const pushRow = () => {
              if (row.length || cell) {
                pushCell();
              }
              const normalized = row.map((value) => value.trim());
              const hasAny = normalized.some((value) => value !== "");
              if (hasAny) rows.push(normalized);
              row = [];
            };

            cell = " 商品名 ";
            pushCell();
            cell = " 原価 ";
            pushCell();
            cell = " 売価 ";
            pushRow();

            return rows.map((entry) => entry.join("|")).join(" / ");
          }

          document.getElementById("out").textContent = buildRows();
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "商品名|原価|売価")?;
    Ok(())
}

#[test]
fn issue_218_simple_loop_parser_keeps_push_cell_and_push_row_updates()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function parseSimple(text) {
            const rows = [];
            let row = [];
            let cell = "";

            const pushCell = () => {
              row.push(cell);
              cell = "";
            };

            const pushRow = () => {
              if (row.length || cell) {
                pushCell();
              }
              const normalized = row.map((value) => value.trim());
              const hasAny = normalized.some((value) => value !== "");
              if (hasAny) rows.push(normalized);
              row = [];
            };

            const input = String(text || "");
            for (let index = 0; index < input.length; index += 1) {
              const char = input[index];
              if (char === "\t") {
                pushCell();
                continue;
              }
              if (char === "\n") {
                pushRow();
                continue;
              }
              cell += char;
            }
            pushRow();
            return rows;
          }

          const rows = parseSimple("商品名\t原価\t売価\n定番A\t1200\t1980");
          document.getElementById("out").textContent = rows.map((entry) => entry.join("|")).join(" / ");
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "商品名|原価|売価 / 定番A|1200|1980")?;
    Ok(())
}

#[test]
fn issue_218_simple_loop_push_cell_updates_survive_across_continue_iterations()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        (() => {
          function parseCells(text) {
            let row = [];
            let cell = "";

            const pushCell = () => {
              row.push(cell);
              cell = "";
            };

            const input = String(text || "");
            for (let index = 0; index < input.length; index += 1) {
              const char = input[index];
              if (char === "\t") {
                pushCell();
                continue;
              }
              cell += char;
            }
            pushCell();
            return row.join("|");
          }

          document.getElementById("out").textContent = parseCells("商品名\t原価\t売価");
        })();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "商品名|原価|売価")?;
    Ok(())
}
