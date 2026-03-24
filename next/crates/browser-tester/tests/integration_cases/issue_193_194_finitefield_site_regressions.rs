use browser_tester::Harness;

#[test]
fn issue_193_postfix_increment_inside_expression_is_supported() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        let rowSeq = 1;
        function createDefaultRow(partial = {}) {
          return {
            id: partial.id || "r" + rowSeq++,
          };
        }
        document.getElementById("out").textContent = createDefaultRow({}).id;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "r1")?;
    Ok(())
}

#[test]
fn issue_194_array_destructure_assignment_inside_else_if_branch_is_supported()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const state = {
          rows: [{ id: "a" }, { id: "b" }, { id: "c" }],
        };

        function reorder(action, index) {
          if (action === "duplicate") {
            state.rows.splice(index + 1, 0, state.rows[index]);
          } else if (action === "delete") {
            state.rows.splice(index, 1);
          } else if (action === "up" && index > 0) {
            const previous = state.rows[index - 1];
            state.rows[index - 1] = state.rows[index];
            state.rows[index] = previous;
          } else if (action === "down" && index < state.rows.length - 1) {
            const next = state.rows[index + 1];
            state.rows[index + 1] = state.rows[index];
            state.rows[index] = next;
          }
        }

        reorder("up", 2);
        document.getElementById("out").textContent = state.rows.map((row) => row.id).join(",");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "a,c,b")?;
    Ok(())
}
