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
fn data_url_anchor_downloads_are_recorded_with_metadata_and_bytes() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const csv = "\ufeffa,b\n1,2";
            const anchor = document.createElement('a');
            anchor.href = `data:text/csv;charset=utf-8,${encodeURIComponent(csv)}`;
            anchor.download = 'sample.csv';
            document.body.appendChild(anchor);
            anchor.click();
            anchor.remove();
          });
        </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;

    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: Some("sample.csv".to_string()),
            mime_type: Some("text/csv".to_string()),
            bytes: "\u{feff}a,b\n1,2".as_bytes().to_vec(),
        }]
    );
    assert!(h.take_location_navigations().is_empty());
    Ok(())
}

#[test]
fn base64_data_url_anchor_downloads_are_recorded() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const anchor = document.createElement('a');
            anchor.href = 'data:text/plain;base64,aGVsbG8=';
            anchor.download = 'hello.txt';
            document.body.appendChild(anchor);
            anchor.click();
            anchor.remove();
          });
        </script>
    "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;

    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: Some("hello.txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            bytes: b"hello".to_vec(),
        }]
    );
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

#[test]
fn anchor_download_default_action_uses_post_click_attribute_state_and_keeps_network_downloads_uncaptured()
-> Result<()> {
    let html = r#"
        <a id='link' href='/initial' download='initial.txt'>link</a>
        <script>
          let step = 0;
          const live = URL.createObjectURL(new Blob(['live'], { type: 'text/plain' }));
          const link = document.getElementById('link');
          link.addEventListener('click', () => {
            if (step === 0) {
              link.href = live;
              link.download = 'live.txt';
            } else if (step === 1) {
              link.href = '/report.csv';
              link.download = 'network.txt';
            } else {
              link.href = '/done';
              link.removeAttribute('download');
            }
            step += 1;
          });
        </script>
    "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#link")?;
    h.click("#link")?;
    h.click("#link")?;

    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: Some("live.txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            bytes: b"live".to_vec(),
        }]
    );
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start".to_string(),
            to: "https://app.local/done".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn reduced_object_url_download_contract_uses_final_click_state_for_capture_and_navigation_work()
-> Result<()> {
    let html = r#"
        <a id='link' href='/initial'>link</a>
        <script>
          let step = 0;
          const blobUrl = URL.createObjectURL(new Blob(['blob-body'], { type: 'text/plain' }));
          const link = document.getElementById('link');
          link.addEventListener('click', () => {
            if (step === 0) {
              link.href = blobUrl;
              link.download = 'blob.txt';
            } else {
              link.href = '/final';
              link.removeAttribute('download');
            }
            step += 1;
          });
        </script>
    "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#link")?;
    h.click("#link")?;

    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: Some("blob.txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            bytes: b"blob-body".to_vec(),
        }]
    );
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start".to_string(),
            to: "https://app.local/final".to_string(),
        }]
    );
    Ok(())
}
