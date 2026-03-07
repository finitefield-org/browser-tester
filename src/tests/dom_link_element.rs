use super::*;

#[test]
fn link_href_rel_and_metadata_properties_roundtrip_work() -> Result<()> {
    let html = r#"
        <head>
          <link
            id='sheet'
            href='/styles/main.css'
            rel='stylesheet preload'
            type='text/css'
            hreflang='en'
            crossorigin='use-credentials'
            referrerpolicy='origin'>
          <link id='nohref' rel='stylesheet'>
        </head>
        <body>
          <a id='anchor' href='/page'>page</a>
          <button id='run' type='button'>run</button>
          <p id='result'></p>
          <script>
            document.getElementById('run').addEventListener('click', () => {
              const sheet = document.getElementById('sheet');
              const nohref = document.getElementById('nohref');

              const initialRoles = sheet.role + ':' + nohref.role;
              const initial =
                sheet.href + ':' +
                sheet.rel + ':' +
                sheet.relList.length + ':' +
                sheet.type + ':' +
                sheet.hreflang + ':' +
                sheet.crossOrigin + ':' +
                sheet.referrerPolicy + ':' +
                sheet.disabled + ':' +
                document.links.length + ':' +
                document.links[0].id;

              sheet.href = '/styles/next.css';
              sheet.rel = 'alternate stylesheet';
              sheet.type = 'text/plain';
              sheet.hreflang = 'ja';
              sheet.crossOrigin = 'anonymous';
              sheet.referrerPolicy = 'no-referrer';
              sheet.disabled = true;

              const assigned =
                sheet.href + ':' +
                sheet.getAttribute('href') + ':' +
                sheet.rel + ':' +
                sheet.relList.length + ':' +
                sheet.type + ':' +
                sheet.hreflang + ':' +
                sheet.crossOrigin + ':' +
                sheet.referrerPolicy + ':' +
                sheet.disabled + ':' +
                sheet.getAttribute('disabled');

              sheet.disabled = false;
              const disabledReset =
                sheet.disabled + ':' + (sheet.getAttribute('disabled') === null);

              nohref.href = '/styles/missing.css';
              const roleAfterHref = nohref.role;

              sheet.role = 'none';
              const roleAssigned = sheet.role + ':' + sheet.getAttribute('role');
              sheet.removeAttribute('role');
              const roleRestored = sheet.role + ':' + (sheet.getAttribute('role') === null);

              document.getElementById('result').textContent =
                initialRoles + '|' +
                initial + '|' +
                assigned + '|' +
                disabledReset + '|' +
                roleAfterHref + '|' +
                roleAssigned + '|' +
                roleRestored;
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/docs/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "link:|https://app.local/styles/main.css:stylesheet preload:2:text/css:en:use-credentials:origin:false:1:anchor|https://app.local/styles/next.css:/styles/next.css:alternate stylesheet:2:text/plain:ja:anonymous:no-referrer:true:true|false:true|link|none:none|link:true",
    )?;
    Ok(())
}

#[test]
fn link_is_void_and_implicit_role_depends_on_href_presence() -> Result<()> {
    let html = r#"
        <p id='line'>
          before
          <link id='icon' rel='icon' href='/favicon.ico'>
          after
        </p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const line = document.getElementById('line');
            const icon = document.getElementById('icon');

            const initial =
              icon.role + ':' +
              icon.tagName + ':' +
              line.childElementCount + ':' +
              line.textContent.replace(/\s+/g, ' ').trim();

            icon.href = '#favicon';
            const nextHref = icon.href;

            icon.removeAttribute('href');
            const roleWithoutHref = icon.role + ':' + (icon.getAttribute('href') === null);

            icon.setAttribute('sizes', '32x32');
            icon.setAttribute('as', 'image');
            const attrs = icon.getAttribute('sizes') + ':' + icon.getAttribute('as');

            document.getElementById('result').textContent =
              initial + '|' + nextHref + '|' + roleWithoutHref + '|' + attrs;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/guide/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "link:LINK:1:before after|https://app.local/guide/page.html#favicon|:true|32x32:image",
    )?;
    Ok(())
}

#[test]
fn link_null_and_invalid_url_properties_match_anchor_semantics_and_click_is_noop() -> Result<()> {
    let html = r#"
        <head>
          <link id='missing' rel='stylesheet'>
          <link id='bad' rel='stylesheet' href='http://'>
          <link id='hostless' rel='stylesheet' href='https:Example.COM:080/styles/main.css'>
        </head>
        <body>
          <button id='run' type='button'>run</button>
          <p id='result'></p>
          <script>
            let clicks = 0;
            document.getElementById('bad').addEventListener('click', () => {
              clicks += 1;
            });
            document.getElementById('run').addEventListener('click', () => {
              const missing = document.getElementById('missing');
              const bad = document.getElementById('bad');
              const hostless = document.getElementById('hostless');

              const missingState = [
                'href=' + missing.href,
                'protocol=' + missing.protocol,
                'host=' + missing.host,
                'pathname=' + missing.pathname,
                'origin=' + missing.origin
              ].join(',');

              const badState = [
                'href=' + bad.href,
                'protocol=' + bad.protocol,
                'host=' + bad.host,
                'pathname=' + bad.pathname,
                'origin=' + bad.origin
              ].join(',');

              bad.protocol = 'https:';
              bad.host = 'example.com:443';

              document.getElementById('result').textContent = [
                missingState,
                badState,
                'attr=' + bad.getAttribute('href'),
                'hostless=' + hostless.href,
                'clicks=' + clicks
              ].join('|');
            });
          </script>
        </body>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/base/index.html", html)?;
    h.click("#bad")?;
    h.click("#hostless")?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "href=,protocol=:,host=,pathname=,origin=|href=http://,protocol=:,host=,pathname=,origin=|attr=http://|hostless=https://example.com:80/styles/main.css|clicks=1",
    )?;
    assert!(h.take_location_navigations().is_empty());
    Ok(())
}
