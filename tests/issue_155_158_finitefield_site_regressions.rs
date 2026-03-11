use browser_tester::Harness;

#[test]
fn issue_155_closest_accepts_selector_variable_in_if_condition() -> browser_tester::Result<()> {
    let html = r#"
      <div class="btn-wrap">
        <span id="child">child</span>
      </div>
      <p id="out"></p>
      <script>
        const child = document.getElementById("child");
        const buttonWrapSelector = ".btn-wrap, .button-block";
        if (child.closest(buttonWrapSelector)) {
          document.getElementById("out").textContent = "matched";
        }
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "matched")?;
    Ok(())
}

#[test]
fn issue_158_closest_accepts_selector_variable_in_expression_position()
-> browser_tester::Result<()> {
    let html = r#"
      <section class="card">
        <button id="child">open</button>
      </section>
      <p id="out"></p>
      <script>
        const child = document.getElementById("child");
        const selector = ".card";
        const matched = child.closest(selector);
        document.getElementById("out").textContent = matched ? matched.tagName : "none";
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "SECTION")?;
    Ok(())
}
