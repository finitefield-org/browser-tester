use browser_tester::Harness;

#[test]
fn issue_202_async_digest_stub_updates_dom_after_await() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <div id="err"></div>
      <div id="meta"></div>
      <script>
        if (!window.crypto) { window.crypto = {}; }
        window.crypto.subtle = {
          digest: function (_alg, _data) {
            return Promise.resolve(new Uint8Array([65, 66, 67]).buffer);
          }
        };

        (async function () {
          const digest = await crypto.subtle.digest("SHA-256", new Uint8Array([1, 2, 3]));
          document.getElementById("meta").textContent =
            typeof digest + ":" + String(digest && digest.byteLength);
          document.getElementById("out").textContent =
            Array.from(new Uint8Array(digest)).join(",");
        })().catch(function (error) {
          document.getElementById("err").textContent =
            error && error.message ? error.message : String(error);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#err", "")?;
    harness.assert_text("#meta", "object:3")?;
    harness.assert_text("#out", "65,66,67")?;
    Ok(())
}

#[test]
fn issue_202_window_property_reads_as_global_identifier_inside_function()
-> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        function installAndRead() {
          window.hashApi = { tag: "ok" };
          document.getElementById("out").textContent =
            typeof hashApi + ":" + hashApi.tag;
        }

        installAndRead();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "object:ok")?;
    Ok(())
}
