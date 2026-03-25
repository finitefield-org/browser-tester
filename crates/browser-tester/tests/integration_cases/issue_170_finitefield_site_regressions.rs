use browser_tester::Harness;

#[test]
fn issue_170_dom_parser_supports_svg_mime() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
        circle.setAttribute("cx", "5");
        circle.setAttribute("cy", "5");
        circle.setAttribute("r", "4");
        svg.appendChild(circle);
        document.getElementById("out").textContent = [
          String(svg.localName),
          String(svg.namespaceURI),
          svg.outerHTML.indexOf("<circle") >= 0 ? "true" : "false"
        ].join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "svg|http://www.w3.org/2000/svg|true")?;
    Ok(())
}
