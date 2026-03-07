use super::*;

#[test]
fn hash_change_event_constructor_populates_properties() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const ev = new HashChangeEvent('hashchange', {
            oldURL: 'https://a.test/path#old',
            newURL: 'https://a.test/path#new',
            bubbles: true,
            cancelable: true
          });
          document.getElementById('result').textContent = [
            ev.type,
            ev.oldURL,
            ev.newURL,
            ev.bubbles,
            ev.cancelable,
            ev.isTrusted
          ].join('~');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "hashchange~https://a.test/path#old~https://a.test/path#new~true~true~false",
    )?;
    Ok(())
}

#[test]
fn dispatch_event_with_hash_change_event_payload_preserves_fields() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.addEventListener('hashchange', (event) => {
            document.getElementById('result').textContent = [
              event.oldURL,
              event.newURL,
              event.isTrusted
            ].join('~');
          });

          document.getElementById('run').addEventListener('click', () => {
            const ev = new HashChangeEvent('hashchange', {
              oldURL: 'https://old.test/app#before',
              newURL: 'https://old.test/app#after'
            });
            target.dispatchEvent(ev);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://old.test/app#before~https://old.test/app#after~false",
    )?;
    Ok(())
}

#[test]
fn hash_change_event_constructor_rejects_non_object_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let threw = false;
            try {
              new HashChangeEvent('hashchange', 123);
            } catch (error) {
              threw = String(error).includes('options argument must be an object');
            }
            document.getElementById('result').textContent = String(threw);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn location_hash_navigation_dispatches_hashchange_event_with_old_and_new_urls() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const logs = [];
          window.addEventListener('hashchange', (event) => {
            logs.push([
              event.type,
              event.oldURL,
              event.newURL,
              event.bubbles,
              event.cancelable,
              event.isTrusted
            ].join('~'));
          });

          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://app.local/path';
            location.hash = 'frag';
            document.getElementById('result').textContent = logs.join('||');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "hashchange~https://app.local/path~https://app.local/path#frag~false~false~true",
    )?;
    Ok(())
}

#[test]
fn location_hash_navigation_uses_normalized_urls_after_default_port_navigation() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const logs = [];
          window.addEventListener('hashchange', (event) => {
            logs.push(event.oldURL + '~' + event.newURL);
          });

          document.getElementById('run').addEventListener('click', () => {
            location.href = 'http://example.com:80/path';
            location.hash = 'frag';
            document.getElementById('result').textContent = [
              location.href,
              logs.join('||')
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "http://example.com/path#frag|http://example.com/path~http://example.com/path#frag",
    )?;
    assert_eq!(
        h.take_location_navigations(),
        vec![
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "about:blank".to_string(),
                to: "http://example.com/path".to_string(),
            },
            LocationNavigation {
                kind: LocationNavigationKind::HrefSet,
                from: "http://example.com/path".to_string(),
                to: "http://example.com/path#frag".to_string(),
            },
        ]
    );
    Ok(())
}

#[test]
fn location_same_url_setters_do_not_dispatch_hashchange_or_record_navigation_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const logs = [];
          window.addEventListener('hashchange', (event) => {
            logs.push(event.oldURL + '~' + event.newURL);
          });

          document.getElementById('run').addEventListener('click', () => {
            location.href = 'https://app.local/path#frag';
            location.hash = 'frag';
            location.hash = '#frag';
            location.port = '443';
            location.hostname = 'app.local';
            location.pathname = '/path';
            location.search = '';
            location.protocol = 'https:';
            document.getElementById('result').textContent = [
              location.href,
              logs.length
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "https://app.local/path#frag|0")?;
    assert_eq!(
        h.take_location_navigations(),
        vec![LocationNavigation {
            kind: LocationNavigationKind::HrefSet,
            from: "about:blank".to_string(),
            to: "https://app.local/path#frag".to_string(),
        }]
    );
    Ok(())
}
