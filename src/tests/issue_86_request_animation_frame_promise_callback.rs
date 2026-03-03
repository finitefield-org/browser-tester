use super::*;

#[test]
fn issue_86_request_animation_frame_accepts_promise_resolve_callback() -> Result<()> {
    let html = r#"
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        document.getElementById('run').addEventListener('click', () => {
          new Promise((resolve) => window.requestAnimationFrame(resolve))
            .then((ts) => {
              document.getElementById('out').textContent = 'ok:' + ts;
            })
            .catch((err) => {
              document.getElementById('out').textContent =
                'err:' + String(err && err.message ? err.message : err);
            });
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "")?;
    h.advance_time(16)?;
    h.assert_text("#out", "ok:16")?;
    Ok(())
}
