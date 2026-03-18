use browser_tester::Harness;

#[test]
fn issue_212_nested_pending_helper_in_listener_keeps_outer_state_capture()
-> browser_tester::Result<()> {
    let html = r#"
      <input id="field" />
      <div id="out"></div>
      <script>
        (() => {
          const state = {
            currency: "JPY",
            decimalOverride: "auto",
            cost: "",
            adoptedPrice: 1200,
          };
          const currencyMap = new Map([
            ["JPY", { code: "JPY", locale: "ja-JP", decimals: 0 }],
          ]);

          function getDecimals() {
            if (state.decimalOverride !== "auto") {
              return Number(state.decimalOverride);
            }
            const meta = currencyMap.get(state.currency);
            return meta && meta.decimals != null ? meta.decimals : 2;
          }

          function formatMoney(value) {
            const meta = currencyMap.get(state.currency) || {
              code: state.currency,
              locale: "en-US",
              decimals: 2,
            };
            const digits = getDecimals();
            return new Intl.NumberFormat(meta.locale, {
              style: "currency",
              currency: meta.code,
              minimumFractionDigits: digits,
              maximumFractionDigits: digits,
            }).format(value);
          }

          function renderSingleResult() {
            document.getElementById("out").textContent = formatMoney(1200);
          }

          function render() {
            renderSingleResult();
          }

          function clearAdoptedPrice() {
            state.adoptedPrice = null;
          }

          function bindTextInput(node, key) {
            node.addEventListener("input", () => {
              state[key] = node.value;
              clearAdoptedPrice();
              render();
            });
          }

          bindTextInput(document.getElementById("field"), "cost");
          render();
        })();
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.assert_text("#out", "￥1,200")?;
    harness.type_text("#field", "1200")?;
    harness.assert_text("#out", "￥1,200")?;
    Ok(())
}

#[test]
fn issue_212_host_like_render_chain_in_listener_keeps_outer_state_capture()
-> browser_tester::Result<()> {
    let html = r#"
      <input id="field" />
      <p id="title"></p>
      <p id="help"></p>
      <p id="out"></p>
      <script>
        (() => {
          const page = {
            mode: {
              costToPrice: { Title: "Cost", Description: "Desc", ResultNote: "" },
              priceToMargin: { Title: "Price", Description: "Desc", ResultNote: "" },
            },
            fields: {
              targetMargin: "Target",
              targetMarginOptional: "Optional",
              price: "Price",
              comparePrice: "Compare",
            },
            results: {
              recommendedPrice: "Recommended",
              theoreticalNote: "Theoretical",
              roundedNote: "Rounded",
              noValue: "-",
              margin: "Margin",
            },
          };
          const currencyMap = new Map([
            ["JPY", { code: "JPY", locale: "ja-JP", decimals: 0 }],
          ]);
          const MODE_COST_TO_PRICE = "cost_to_price";
          const MODE_PRICE_TO_MARGIN = "price_to_margin";
          const el = {
            targetMarginLabel: document.getElementById("help"),
            priceLabel: document.getElementById("help"),
            resultModeNote: document.getElementById("help"),
            resultPrimaryLabel: document.getElementById("title"),
            resultPrimaryValue: document.getElementById("out"),
            resultPrimarySub: document.getElementById("help"),
          };
          const state = {
            mode: MODE_COST_TO_PRICE,
            currency: "JPY",
            decimalOverride: "auto",
            adoptedPrice: 1200,
            cost: "",
          };

          function getDecimals() {
            if (state.decimalOverride !== "auto") {
              return Number(state.decimalOverride);
            }
            const meta = currencyMap.get(state.currency);
            return meta && meta.decimals != null ? meta.decimals : 2;
          }

          function formatMoney(value) {
            if (!Number.isFinite(value)) return page.results?.noValue || "-";
            const meta = currencyMap.get(state.currency) || {
              code: state.currency,
              locale: "en-US",
              decimals: 2,
            };
            const digits = getDecimals();
            return new Intl.NumberFormat(meta.locale, {
              style: "currency",
              currency: meta.code,
              minimumFractionDigits: digits,
              maximumFractionDigits: digits,
            }).format(value);
          }

          function formatPlain(value, digits) {
            if (!Number.isFinite(value)) return page.results?.noValue || "-";
            return new Intl.NumberFormat(currencyMap.get(state.currency)?.locale || "en-US", {
              minimumFractionDigits: digits,
              maximumFractionDigits: digits,
            }).format(value);
          }

          function formatPercent(ratio) {
            if (!Number.isFinite(ratio)) return page.results?.noValue || "-";
            return `${formatPlain(ratio * 100, 1)}%`;
          }

          function clearAdoptedPrice() {
            state.adoptedPrice = null;
          }

          function getModeMeta() {
            return state.mode === MODE_PRICE_TO_MARGIN
              ? page.mode?.priceToMargin
              : page.mode?.costToPrice;
          }

          function setText(node, value) {
            if (node) node.textContent = value;
          }

          function renderSingleResult(result) {
            const modeMeta = getModeMeta();
            setText(el.resultPrimaryLabel, modeMeta?.Title || "");
            setText(
              el.targetMarginLabel,
              state.mode === MODE_PRICE_TO_MARGIN ? page.fields.targetMarginOptional : page.fields.targetMargin
            );
            setText(
              el.priceLabel,
              state.mode === MODE_PRICE_TO_MARGIN ? page.fields.price : page.fields.comparePrice
            );
            setText(el.resultModeNote, modeMeta?.ResultNote || "");
            setText(el.resultPrimaryLabel, page.results.recommendedPrice);
            setText(el.resultPrimaryValue, formatMoney(result.adoptedPrice));
            setText(
              el.resultPrimarySub,
              `${page.results.margin}: ${formatPercent(result.metrics.margin)}`
            );
          }

          function render() {
            renderSingleResult({
              adoptedPrice: Number.isFinite(state.adoptedPrice) ? state.adoptedPrice : 1200,
              metrics: { margin: 0.2 },
            });
          }

          function bindTextInput(node, key) {
            node.addEventListener("input", () => {
              state[key] = node.value;
              clearAdoptedPrice();
              render();
            });
          }

          bindTextInput(document.getElementById("field"), "cost");
          render();
        })();
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.assert_text("#out", "￥1,200")?;
    harness.type_text("#field", "1200")?;
    harness.assert_text("#title", "Recommended")?;
    harness.assert_text("#out", "￥1,200")?;
    Ok(())
}

