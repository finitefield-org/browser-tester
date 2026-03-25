use browser_tester::Harness;

#[test]
fn nested_helper_call_in_return_expression_keeps_outer_return_value() -> browser_tester::Result<()>
{
    let html = r##"
      <div id="out"></div>
      <script>
        (() => {
          function renderLabel(label) {
            return '<div class="field">' + String(label || '') + '</div>';
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
            document.getElementById("grid").innerHTML =
              '<div class="field">' +
                '<label class="field-label" for="eoq-calculator-map-holdingRate">Holding rate</label>' +
                '<select id="eoq-calculator-map-holdingRate" data-map-key="holdingRate">' +
                  '<option value="-1">Unused</option>' +
                  '<option value="0">#1 annual_demand</option>' +
                  '<option value="1" selected>#2 order_cost</option>' +
                  '<option value="2">#3 alt_rate</option>' +
                  '<option value="3">#4 unit_cost</option>' +
                '</select>' +
              '</div>';
          }

          renderBatchMappingGrid();
        })();
      </script>
    "##;

    let harness = Harness::from_html(html)?;
    harness.assert_exists("#eoq-calculator-map-holdingRate")?;
    harness.assert_value("#eoq-calculator-map-holdingRate", "1")?;

    let snippet = harness.dump_dom("#grid")?;
    assert!(
        snippet.contains("<select")
            && snippet.contains("id=\"eoq-calculator-map-holdingRate\"")
            && snippet.contains("<option value=\"2\">#3 alt_rate</option>"),
        "expected rendered select markup in mapping grid; actual: {snippet}"
    );
    Ok(())
}
