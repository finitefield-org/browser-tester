use super::*;

#[test]
fn navigation_object_is_exposed_on_window_and_global() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const nav = navigation;
          const methods = [
            'back',
            'entries',
            'forward',
            'navigate',
            'reload',
            'traverseTo',
            'updateCurrentEntry',
          ];
          const hasMethods = methods.every((name) => typeof nav[name] === 'function');
          const state = [
            nav === window.navigation,
            nav.activation === null,
            nav.transition === null,
            typeof nav.canGoBack === 'boolean',
            typeof nav.canGoForward === 'boolean',
            nav.currentEntry !== null && typeof nav.currentEntry.key === 'string',
            hasMethods,
            typeof nav.addEventListener === 'function' && typeof nav.dispatchEvent === 'function',
          ];
          document.getElementById('result').textContent = state.join(':');
        </script>
        "#;

    let h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.assert_text("#result", "true:true:true:true:true:true:true:true")?;
    Ok(())
}

#[test]
fn navigation_entries_and_traverse_to_key() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const firstKey = navigation.currentEntry.key;
            navigation.navigate('/a');
            const secondKey = navigation.currentEntry.key;
            navigation.navigate('/b');
            const thirdKey = navigation.currentEntry.key;

            const before = [
              navigation.canGoBack,
              navigation.canGoForward,
              navigation.entries().length,
            ].join(',');

            navigation.traverseTo(firstKey);

            const after = [
              location.pathname,
              navigation.currentEntry.key === firstKey,
              navigation.canGoBack,
              navigation.canGoForward,
              navigation.entries().length,
              secondKey !== firstKey,
              thirdKey !== secondKey,
            ].join(',');

            document.getElementById('result').textContent = `${before}|${after}`;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text("#result", "true,false,3|/start,true,false,true,3,true,true")?;
    Ok(())
}

#[test]
fn navigation_back_forward_return_promises() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            navigation.navigate('/one');
            navigation.navigate('/two');

            const backResult = navigation.back();
            const forwardResult = navigation.forward();

            document.getElementById('result').textContent = [
              location.pathname,
              backResult.committed !== undefined,
              backResult.finished !== undefined,
              forwardResult.finished !== undefined,
              navigation.currentEntry.url.endsWith('/two'),
              navigation.canGoBack,
              navigation.canGoForward,
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text("#result", "/two:true:true:true:true:true:false")?;
    Ok(())
}

#[test]
fn navigation_navigate_invalid_absolute_url_throws_and_keeps_current_entry() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let first = false;
            let second = false;

            try {
              navigation.navigate('https://example.com:abc/path');
            } catch (err) {
              first = String(err).includes('Invalid URL');
            }

            try {
              navigation.navigate('http://[::1/path');
            } catch (err) {
              second = String(err).includes('Invalid URL');
            }

            document.getElementById('result').textContent = [
              first,
              second,
              location.href,
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:https://app.local/start:https://app.local/start:1",
    )?;
    Ok(())
}

