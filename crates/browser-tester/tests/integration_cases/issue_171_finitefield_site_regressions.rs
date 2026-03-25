use browser_tester::Harness;

#[test]
fn issue_171_intl_collator_numeric_option_orders_digit_runs_naturally() -> browser_tester::Result<()>
{
    let html = r#"
      <pre id="out"></pre>
      <script>
        const values = ["item 10", "item 2", "item 1"];
        const collator = new Intl.Collator("en", {
          usage: "sort",
          numeric: true,
          sensitivity: "variant",
        });

        const asc = values.slice().sort(collator.compare).join(",");
        const desc = values.slice().sort((left, right) => collator.compare(right, left)).join(",");
        const zeroPadded = collator.compare("item 02", "item 2");
        const numeric = String(collator.resolvedOptions().numeric);

        document.getElementById("out").textContent =
          asc + "|" + desc + "|" + zeroPadded + "|" + numeric;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "item 1,item 2,item 10|item 10,item 2,item 1|0|true")?;
    Ok(())
}
