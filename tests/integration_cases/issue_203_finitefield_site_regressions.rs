use browser_tester::Harness;

#[test]
fn issue_203_sticky_element_stays_pinned_after_window_scroll() -> browser_tester::Result<()> {
    let html = r#"
      <style>
        body { margin: 0; }
        #sticky {
          position: sticky;
          top: 0;
          height: 40px;
          background: #ddd;
        }
        #spacer {
          height: 1600px;
        }
      </style>
      <div id="sticky">sticky</div>
      <div id="spacer"></div>
      <button id="go" type="button">go</button>
      <div id="out"></div>
      <script>
        const sticky = document.getElementById("sticky");
        const out = document.getElementById("out");
        document.getElementById("go").addEventListener("click", () => {
          const beforeTop = Math.round(sticky.getBoundingClientRect().top);
          window.scrollTo(0, 300);
          const afterTop = Math.round(sticky.getBoundingClientRect().top);
          out.textContent =
            "scrollY=" + window.scrollY + ",before=" + beforeTop + ",after=" + afterTop;
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "scrollY=300,before=0,after=0")?;
    Ok(())
}

#[test]
fn issue_203_sticky_element_honors_rem_top_inset_during_scroll() -> browser_tester::Result<()> {
    let html = r#"
      <style>
        body { margin: 0; }
        #sticky {
          position: sticky;
          top: 5.75rem;
          height: 40px;
          background: #ddd;
        }
        #spacer {
          height: 1600px;
        }
      </style>
      <div id="sticky">sticky</div>
      <div id="spacer"></div>
      <button id="go" type="button">go</button>
      <div id="out"></div>
      <script>
        const sticky = document.getElementById("sticky");
        const out = document.getElementById("out");
        document.getElementById("go").addEventListener("click", () => {
          const beforeTop = Math.round(sticky.getBoundingClientRect().top);
          window.scrollTo(0, 300);
          const afterTop = Math.round(sticky.getBoundingClientRect().top);
          out.textContent =
            "scrollY=" + window.scrollY + ",before=" + beforeTop + ",after=" + afterTop;
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "scrollY=300,before=92,after=92")?;
    Ok(())
}