#[test]
fn navigation_navigate_special_host_inputs_canonicalize_and_empty_host_throw() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let invalidEmpty = false;
            let invalidQuery = false;

            navigation.navigate('http:Example.COM:080/root');
            const afterFirst = [
              location.href,
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            navigation.navigate('https:\\Example.COM\\next\\page');
            const afterSecond = [
              location.href,
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            try {
              navigation.navigate('http://');
            } catch (err) {
              invalidEmpty = String(err).includes('Invalid URL');
            }

            try {
              navigation.navigate('http:?x');
            } catch (err) {
              invalidQuery = String(err).includes('Invalid URL');
            }

            document.getElementById('result').textContent = [
              afterFirst,
              afterSecond,
              invalidEmpty,
              invalidQuery,
              location.href,
              navigation.entries().length
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "http://example.com/root,http://example.com/root,2|https://example.com/next/page,https://example.com/next/page,3|true|true|https://example.com/next/page|3",
    )?;
    Ok(())
}

#[test]
fn navigation_navigate_credentials_and_delimiter_inputs_canonicalize() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            navigation.navigate('https://a@b:p@q:r@example.com\\docs\\a b?a\'b#x`y');
            const special = [
              location.href,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            navigation.navigate('foo://example.com/\\docs\\a b?a\'b#x`y');
            const custom = [
              location.href,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            document.getElementById('result').textContent =
              [special, custom].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://a%40b:p%40q%3Ar@example.com/docs/a%20b?a%27b#x%60y,/docs/a%20b,?a%27b,#x%60y,https://a%40b:p%40q%3Ar@example.com/docs/a%20b?a%27b#x%60y,2|foo://example.com/\\docs\\a%20b?a'b#x%60y,/\\docs\\a%20b,?a'b,#x%60y,foo://example.com/\\docs\\a%20b?a'b#x%60y,3",
    )?;
    Ok(())
}

#[test]
fn navigation_navigate_authority_and_percent_residuals_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            navigation.navigate('https://a@@ExA%41mple.ORG/%2f%zz?x=%2f%zz#y=%2f%zz');
            const special = [
              location.href,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            const invalid = (() => {
              try {
                navigation.navigate('https://exa%mple.org/');
                return 'false';
              } catch (err) {
                return [
                  String(err).includes('Invalid URL'),
                  location.href,
                  navigation.currentEntry.url,
                  navigation.entries().length
                ].join(',');
              }
            })();

            navigation.navigate('foo://example.com/%2f%zz?x=%2f%zz#y=%2f%zz');
            const custom = [
              location.href,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            navigation.navigate('https://user:@example.com/%2f%zz?x=%2f%zz#y=%2f%zz', { history: 'replace' });
            const replaced = [
              location.href,
              location.pathname,
              location.search,
              location.hash,
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            document.getElementById('result').textContent =
              [special, invalid, custom, replaced].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz,/%2f%zz,?x=%2f%zz,#y=%2f%zz,https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz,2|true,https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz,https://a%40@exaample.org/%2f%zz?x=%2f%zz#y=%2f%zz,2|foo://example.com/%2f%zz?x=%2f%zz#y=%2f%zz,/%2f%zz,?x=%2f%zz,#y=%2f%zz,foo://example.com/%2f%zz?x=%2f%zz#y=%2f%zz,3|https://user@example.com/%2f%zz?x=%2f%zz#y=%2f%zz,/%2f%zz,?x=%2f%zz,#y=%2f%zz,https://user@example.com/%2f%zz?x=%2f%zz#y=%2f%zz,3",
    )?;
    Ok(())
}

#[test]
fn navigation_navigate_malformed_query_and_host_code_point_work() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            navigation.navigate('https://\uFF21example.com/?a=%zz&b=%E0%A4&c=%C3%28');
            const parsed = new URL(location.href);
            const afterNavigate = [
              location.href,
              location.host,
              location.search,
              parsed.searchParams.get('a'),
              parsed.searchParams.get('b'),
              parsed.searchParams.get('c'),
              parsed.searchParams.toString(),
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            navigation.navigate('https://e\u0301xample.com/?b=%E0%A4&a=%zz&a=1');
            const parsedIdna = new URL(location.href);
            const afterIdna = [
              location.href,
              location.host,
              location.search,
              parsedIdna.searchParams.getAll('a').join(':'),
              parsedIdna.searchParams.get('b'),
              parsedIdna.searchParams.toString(),
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            const mutated = new URL(navigation.currentEntry.url);
            mutated.searchParams.sort();
            mutated.searchParams.set('a', '%zz');
            navigation.navigate(mutated.href);
            const afterMutation = [
              location.href,
              location.search,
              mutated.searchParams.getAll('a').join(':'),
              mutated.searchParams.get('b'),
              mutated.searchParams.toString(),
              navigation.currentEntry.url,
              navigation.entries().length
            ].join(',');

            const invalid = (() => {
              try {
                navigation.navigate('https://%00example.com/');
                return 'false';
              } catch (err) {
                return [
                  String(err).includes('Invalid URL'),
                  location.href,
                  navigation.currentEntry.url,
                  navigation.entries().length
                ].join(',');
              }
            })();

            document.getElementById('result').textContent =
              [afterNavigate, afterIdna, afterMutation, invalid].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://aexample.com/?a=%zz&b=%E0%A4&c=%C3%28,aexample.com,?a=%zz&b=%E0%A4&c=%C3%28,%zz,\u{FFFD},\u{FFFD}(,a=%25zz&b=%EF%BF%BD&c=%EF%BF%BD%28,https://aexample.com/?a=%zz&b=%E0%A4&c=%C3%28,2|https://xn--xample-9ua.com/?b=%E0%A4&a=%zz&a=1,xn--xample-9ua.com,?b=%E0%A4&a=%zz&a=1,%zz:1,\u{FFFD},b=%EF%BF%BD&a=%25zz&a=1,https://xn--xample-9ua.com/?b=%E0%A4&a=%zz&a=1,3|https://xn--xample-9ua.com/?a=%25zz&b=%EF%BF%BD,?a=%25zz&b=%EF%BF%BD,%zz,\u{FFFD},a=%25zz&b=%EF%BF%BD,https://xn--xample-9ua.com/?a=%25zz&b=%EF%BF%BD,4|true,https://xn--xample-9ua.com/?a=%25zz&b=%EF%BF%BD,https://xn--xample-9ua.com/?a=%25zz&b=%EF%BF%BD,4",
    )?;
    Ok(())
}

#[test]
fn navigation_navigate_reload_and_update_current_entry_state() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            navigation.navigate('/article', { state: { step: 1 } });
            const afterNavigate = navigation.currentEntry.state.step;

            navigation.updateCurrentEntry({ state: { step: 2 } });
            const afterUpdate = navigation.currentEntry.state.step;

            navigation.reload({ state: { step: 3 } });
            const afterReload = navigation.currentEntry.state.step;

            document.getElementById('result').textContent = [
              location.pathname,
              afterNavigate,
              afterUpdate,
              afterReload,
              navigation.currentEntry.state.step,
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text("#result", "/article:1:2:3:3")?;
    Ok(())
}

#[test]
fn navigation_events_fire_for_successful_actions() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const log = [];

            navigation.addEventListener('navigate', (event) => {
              log.push(`navigate:${event.navigationType}:${event.destination.url}`);
            });
            navigation.addEventListener('currententrychange', () => {
              log.push('currententrychange');
            });
            navigation.addEventListener('navigatesuccess', () => {
              log.push('navigatesuccess');
            });
            navigation.addEventListener('navigateerror', () => {
              log.push('navigateerror');
            });

            navigation.navigate('/alpha');
            navigation.reload();

            document.getElementById('result').textContent = log.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/start", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "navigate:push:https://app.local/alpha|currententrychange|navigatesuccess|navigate:reload:https://app.local/alpha|navigatesuccess",
    )?;
    Ok(())
}
