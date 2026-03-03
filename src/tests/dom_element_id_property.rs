use super::*;

#[test]
fn element_id_reflects_attribute_and_document_lookup() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const initial = box.id + ',' + box.getAttribute('id');

            box.id = 'main-panel';
            const byNew = document.getElementById('main-panel') === box;
            const byOld = document.getElementById('box') === null;

            box.setAttribute('id', 'AttrId');
            const fromAttr = box.id;
            const byAttr = document.getElementById('AttrId') === box;
            const caseSensitive = document.getElementById('attrid') === null;

            box.id = '';
            const idEmpty = box.id === '';
            const attrEmpty = box.getAttribute('id') === '';
            const clearedLookup = document.getElementById('AttrId') === null;

            document.getElementById('result').textContent = [
              initial,
              byNew,
              byOld,
              fromAttr,
              byAttr,
              caseSensitive,
              idEmpty,
              attrEmpty,
              clearedLookup
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "box,box|true|true|AttrId|true|true|true|true|true",
    )?;
    Ok(())
}

#[test]
fn element_id_duplicate_values_follow_document_order_for_lookup() -> Result<()> {
    let html = r#"
        <div id='dup'>A</div>
        <div id='dup'>B</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const first = document.getElementById('dup');
            const firstText = first ? first.textContent.trim() : 'none';

            first.removeAttribute('id');
            const second = document.getElementById('dup');
            const secondText = second ? second.textContent.trim() : 'none';
            const caseSensitive = document.getElementById('Dup') === null;

            document.getElementById('result').textContent =
              firstText + ':' + secondText + ':' + caseSensitive;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "A:B:true")?;
    Ok(())
}
