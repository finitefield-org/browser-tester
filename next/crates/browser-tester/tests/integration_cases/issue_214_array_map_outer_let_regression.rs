use browser_tester::Harness;

#[test]
fn issue_214_array_map_callback_mutations_update_outer_let_bindings() -> browser_tester::Result<()>
{
    let html = r#"
      <div id="calculated">0</div>
      <div id="errors">0</div>
      <div id="preview"></div>
      <script>
        const rows = [
          { ok: true, label: "valid" },
          { ok: false, label: "invalid" }
        ];
        let calculatedCount = 0;
        let errorCount = 0;
        const previewRows = rows.map((row) => {
          const notes = [];
          if (!row.ok) {
            notes.push("bad");
            errorCount += 1;
            return { label: row.label, notes };
          }
          calculatedCount += 1;
          return { label: row.label, notes };
        });
        document.getElementById("calculated").textContent = String(calculatedCount);
        document.getElementById("errors").textContent = String(errorCount);
        document.getElementById("preview").textContent = previewRows
          .map((row) => row.label + ":" + row.notes.join(";"))
          .join("|");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#calculated", "1")?;
    harness.assert_text("#errors", "1")?;
    harness.assert_text("#preview", "valid:|invalid:bad")?;
    Ok(())
}
