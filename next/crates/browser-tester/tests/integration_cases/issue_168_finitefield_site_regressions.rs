use browser_tester::Harness;

#[test]
fn issue_168_object_from_entries_supports_page_init_lookup_tables() -> browser_tester::Result<()> {
    let html = r#"
      <pre id="out"></pre>
      <script>
        const kanaPairs = [
          ["full", "アイウ"],
          ["half", "ｱｲｳ"]
        ];
        const normalized = Object.fromEntries(
          kanaPairs.map(([key, value]) => [key, value.slice(0, 2)])
        );
        const aliases = Object.fromEntries(
          new Map([
            ["zenkaku", normalized.full],
            ["hankaku", normalized.half]
          ])
        );

        document.getElementById("out").textContent =
          aliases.zenkaku + "|" + aliases.hankaku + "|" + Object.keys(aliases).join(",");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "アイ|ｱｲ|zenkaku,hankaku")?;
    Ok(())
}
