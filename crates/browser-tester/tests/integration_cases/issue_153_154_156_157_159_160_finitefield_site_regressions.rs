use browser_tester::Harness;

#[test]
fn issue_153_dynamic_index_compound_assignment_is_supported() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const values = [1, 2];
        const index = 1;
        values[index] += 3;
        document.getElementById("out").textContent = String(values[1]);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "5")?;
    Ok(())
}

#[test]
fn issue_154_function_listener_binds_this_to_current_target() -> browser_tester::Result<()> {
    let html = r#"
      <button id="button" data-value="ok">go</button>
      <div id="out"></div>
      <script>
        const button = document.getElementById("button");
        const out = document.getElementById("out");
        button.addEventListener("click", function () {
          out.textContent = this.getAttribute("data-value");
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#button")?;
    harness.assert_text("#out", "ok")?;
    Ok(())
}

#[test]
fn issue_156_request_animation_frame_ignores_extra_arguments() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const out = document.getElementById("out");
        window.requestAnimationFrame(function () {
          out.textContent = "done";
        }, 0);
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.assert_text("#out", "")?;
    harness.advance_time(16)?;
    harness.assert_text("#out", "done")?;
    Ok(())
}

#[test]
fn issue_157_date_to_locale_date_string_is_available() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const date = new Date("2024-02-03T00:00:00Z");
        document.getElementById("out").textContent = date.toLocaleDateString("en-US");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "2/3/2024")?;
    Ok(())
}

#[test]
fn issue_159_assignment_through_call_result_is_supported() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const warnings = new Map([["a", { overlap: false }]]);
        warnings.get("a").overlap = true;
        document.getElementById("out").textContent = String(warnings.get("a").overlap);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true")?;
    Ok(())
}

#[test]
fn issue_160_array_flatmap_is_supported() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const values = ["north", "south"];
        const result = values.flatMap((value) => [value]);
        document.getElementById("out").textContent = result.join(",");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "north,south")?;
    Ok(())
}
