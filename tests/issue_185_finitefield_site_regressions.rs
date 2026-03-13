use browser_tester::Harness;

#[test]
fn issue_185_inline_object_literal_computed_lookup_returns_selected_value(
) -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const backgroundCss = {
          checker: "background:#f8fafc;",
          dark: "background:#0f172a;"
        }["dark"] || "background:#ffffff;";

        const zoomScale = {
          fit: 1,
          "200": 2
        }["200"] || 1;

        document.getElementById("out").textContent =
          backgroundCss + "|" + String(zoomScale);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "background:#0f172a;|2")?;
    Ok(())
}

#[test]
fn issue_185_inline_object_literal_lookup_survives_template_interpolation(
) -> browser_tester::Result<()> {
    let html = r#"
      <pre id="out"></pre>
      <script>
        const srcdoc = `
          <style>
            body { overflow: auto; ${
              {
                checker: "background:#f8fafc;",
                dark: "background:#0f172a;"
              }["dark"] || "background:#ffffff;"
            } }
            svg { transform: scale(${
              {
                fit: 1,
                "200": 2
              }["200"] || 1
            }); }
          </style>
        `;
        document.getElementById("out").textContent =
          srcdoc.includes("background:#0f172a;") &&
          srcdoc.includes("transform: scale(2);")
            ? "ok"
            : srcdoc;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "ok")?;
    Ok(())
}
