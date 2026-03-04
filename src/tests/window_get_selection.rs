use super::*;

#[test]
fn window_get_selection_returns_document_selection_and_selected_text() -> Result<()> {
    let html = r#"
        <p id='host'>Hello Brave New World</p>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const text = host.firstChild;
            const selection = window.getSelection();
            selection.removeAllRanges();

            const range = document.createRange();
            range.setStart(text, 6);
            range.setEnd(text, 11);
            selection.addRange(range);

            const fromWindow = window.getSelection();
            const fromDocument = document.getSelection();
            const selectedRange = fromWindow.getRangeAt(0);
            const selectedText = text.textContent.substring(
              selectedRange.startOffset,
              selectedRange.endOffset
            );
            document.getElementById('result').textContent = [
              fromWindow === fromDocument,
              fromWindow.type,
              fromWindow.rangeCount,
              selectedText
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:Range:1:Brave")?;
    Ok(())
}

#[test]
fn window_get_selection_rejects_arguments() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let threw = false;
            try {
              window.getSelection(1);
            } catch (e) {
              threw = String(e).includes('getSelection takes no arguments');
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
fn window_get_selection_property_is_read_only() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let readOnly = false;
            try {
              window.getSelection = () => null;
            } catch (e) {
              readOnly = String(e).includes('window.getSelection is read-only');
            }
            document.getElementById('result').textContent = [
              readOnly,
              typeof window.getSelection === 'function',
              window.getSelection() === document.getSelection()
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:true")?;
    Ok(())
}
