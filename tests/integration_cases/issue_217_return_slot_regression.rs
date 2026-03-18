use browser_tester::Harness;

#[test]
fn nested_helper_call_in_return_expression_keeps_outer_return_value() -> browser_tester::Result<()>
{
    let html = r##"
      <div id="out"></div>
      <script>
        (() => {
          function renderLabel(label) {
            return `<div class="field">${escapeHtml(label)}</div>`;
          }

          function escapeHtml(value) {
            return String(value || "")
              .replace(/&/g, "&amp;")
              .replace(/</g, "&lt;")
              .replace(/>/g, "&gt;")
              .replace(/"/g, "&quot;")
              .replace(/'/g, "&#39;");
          }

          document.getElementById("out").textContent = renderLabel("Holding rate");
        })();
      </script>
    "##;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "<div class=\"field\">Holding rate</div>")?;
    Ok(())
}

#[test]
fn issue_217_batch_mapping_grid_keeps_select_markup_with_late_helper_declaration()
-> browser_tester::Result<()> {
    let html = r##"
      <div id="grid"></div>
      <script>
        (() => {
          function renderBatchMappingGrid() {
            const labels = [
              "#1 annual_demand",
              "#2 order_cost",
              "#3 alt_rate",
              "#4 unit_cost",
            ];
            const mapping = {
              annualDemand: 0,
              orderCost: 1,
              holdingRate: -1,
              unitCost: 3,
            };
            const mappingFields = [
              ["annualDemand", "Annual demand"],
              ["orderCost", "Order cost"],
              ["holdingRate", "Holding rate"],
              ["unitCost", "Unit cost"],
            ];

            document.getElementById("grid").innerHTML = mappingFields.map(([key, label]) => {
              const options = [`<option value="-1">${escapeHtml("Unused")}</option>`]
                .concat(labels.map((header, index) => `<option value="${index}" ${mapping[key] === index ? "selected" : ""}>${escapeHtml(header)}</option>`))
                .join("");
              return `<div class="field">
                <label class="field-label" for="eoq-calculator-map-${key}">${escapeHtml(label)}</label>
                <select id="eoq-calculator-map-${key}" data-map-key="${key}">${options}</select>
              </div>`;
            }).join("");
          }

          function escapeHtml(value) {
            return String(value || "")
              .replace(/&/g, "&amp;")
              .replace(/</g, "&lt;")
              .replace(/>/g, "&gt;")
              .replace(/"/g, "&quot;")
              .replace(/'/g, "&#39;");
          }

          renderBatchMappingGrid();
        })();
      </script>
    "##;

    let harness = Harness::from_html(html)?;
    harness.assert_exists("#eoq-calculator-map-holdingRate")?;
    harness.assert_value("#eoq-calculator-map-orderCost", "1")?;

    let snippet = harness.dump_dom("#grid")?;
    assert!(
        snippet.contains("<select")
            && snippet.contains("id=\"eoq-calculator-map-holdingRate\"")
            && snippet.contains("<option value=\"2\">#3 alt_rate</option>"),
        "expected rendered select markup in mapping grid; actual: {snippet}"
    );
    Ok(())
}