#[test]
fn issue_212_bulk_mapping_listener_can_replace_outer_const_state_array()
-> browser_tester::Result<()> {
    let html = r#"
      <select id="mapping">
        <option value="unused" selected>unused</option>
        <option value="price">price</option>
      </select>
      <div id="out"></div>
      <script>
        (() => {
          const state = {
            bulkMappings: ["unused", "cost"],
          };

          function render() {
            document.getElementById("out").textContent = state.bulkMappings.join(",");
          }

          document.getElementById("mapping").addEventListener("change", (event) => {
            const target = event.target;
            if (!(target instanceof HTMLSelectElement)) return;
            const nextRole = target.value;
            state.bulkMappings = state.bulkMappings.map((value, currentIndex) => {
              if (currentIndex === 0) return nextRole;
              if (value === nextRole && nextRole !== "unused") return "unused";
              return value;
            });
            render();
          });

          render();
        })();
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.assert_text("#out", "unused,cost")?;
    harness.set_select_value("#mapping", "price")?;
    harness.assert_text("#out", "price,cost")?;
    Ok(())
}

#[test]
fn issue_212_bulk_callback_can_update_outer_let_counter_inside_listener_render()
-> browser_tester::Result<()> {
    let html = r#"
      <textarea id="bulk"></textarea>
      <div id="out"></div>
      <script>
        (() => {
          const state = {
            bulkText: "",
          };

          function parseRows(text) {
            return text
              .split(/\r?\n/)
              .filter((line) => line.trim() !== "")
              .map((line) => line.split(","));
          }

          function computeBulkResult() {
            const rows = parseRows(state.bulkText);
            let calculated = 0;
            const resultRows = rows.map((row) => {
              if (row[0]) {
                calculated += 1;
              }
              return row[0] || "";
            });
            return `${calculated}:${resultRows.join("|")}`;
          }

          function render() {
            document.getElementById("out").textContent = computeBulkResult();
          }

          document.getElementById("bulk").addEventListener("input", () => {
            state.bulkText = document.getElementById("bulk").value;
            render();
          });

          render();
        })();
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.type_text("#bulk", "A,1\nB,2")?;
    harness.assert_text("#out", "2:A|B")?;
    Ok(())
}
