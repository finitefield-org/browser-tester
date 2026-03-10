use super::*;

#[test]
fn element_cut_event_default_action_copies_selected_text_and_removes_it_from_input() -> Result<()> {
    let html = r#"
        <input id='target' value='Alpha Beta' />
        <p id='event'></p>
        <script>
          const target = document.getElementById('target');
          const log = [];
          target.setSelectionRange(0, 5);

          target.addEventListener('focus', () => {
            log.push('focus:' + String(document.activeElement === target));
          });
          target.addEventListener('focusin', () => {
            log.push('focusin:' + String(document.activeElement === target));
          });
          target.addEventListener('cut', (event) => {
            log.push([
              'cut',
              event.bubbles,
              event.cancelable,
              event.isTrusted
            ].join(':'));
          });
          document.addEventListener('selectionchange', () => {
            log.push('selectionchange:' + target.selectionStart + '-' + target.selectionEnd);
          });
          target.addEventListener('input', () => {
            log.push([
              'input',
              target.value,
              target.selectionStart + '-' + target.selectionEnd,
              String(document.activeElement === target)
            ].join(':'));
            document.getElementById('event').textContent = log.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("original");
    h.cut("#target")?;
    assert_eq!(h.clipboard_text(), "Alpha");
    h.assert_text(
        "#event",
        "focus:true,focusin:true,cut:true:true:true,selectionchange:0-0,input: Beta:0-0:true",
    )?;
    Ok(())
}

#[test]
fn element_cut_event_prevent_default_and_set_data_overrides_clipboard_without_mutating_value()
-> Result<()> {
    let html = r#"
        <input id='target' value='Alpha Beta' />
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(0, 5);
          document.addEventListener('cut', (event) => {
            event.preventDefault();
            event.clipboardData.setData('text/plain', 'CUSTOM-CUT');
            document.getElementById('result').textContent =
              target.value + '|' + target.selectionStart + '-' + target.selectionEnd;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("original");
    h.cut("#target")?;
    assert_eq!(h.clipboard_text(), "CUSTOM-CUT");
    h.assert_text("#result", "Alpha Beta|0-5")?;
    Ok(())
}

#[test]
fn element_cut_event_synthetic_dispatch_does_not_change_clipboard_or_value() -> Result<()> {
    let html = r#"
        <input id='target' value='Alpha Beta' />
        <p id='result'></p>
        <script>
          const target = document.getElementById('target');
          target.setSelectionRange(0, 5);
          target.addEventListener('cut', (event) => {
            event.preventDefault();
            event.clipboardData.setData('text/plain', 'SHOULD_NOT_APPLY');
            document.getElementById('result').textContent =
              String(event.isTrusted) + '|' + target.value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_clipboard_text("original");
    h.dispatch("#target", "cut")?;
    assert_eq!(h.clipboard_text(), "original");
    h.assert_text("#result", "false|Alpha Beta")?;
    Ok(())
}
