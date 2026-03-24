use browser_tester::Harness;

#[test]
fn repeated_type_text_with_large_id_heavy_dom_and_history_storage_sync_completes()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="fixture">
        <div id="seed-1"><span id="seed-text-1">seed</span></div>
        <div id="seed-2"><span id="seed-text-2">seed</span></div>
        <div id="seed-3"><span id="seed-text-3">seed</span></div>
      </div>
      <input id="cargo-w" type="text" inputmode="decimal" />
      <input id="cargo-d" type="text" inputmode="decimal" />
      <input id="cargo-h" type="text" inputmode="decimal" />
      <p id="total"></p>
      <p id="status"></p>
      <script>
        const cargoW = document.getElementById("cargo-w");
        const cargoD = document.getElementById("cargo-d");
        const cargoH = document.getElementById("cargo-h");
        const total = document.getElementById("total");
        const status = document.getElementById("status");

        function sync() {
          let count = 0;
          if (cargoW.value) count += 1;
          if (cargoD.value) count += 1;
          if (cargoH.value) count += 1;
          total.textContent = String(count);
          window.history.replaceState(null, "", count ? "?count=" + count : window.location.pathname);
          window.localStorage.setItem("tool.fishery.boxLoading.lastState.v1", cargoW.value + "|" + cargoD.value + "|" + cargoH.value);
          status.textContent = String(count);
        }

        cargoW.addEventListener("input", sync);
        cargoD.addEventListener("input", sync);
        cargoH.addEventListener("input", sync);
        sync();
      </script>
    "#;

    let mut harness = Harness::from_html_with_url(
        "https://example.com/tools/fishery/box-loading-calculator/",
        html,
    )?;
    harness.type_text("#cargo-w", "47.2in")?;
    harness.type_text("#cargo-d", "35.4in")?;
    harness.type_text("#cargo-h", "31,5in")?;

    harness.assert_text("#total", "3")?;
    let summary = harness.dump_dom("#fixture")?;
    assert!(
        summary.contains("seed-1") && summary.contains("seed-2") && summary.contains("seed-3"),
        "expected heavy fixture seeds to remain present, got: {summary}"
    );
    let status = harness.dump_dom("#status")?;
    assert!(
        status.contains(">3<"),
        "expected count sync to complete during repeated typing, got: {status}"
    );
    Ok(())
}
