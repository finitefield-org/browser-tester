use browser_tester::Harness;

#[test]
fn issue_190_document_active_element_tag_name_is_supported() -> browser_tester::Result<()> {
    let html = r#"
      <textarea id="field"></textarea>
      <div id="out"></div>
      <script>
        const field = document.getElementById("field");
        field.focus();
        document.getElementById("out").textContent = document.activeElement.tagName;
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "TEXTAREA")?;
    Ok(())
}

#[test]
fn issue_191_data_url_anchor_download_is_captured_as_artifact() -> browser_tester::Result<()> {
    let html = r#"
      <button id="download">Download</button>
      <script>
        document.getElementById("download").addEventListener("click", () => {
          const csv = "\ufeffa,b\n1,2";
          const link = document.createElement("a");
          link.href = `data:text/csv;charset=utf-8,${encodeURIComponent(csv)}`;
          link.download = "sample.csv";
          document.body.appendChild(link);
          link.click();
          document.body.removeChild(link);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.click("#download")?;

    let downloads = harness.take_downloads();
    assert_eq!(downloads.len(), 1, "expected one download artifact");
    assert_eq!(downloads[0].filename.as_deref(), Some("sample.csv"));
    assert_eq!(downloads[0].mime_type.as_deref(), Some("text/csv"));
    assert_eq!(downloads[0].bytes, "\u{feff}a,b\n1,2".as_bytes());
    Ok(())
}

#[test]
fn issue_192_array_flat_is_supported() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const values = [["north"], ["south"]].flat();
        document.getElementById("out").textContent = values.join(",");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "north,south")?;
    Ok(())
}

#[test]
fn issue_192_array_flat_honors_depth_and_skips_sparse_slots() -> browser_tester::Result<()> {
    let html = r#"
      <div id="out"></div>
      <script>
        const nested = [];
        nested[0] = 1;
        nested[1] = [2, [3]];
        nested[2] = "skip";
        delete nested[2];
        nested[3] = [4];
        const result = nested.flat(2);
        document.getElementById("out").textContent = result.join(",");
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "1,2,3,4")?;
    Ok(())
}
