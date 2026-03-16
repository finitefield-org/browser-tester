use super::*;

#[test]
fn nested_function_reads_global_added_after_closure_creation() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        function makeReader() {
          return function () {
            return lateGlobal + ':' + Math.floor(2.9);
          };
        }

        const reader = makeReader();
        window.lateGlobal = 'ready';
        document.getElementById('out').textContent = reader();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "ready:2")?;
    Ok(())
}

#[test]
fn nested_function_updates_global_added_after_closure_creation() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        function makeWriter() {
          return function () {
            lateGlobal += '!';
            return lateGlobal;
          };
        }

        const writer = makeWriter();
        window.lateGlobal = 'ready';
        document.getElementById('out').textContent = writer() + '|' + lateGlobal;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "ready!|ready!")?;
    Ok(())
}
