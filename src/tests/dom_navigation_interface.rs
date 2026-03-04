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
