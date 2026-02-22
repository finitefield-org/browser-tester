use super::*;

#[test]
fn base_href_updates_base_uri_and_relative_url_resolution() -> Result<()> {
    let html = r#"
        <head>
          <base href='https://cdn.example/docs/' />
        </head>
        <body>
          <a id='rel' href='guide.html'>Guide</a>
          <a id='frag' href='#anchor'>Anchor</a>
          <map name='hotspots'>
            <area id='hot' href='map/start' alt='hotspot'>
          </map>
          <audio id='snd' src='media/theme.mp3'></audio>
          <div id='anchor'>section</div>
          <button id='run'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              document.getElementById('result').textContent =
                document.baseURI + '|' +
                document.getElementById('rel').href + '|' +
                document.getElementById('anchor').baseURI + '|' +
                document.getElementById('hot').href + '|' +
                document.getElementById('snd').src;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://cdn.example/docs/|https://cdn.example/docs/guide.html|https://cdn.example/docs/|https://cdn.example/docs/map/start|https://cdn.example/docs/media/theme.mp3",
    )?;

    h.click("#frag")?;
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start/index.html".to_string(),
            to: "https://cdn.example/docs/#anchor".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn multiple_base_elements_obey_first_href_and_first_target_independently() -> Result<()> {
    let html = r#"
        <head>
          <base target='_self' />
          <base href='https://second.example/base/' target='_blank' />
          <base href='https://third.example/ignored/' target='_top' />
        </head>
        <body>
          <a id='go' href='page.html'>go</a>
          <button id='run'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              document.getElementById('result').textContent =
                document.baseURI + '|' + document.getElementById('go').href;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://second.example/base/|https://second.example/base/page.html",
    )?;

    h.click("#go")?;
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start/index.html".to_string(),
            to: "https://second.example/base/page.html".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn base_target_with_newline_is_sanitized_to_blank_for_default_navigation() -> Result<()> {
    let html = r#"
        <head>
          <base href='https://example.com/base/' target='bad&#10;name' />
        </head>
        <body>
          <a id='default' href='next'>default target</a>
          <a id='self' href='self' target='_self'>self target</a>
          <button id='run'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              document.getElementById('result').textContent =
                document.baseURI + '|' + document.getElementById('default').href;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com/base/|https://example.com/base/next",
    )?;

    h.click("#default")?;
    h.click("#self")?;
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::Assign,
            from: "https://app.local/start".to_string(),
            to: "https://example.com/base/self".to_string(),
        }]
    );
    Ok(())
}

#[test]
fn base_uri_defaults_to_location_href_when_base_element_is_missing() -> Result<()> {
    let html = r#"
        <a id='rel' href='next.html'>next</a>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.baseURI + '|' +
              document.getElementById('rel').baseURI + '|' +
              document.getElementById('rel').href;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/docs/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/docs/page.html|https://app.local/docs/page.html|https://app.local/docs/next.html",
    )?;
    Ok(())
}
