use super::*;

#[test]
fn assigning_window_open_fallback_does_not_fail_harness_from_html() -> Result<()> {
    let html = r#"
        <div id='x'>ok</div>
        <script>
          window.open = window.open || function(){};
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#x", "ok")?;
    Ok(())
}
