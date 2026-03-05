use browser_tester::Harness;

#[test]
fn issue_151_map_delete_with_extra_argument_is_not_dispatched_as_form_data()
-> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const pickMap = new Map();
        pickMap.set('sku-1', 12);
        pickMap.set('sku-2', 5);
        const deleted = pickMap.delete('sku-1', 'extra');
        const missing = pickMap.delete('missing', 'extra');
        document.getElementById('out').textContent = [
          String(deleted),
          String(missing),
          String(pickMap.size),
          String(pickMap.get('sku-2')),
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|false|1|5")?;
    Ok(())
}

#[test]
fn issue_151_map_has_with_extra_argument_uses_map_semantics()
-> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const pickMap = new Map();
        pickMap.set('sku-1', 12);
        const hasSku = pickMap.has('sku-1', 'extra');
        const hasMissing = pickMap.has('missing', 'extra');
        document.getElementById('out').textContent = [
          String(hasSku),
          String(hasMissing),
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|false")?;
    Ok(())
}

#[test]
fn issue_151_pickmap_get_or_fallback_does_not_overwrite_map_binding() -> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const pickMap = new Map();
        pickMap.set('W1', { fixed: 0, unit: 2 });
        const getUnitCost = (warehouse) => {
          const pick = pickMap.get(warehouse) || { fixed: 0, unit: 0 };
          return pick.unit;
        };
        const costMissing = getUnitCost('W9');
        const costExisting = getUnitCost('W1');
        const mapValue = pickMap.get('W1');
        document.getElementById('out').textContent = [
          String(costMissing),
          String(costExisting),
          mapValue ? String(mapValue.unit) : 'none',
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "0|2|2")?;
    Ok(())
}

#[test]
fn issue_151_pickmap_get_or_object_literal_fallback_keeps_map() -> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const pickMap = new Map();
        pickMap.set('W1', { fixed: 0, unit: 2 });
        const fallback = pickMap.get('W9') || { fixed: 0, unit: 0 };
        const mapValue = pickMap.get('W1');
        document.getElementById('out').textContent = [
          String(fallback.unit),
          mapValue ? String(mapValue.unit) : 'none',
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "0|2")?;
    Ok(())
}

#[test]
fn issue_151_nested_const_shadow_named_pick_does_not_overwrite_outer_pick() -> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const pick = new Map();
        pick.set('W1', { unit: 2 });
        const run = () => {
          const inner = () => {
            const pick = { fixed: 0, unit: 0 };
            return pick.unit;
          };
          inner();
          const outer = pick.get('W1');
          return outer ? String(outer.unit) : 'none';
        };
        document.getElementById('out').textContent = run();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "2")?;
    Ok(())
}

#[test]
fn issue_151_minimal_buildplan_pick_map_not_clobbered() -> browser_tester::Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const choose = (arr, k) => {
          const out = [];
          function build(start, cur) {
            if (cur.length === k) {
              out.push([...cur]);
              return;
            }
            for (let i = start; i < arr.length; i += 1) {
              cur.push(arr[i]);
              build(i + 1, cur);
              cur.pop();
            }
          }
          build(0, []);
          return out;
        };

        const runGreedy = ({ subset, pickMap }) => {
          const usage = new Map();
          subset.forEach((w) => usage.set(w, 0));
          const getUnitCost = (warehouse) => {
            const pick = pickMap.get(warehouse) || { fixed: 0, unit: 0 };
            return pick.unit;
          };
          subset.forEach((warehouse) => {
            const pick = pickMap.get(warehouse) || { fixed: 0, unit: 0 };
            usage.set(warehouse, (usage.get(warehouse) || 0) + pick.unit + getUnitCost(warehouse));
          });
          const usedWarehouses = subset.filter((w) => (usage.get(w) || 0) >= 0);
          const breakdown = usedWarehouses.map((warehouse) => {
            const pick = pickMap.get(warehouse) || { fixed: 0, unit: 0 };
            return pick.unit;
          });
          return breakdown.length;
        };

        const buildPlan = () => {
          const pick = new Map();
          pick.set('W1', { fixed: 0, unit: 1 });
          const allCandidates = [];
          const combos = choose(['W1', 'W2'], 1);
          combos.forEach((subset) => {
            allCandidates.push(runGreedy({ subset, pickMap: pick }));
          });
          const mapValue = pick.get('W1');
          return [
            String(allCandidates.length),
            mapValue ? String(mapValue.unit) : 'none',
          ].join('|');
        };

        document.getElementById('out').textContent = buildPlan();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "2|1")?;
    Ok(())
}

#[test]
fn issue_151_map_get_property_access_is_not_treated_as_form_data_call() -> browser_tester::Result<()>
{
    let html = r#"
      <p id='out'></p>
      <script>
        const pickMap = new Map();
        pickMap.set('W1', { unit: 2 });
        const getter = pickMap.get;
        document.getElementById('out').textContent = String(typeof getter);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "function")?;
    Ok(())
}
