use super::*;

#[test]
fn dispatch_event_returns_false_when_cancelable_and_prevented() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const box = document.getElementById('box');
          box.addEventListener('custom', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent = String(event.defaultPrevented);
          });
          document.getElementById('btn').addEventListener('click', () => {
            const ok = box.dispatchEvent(new Event('custom', { cancelable: true }));
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + String(ok);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false")?;
    Ok(())
}

#[test]
fn dispatch_event_returns_true_when_not_cancelable_even_if_prevent_default_called() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const box = document.getElementById('box');
          box.addEventListener('custom', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent = String(event.defaultPrevented);
          });
          document.getElementById('btn').addEventListener('click', () => {
            const ok = box.dispatchEvent(new Event('custom'));
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + ':' + String(ok);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:true")?;
    Ok(())
}

#[test]
fn dispatch_event_is_synchronous_and_sets_target_to_dispatch_target() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const box = document.getElementById('box');
          const order = [];
          box.addEventListener('custom', (event) => {
            order.push('listener:' + String(event.target === box));
          });
          document.getElementById('btn').addEventListener('click', () => {
            order.push('before');
            const ok = box.dispatchEvent(new Event('custom'));
            order.push('after:' + String(ok));
            document.getElementById('result').textContent = order.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "before|listener:true|after:true")?;
    Ok(())
}

#[test]
fn dispatch_event_works_on_event_target_instance_and_window() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const emitter = new EventTarget();
          const out = document.getElementById('result');
          emitter.addEventListener('ping', (event) => {
            out.textContent = out.textContent + 'E' + String(event.target === emitter) + '|';
          });
          window.addEventListener('resize', (event) => {
            out.textContent = out.textContent + 'W' + String(event.target === window) + '|';
          });
          document.getElementById('btn').addEventListener('click', () => {
            out.textContent = '';
            const e1 = emitter.dispatchEvent(new Event('ping'));
            const e2 = window.dispatchEvent(new Event('resize'));
            out.textContent = out.textContent + 'R' + String(e1) + ':' + String(e2);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "Etrue|Wtrue|Rtrue:true")?;
    Ok(())
}

#[test]
fn dispatch_event_throws_for_empty_event_type() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const box = document.getElementById('box');
          document.getElementById('btn').addEventListener('click', () => {
            let threw = false;
            try {
              box.dispatchEvent('');
            } catch (err) {
              threw = String(err).includes('InvalidStateError');
            }
            document.getElementById('result').textContent = String(threw);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true")?;
    Ok(())
}
