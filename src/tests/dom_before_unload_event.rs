use super::*;

#[test]
fn before_unload_event_constructor_exposes_legacy_return_value() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const ev = new BeforeUnloadEvent('beforeunload');
          document.getElementById('result').textContent = [
            ev.type,
            ev.returnValue === '',
            ev.defaultPrevented,
            ev.cancelable
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "beforeunload:true:false:false")?;
    Ok(())
}

#[test]
fn before_unload_event_constructor_return_value_can_mark_default_prevented() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const ev = new BeforeUnloadEvent('beforeunload', {
            cancelable: true,
            returnValue: 'leave?'
          });
          document.getElementById('result').textContent = [
            ev.returnValue,
            ev.defaultPrevented,
            ev.cancelable
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "leave?:true:true")?;
    Ok(())
}

#[test]
fn before_unload_event_return_value_assignment_cancels_dispatch() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.addEventListener('beforeunload', (event) => {
            event.returnValue = 'stay';
            document.getElementById('result').textContent = [
              event.returnValue,
              event.defaultPrevented
            ].join(':');
          });

          document.getElementById('run').addEventListener('click', () => {
            const ok = target.dispatchEvent(
              new BeforeUnloadEvent('beforeunload', { cancelable: true })
            );
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + String(ok);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "stay:true:false")?;
    Ok(())
}

#[test]
fn before_unload_event_return_value_propagates_to_later_listeners() -> Result<()> {
    let html = r#"
        <div id='target'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.addEventListener('beforeunload', (event) => {
            const alias = event;
            alias.returnValue = 'stay';
          });
          target.addEventListener('beforeunload', (event) => {
            document.getElementById('result').textContent = [
              event.returnValue,
              event.defaultPrevented
            ].join(':');
          });

          document.getElementById('run').addEventListener('click', () => {
            const ok = target.dispatchEvent(
              new BeforeUnloadEvent('beforeunload', { cancelable: true })
            );
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + String(ok);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "stay:true:false")?;
    Ok(())
}

#[test]
fn before_unload_event_constructor_rejects_non_object_options() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let threw = false;
            try {
              new BeforeUnloadEvent('beforeunload', 123);
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
