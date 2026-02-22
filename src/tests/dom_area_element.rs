use super::*;

#[test]
fn area_properties_and_document_links_include_href_areas() -> Result<()> {
    let html = r#"
        <map name='primary'>
          <area id='left' shape='circle' coords='75,75,75' href='/left' alt='Click left'>
          <area id='noop' shape='default' alt='No link'>
        </map>
        <a id='other' href='/other'>other</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const left = document.getElementById('left');
            left.referrerPolicy = 'origin';
            left.download = 'left.txt';
            left.ping = 'https://ping.example';

            document.getElementById('result').textContent =
              left.href + '|' +
              left.shape + '|' +
              left.coords + '|' +
              left.getAttribute('alt') + '|' +
              left.referrerPolicy + '|' +
              left.download + '|' +
              left.ping + '|' +
              document.links.length + '|' +
              document.links[0].id + ':' + document.links[1].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://example.com/base/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/left|circle|75,75,75|Click left|origin|left.txt|https://ping.example|2|left:other",
    )?;
    Ok(())
}

#[test]
fn area_click_follows_href_and_skips_blank_target_or_missing_href() -> Result<()> {
    let html = r#"
        <map name='routes'>
          <area id='go' href='/go' alt='go'>
          <area id='mail' href='mailto:m.bluth@example.com' alt='mail'>
          <area id='blank' href='/blank' target='_blank' alt='blank'>
          <area id='nohref' alt='no href'>
        </map>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#go")?;
    h.click("#mail")?;
    h.click("#blank")?;
    h.click("#nohref")?;

    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "https://app.local/start".to_string(),
                to: "https://app.local/go".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::Assign,
                from: "https://app.local/go".to_string(),
                to: "mailto:m.bluth@example.com".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn area_download_blob_click_captures_download_without_navigation() -> Result<()> {
    let html = r#"
        <map name='save'>
          <area id='save-area' alt='save'>
        </map>
        <button id='prep'>prep</button>
        <script>
          document.getElementById('prep').addEventListener('click', () => {
            const blob = new Blob(['abc'], { type: 'text/plain' });
            const url = URL.createObjectURL(blob);
            const area = document.getElementById('save-area');
            area.href = url;
            area.download = 'map.txt';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#prep")?;
    h.click("#save-area")?;

    assert!(h.take_location_navigations().is_empty());
    assert_eq!(
        h.take_downloads(),
        vec![DownloadArtifact {
            filename: Some("map.txt".to_string()),
            mime_type: Some("text/plain".to_string()),
            bytes: b"abc".to_vec(),
        }]
    );
    Ok(())
}

#[test]
fn area_role_is_link_with_href_and_empty_without_href() -> Result<()> {
    let html = r#"
        <map name='roles'>
          <area id='hot' href='/x' alt='x'>
          <area id='plain' alt='plain'>
        </map>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const hot = document.getElementById('hot');
            const plain = document.getElementById('plain');
            const initial = hot.role + ':' + plain.role;
            plain.role = 'note';
            const assigned = plain.role + ':' + plain.getAttribute('role');
            plain.removeAttribute('role');
            const restored = plain.role + ':' + (plain.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "link:|note:note|:true")?;
    Ok(())
}

#[test]
fn area_url_and_hyperlink_properties_reflect_and_to_string_work() -> Result<()> {
    let html = r#"
        <map name='primary'>
          <area id='left' href='/docs/p?x=1#h' alt='left'>
        </map>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://example.com/base/index.html';

            document.getElementById('left').download = 'left.txt';
            document.getElementById('left').referrerPolicy = 'strict-origin';
            document.getElementById('left').rel = 'noopener';
            document.getElementById('left').target = '_blank';
            document.getElementById('left').type = 'text/plain';
            document.getElementById('left').ping = 'https://ping.example';
            document.getElementById('left').shape = 'rect';
            document.getElementById('left').coords = '0,0,10,10';

            document.getElementById('result').textContent =
              document.getElementById('left').href + '|' +
              document.getElementById('left').protocol + '|' +
              document.getElementById('left').host + '|' +
              document.getElementById('left').pathname + '|' +
              document.getElementById('left').search + '|' +
              document.getElementById('left').hash + '|' +
              document.getElementById('left').origin + '|' +
              document.getElementById('left').download + '|' +
              document.getElementById('left').referrerPolicy + '|' +
              document.getElementById('left').rel + '|' +
              document.getElementById('left').target + '|' +
              document.getElementById('left').type + '|' +
              document.getElementById('left').ping + '|' +
              document.getElementById('left').shape + '|' +
              document.getElementById('left').coords + '|' +
              document.getElementById('left').toString();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/docs/p?x=1#h|https:|example.com|/docs/p|?x=1|#h|https://example.com|left.txt|strict-origin|noopener|_blank|text/plain|https://ping.example|rect|0,0,10,10|https://example.com/docs/p?x=1#h",
    )?;
    Ok(())
}
