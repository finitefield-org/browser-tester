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

#[test]
fn object_url_anchor_download_with_empty_filename_and_blank_target_captures_without_navigation()
-> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const blob = new Blob(['hello'], { type: 'text/plain' });
            const url = URL.createObjectURL(blob);
            const anchor = document.createElement('a');
            anchor.href = url;
            anchor.download = '';
            anchor.target = '_blank';
            document.body.appendChild(anchor);
            anchor.click();
            anchor.remove();
            URL.revokeObjectURL(url);
          });
        </script>
    "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;

    assert!(h.take_location_navigations().is_empty());
    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: None,
            mime_type: Some("text/plain".to_string()),
            bytes: b"hello".to_vec(),
        }]
    );
    Ok(())
}

#[test]
fn object_url_anchor_download_is_suppressed_when_click_default_is_prevented() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const blob = new Blob(['blocked'], { type: 'text/plain' });
            const url = URL.createObjectURL(blob);
            const anchor = document.createElement('a');
            anchor.href = url;
            anchor.download = 'blocked.txt';
            anchor.addEventListener('click', (event) => event.preventDefault());
            document.body.appendChild(anchor);
            anchor.click();
            anchor.remove();
            URL.revokeObjectURL(url);
          });
        </script>
    "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;

    assert!(h.take_location_navigations().is_empty());
    assert!(h.take_downloads().is_empty());
    Ok(())
}

#[test]
fn revoked_object_url_anchor_download_is_not_captured_and_does_not_navigate() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const blob = new Blob(['gone'], { type: 'text/plain' });
            const url = URL.createObjectURL(blob);
            const anchor = document.createElement('a');
            anchor.href = url;
            anchor.download = 'gone.txt';
            URL.revokeObjectURL(url);
            document.body.appendChild(anchor);
            anchor.click();
            anchor.remove();
          });
        </script>
    "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;

    assert!(h.take_downloads().is_empty());
    assert!(h.take_location_navigations().is_empty());
    Ok(())
}

#[test]
fn revoking_one_object_url_does_not_break_other_object_url_downloads() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const blob = new Blob(['shared'], { type: 'text/plain' });
            const stale = URL.createObjectURL(blob);
            const live = URL.createObjectURL(blob);
            URL.revokeObjectURL(stale);

            const anchor = document.createElement('a');
            anchor.href = live;
            anchor.download = 'shared.txt';
            document.body.appendChild(anchor);
            anchor.click();
            anchor.remove();

            URL.revokeObjectURL(live);
          });
        </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;

    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: Some("shared.txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            bytes: b"shared".to_vec(),
        }]
    );
    Ok(())
}
