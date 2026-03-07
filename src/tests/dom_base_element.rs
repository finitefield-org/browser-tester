use super::*;

#[test]
fn html_base_element_global_and_instanceof_work() -> Result<()> {
    let html = r#"
        <head>
          <base id='base' href='/docs/' target='_self'>
        </head>
        <body>
          <a id='link' href='/other'>other</a>
          <p id='result'></p>
          <script>
            const base = document.getElementById('base');
            const link = document.getElementById('link');
            document.getElementById('result').textContent = [
              typeof HTMLBaseElement,
              window.HTMLBaseElement === HTMLBaseElement,
              base instanceof HTMLBaseElement,
              base instanceof HTMLElement,
              link instanceof HTMLBaseElement
            ].join(':');
          </script>
        </body>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "function:true:true:true:false")?;
    Ok(())
}

#[test]
fn base_href_and_target_properties_reflect_and_change_default_targeting() -> Result<()> {
    let html = r#"
        <head>
          <base id='base' href='/assets/' target='_self'>
        </head>
        <body>
          <a id='rel' href='guide.html'>Guide</a>
          <button id='run'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              const rel = document.getElementById('rel');
              const before = [
                document.getElementById('base').href,
                document.getElementById('base').target,
                document.getElementById('base').getAttribute('href'),
                document.getElementById('base').getAttribute('target'),
                rel.href
              ].join(':');

              document.getElementById('base').href = '/docs/';
              document.getElementById('base').target = 'frameA';

              const after = [
                document.getElementById('base').href,
                document.getElementById('base').target,
                document.getElementById('base').getAttribute('href'),
                document.getElementById('base').getAttribute('target'),
                document.baseURI,
                rel.href
              ].join(':');

              document.getElementById('result').textContent = before + '|' + after;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/assets/:_self:/assets/:_self:https://app.local/assets/guide.html|https://app.local/docs/:frameA:/docs/:frameA:https://app.local/docs/:https://app.local/docs/guide.html",
    )?;

    h.click("#rel")?;
    assert!(h.take_location_navigations().is_empty());
    Ok(())
}

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

#[test]
fn base_href_protocol_relative_canonicalizes_and_invalid_authority_falls_back_to_document_url()
-> Result<()> {
    let html = r#"
        <head>
          <base id='base' href='//Example.COM:080/Docs/' />
        </head>
        <body>
          <a id='rel' href='Guide.html'>Guide</a>
          <button id='run'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              const base = document.getElementById('base');
              const rel = document.getElementById('rel');

              const initial = [document.baseURI, rel.href].join('|');

              base.setAttribute('href', '//Example.COM:99999/Bad/');
              const invalid = [document.baseURI, rel.href].join('|');

              document.getElementById('result').textContent =
                [initial, invalid].join(',');
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://example.com:80/Docs/|https://example.com:80/Docs/Guide.html,https://app.local/start/index.html|https://app.local/start/Guide.html",
    )?;
    Ok(())
}
