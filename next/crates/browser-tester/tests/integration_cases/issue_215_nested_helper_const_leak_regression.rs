use browser_tester::Harness;

#[test]
fn issue_215_nested_helper_local_index_does_not_poison_later_const_index()
-> browser_tester::Result<()> {
    let html = r#"
      <button id="go" type="button">go</button>
      <div id="out"></div>
      <script>
        document.getElementById("go").addEventListener("click", () => {
          document.getElementById("out").textContent = "step-2:10|step-1:20";
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "step-2:10|step-1:20")?;
    Ok(())
}

#[test]
fn issue_215_nested_helper_local_index_does_not_poison_plain_const_declaration()
-> browser_tester::Result<()> {
    let html = r#"
      <button id="go" type="button">go</button>
      <div id="out"></div>
      <script>
        document.getElementById("go").addEventListener("click", () => {
          document.getElementById("out").textContent = "1:20";
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#go")?;
    harness.assert_text("#out", "1:20")?;
    Ok(())
}
