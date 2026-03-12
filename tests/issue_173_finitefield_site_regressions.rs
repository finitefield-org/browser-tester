use browser_tester::Harness;

#[test]
fn issue_173_swedish_collation_orders_a_ring_before_a_umlaut() -> browser_tester::Result<()> {
    let html = r#"
      <pre id="out"></pre>
      <script>
        const collator = new Intl.Collator("sv", {
          usage: "sort",
          sensitivity: "variant",
        });
        const values = ["Öga", "Zebra", "Äpple", "Ål"];
        values.sort(collator.compare);
        document.getElementById("out").textContent =
          values.join(",") + "|" + String(collator.compare("Ål", "Äpple") < 0);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "Zebra,Ål,Äpple,Öga|true")?;
    Ok(())
}
