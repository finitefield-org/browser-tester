use super::*;

#[test]
fn issue_104_nested_helper_closure_reads_live_outer_let_bindings() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <pre id='out'></pre>
      <script>
        function parseDelimited(text, delimiter) {
          const rows = [];
          let row = [];
          let field = '';

          function pushField() {
            row.push(field);
            field = '';
          }

          function pushRow() {
            if (row.length === 0) return;
            rows.push(row);
            row = [];
          }

          for (let i = 0; i < text.length; i += 1) {
            const ch = text[i];
            if (ch === delimiter) { pushField(); continue; }
            if (ch === '\n') { pushField(); pushRow(); continue; }
            field += ch;
          }

          if (field.length > 0 || row.length > 0) {
            pushField();
            pushRow();
          }

          return rows;
        }

        document.getElementById('run').addEventListener('click', () => {
          const rows = parseDelimited('a,b\n1,2', ',');
          document.getElementById('out').textContent = JSON.stringify(rows);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#run")?;
    harness.assert_text("#out", r#"[["a","b"],["1","2"]]"#)?;
    Ok(())
}
