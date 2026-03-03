use super::*;

#[test]
fn object_url_anchor_downloads_are_recorded_with_metadata_and_count() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          function triggerDownload(filename, content, type) {
            const blob = new Blob([content], { type });
            const url = URL.createObjectURL(blob);
            const anchor = document.createElement('a');
            anchor.href = url;
            anchor.download = filename;
            document.body.appendChild(anchor);
            anchor.click();
            anchor.remove();
            URL.revokeObjectURL(url);
          }

          document.getElementById('run').addEventListener('click', () => {
            triggerDownload('report.csv', 'a,b\n1,2', 'text/csv');
            triggerDownload('notes.txt', 'hello', 'text/plain');
          });
        </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;

    assert!(h.take_location_navigations().is_empty());
    assert_eq!(
        h.take_downloads(),
        vec![
            DownloadArtifact {
                filename: Some("report.csv".to_string()),
                mime_type: Some("text/csv".to_string()),
                bytes: b"a,b\n1,2".to_vec(),
            },
            DownloadArtifact {
                filename: Some("notes.txt".to_string()),
                mime_type: Some("text/plain".to_string()),
                bytes: b"hello".to_vec(),
            },
        ]
    );
    assert!(h.take_downloads().is_empty());
    Ok(())
}
