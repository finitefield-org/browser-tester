use browser_tester::Harness;

#[test]
fn ignores_json_ld_script_blocks_and_runs_executable_script() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result">init</div>
    <script type="application/ld+json">
      {"@context":"https://schema.org","@type":"FAQPage"}
    </script>
    <script>
      document.getElementById("result").textContent = "ok";
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn script_end_extractor_handles_regex_literals_with_quotes() -> browser_tester::Result<()> {
    let html = r##"
    <div id="result">init</div>
    <script>
      const sanitizer = /["]/g;
      document.getElementById("result").textContent = "ok";
    </script>
    "##;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn array_from_supports_nodelist_and_map_callback() -> browser_tester::Result<()> {
    let html = r#"
    <ul>
      <li class="item">A</li>
      <li class="item">B</li>
    </ul>
    <div id="result"></div>
    <script>
      const nodes = Array.from(document.querySelectorAll(".item"));
      const mapped = Array.from(nodes, (node, idx) => node.textContent + idx);
      document.getElementById("result").textContent = nodes.length + ":" + mapped.join(",");
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "2:A0,B1")?;
    Ok(())
}

#[test]
fn trailing_commas_in_literals_are_supported_without_allowing_sparse_entries() {
    let html = r#"
    <div id="result"></div>
    <script>
      const base = { a: 1, b: 2, };
      const merged = { ...base, c: 3, };
      const values = [merged.a, merged.b, merged.c,];
      document.getElementById("result").textContent = values.join("-");
    </script>
    "#;

    let harness = Harness::from_html(html).expect("trailing commas should be accepted");
    harness
        .assert_text("#result", "1-2-3")
        .expect("result text should match");

    let sparse = Harness::from_html("<script>const bad = [1,,2];</script>")
        .expect_err("sparse arrays should remain unsupported");
    match sparse {
        browser_tester::Error::ScriptParse(msg) => {
            assert!(msg.contains("array literal does not support empty elements"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn nested_object_path_access_on_runtime_objects_is_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const page = {
        input: {
          inventory: {
            defaultLotPrefix: "Lot"
          }
        }
      };
      document.getElementById("result").textContent = page.input.inventory.defaultLotPrefix;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "Lot")?;
    Ok(())
}

#[test]
fn function_declaration_can_be_called_before_its_definition() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const state = {
        lots: [createLotRow()],
      };

      function createLotRow(seed = {}) {
        return {
          name: seed.name || "lot-1",
        };
      }

      document.getElementById("result").textContent = String(state.lots.length);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "1")?;
    Ok(())
}
