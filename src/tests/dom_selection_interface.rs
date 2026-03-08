use super::*;

#[test]
fn document_and_window_get_selection_return_singleton() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const a = document.getSelection();
            const b = window.getSelection();
            document.getElementById('result').textContent = [
              a === b,
              a.rangeCount,
              a.type,
              a.isCollapsed,
              a.anchorNode === null,
              a.focusNode === null,
              a.direction
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:0:None:true:true:true:none")?;
    Ok(())
}

#[test]
fn selection_add_range_get_range_at_contains_node_and_remove_range() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const text = document.createTextNode('hello');
            host.replaceChildren(text);
            const selection = document.getSelection();
            const range = document.createRange();
            range.setStart(text, 1);
            range.setEnd(text, 4);

            selection.removeAllRanges();
            selection.addRange(range);

            const sameRange = selection.getRangeAt(0) === range;
            const selectedRange = selection.getRangeAt(0);
            const selected = text.textContent.substring(
              selectedRange.startOffset,
              selectedRange.endOffset
            );
            const rangeCount = selection.rangeCount;
            const type = selection.type;

            let getErr = false;
            try {
              selection.getRangeAt(1);
            } catch (e) {
              getErr = String(e).includes('IndexSizeError');
            }

            const other = document.createRange();
            other.setStart(text, 0);
            other.setEnd(text, 1);

            let removeErr = false;
            try {
              selection.removeRange(other);
            } catch (e) {
              removeErr = String(e).includes('NotFoundError');
            }

            const containsPartial = selection.containsNode(text, true);
            const containsWhole = selection.containsNode(text, false);

            selection.removeRange(range);

            document.getElementById('result').textContent = [
              sameRange,
              selected,
              rangeCount,
              type,
              getErr,
              removeErr,
              containsPartial,
              containsWhole,
              selection.rangeCount
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:ell:1:Range:true:true:true:false:0")?;
    Ok(())
}

#[test]
fn selection_direction_extend_collapse_and_set_position_work() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const text = document.createTextNode('abcde');
            host.replaceChildren(text);
            const selection = document.getSelection();

            selection.setBaseAndExtent(text, 4, text, 1);
            const range1 = selection.getRangeAt(0);
            const step1 = [
              selection.anchorOffset,
              selection.focusOffset,
              selection.direction,
              range1.startOffset,
              range1.endOffset,
              selection.type
            ].join(',');

            selection.collapseToEnd();
            const range2 = selection.getRangeAt(0);
            const step2 = [
              selection.rangeCount,
              selection.type,
              selection.anchorOffset,
              selection.focusOffset,
              range2.startOffset,
              range2.endOffset
            ].join(',');

            selection.setBaseAndExtent(text, 1, text, 4);
            selection.collapseToStart();
            const step3 = [selection.type, selection.anchorOffset, selection.focusOffset].join(',');

            selection.setPosition(text, 3);
            const range4 = selection.getRangeAt(0);
            const step4 = [
              selection.type,
              selection.anchorOffset,
              selection.focusOffset,
              range4.startOffset,
              range4.endOffset
            ].join(',');

            selection.collapse(null);
            const step5 = [
              selection.rangeCount,
              selection.type,
              selection.anchorNode === null,
              selection.focusNode === null
            ].join(',');

            document.getElementById('result').textContent =
              [step1, step2, step3, step4, step5].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "4,1,backward,1,4,Range|1,Caret,4,4,4,4|Caret,1,1|Caret,3,3,3,3|0,None,true,true",
    )?;
    Ok(())
}

#[test]
fn document_range_and_selection_placeholder_methods_support_extracted_and_inherited_calls_work()
-> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const text = document.createTextNode('ABCDE');
            host.replaceChildren(text);

            const getById = document.getElementById;
            const range = document.createRange();
            const setStart = range.setStart;
            const setEnd = range.setEnd;
            setStart.call(range, text, 1);
            setEnd.call(range, text, 4);

            const selection = document.getSelection();
            selection.removeAllRanges();
            const addRange = selection.addRange;
            addRange.call(selection, range);
            const getRangeAt = selection.getRangeAt;
            const selectedRange = getRangeAt.call(selection, 0);

            const documentReceiverError = (() => {
              const inheritor = Object.create(document);
              try {
                inheritor.getElementById('host');
                return false;
              } catch (e) {
                return String(e).includes('Document method called on incompatible receiver');
              }
            })();

            const rangeReceiverError = (() => {
              const inheritor = Object.create(range);
              try {
                inheritor.setStart(text, 0);
                return false;
              } catch (e) {
                return String(e).includes('Range method called on incompatible receiver');
              }
            })();

            const selectionReceiverError = (() => {
              const inheritor = Object.create(selection);
              try {
                inheritor.addRange(range);
                return false;
              } catch (e) {
                return String(e).includes('Selection method called on incompatible receiver');
              }
            })();

            document.getElementById('result').textContent = [
              getById.name,
              getById.length,
              getById.call(document, 'host').id,
              setStart.name,
              setStart.length,
              selectedRange.startOffset,
              selectedRange.endOffset,
              addRange.name,
              addRange.length,
              selection.toString(),
              documentReceiverError,
              rangeReceiverError,
              selectionReceiverError
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "getElementById|1|host|setStart|2|1|4|addRange|1|BCD|true|true|true",
    )?;
    Ok(())
}

#[test]
fn document_range_and_selection_methods_are_non_enumerable_and_shadowable_work() -> Result<()> {
    let html = r#"
        <div id='host'></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const host = document.getElementById('host');
            const text = document.createTextNode('ABCDE');
            host.replaceChildren(text);

            const docDesc = Object.getOwnPropertyDescriptor(document, 'getElementById');
            const docSummary = [
              docDesc.value.name,
              docDesc.value.length,
              docDesc.enumerable,
              Object.keys(document).includes('getElementById'),
              Object.getOwnPropertyNames(document).includes('getElementById')
            ].join(':');
            Object.defineProperty(document, 'getElementById', {
              value(id) { return 'shadow-doc:' + id; },
              configurable: true
            });
            const docShadow = document.getElementById('host');
            const docDeleted = delete document.getElementById;
            const docGone = document.getElementById === undefined;

            const range = document.createRange();
            const rangeDesc = Object.getOwnPropertyDescriptor(range, 'setStart');
            const rangeSummary = [
              rangeDesc.value.name,
              rangeDesc.value.length,
              rangeDesc.enumerable,
              Object.keys(range).includes('setStart'),
              Object.getOwnPropertyNames(range).includes('setStart')
            ].join(':');
            Object.defineProperty(range, 'setStart', {
              value() { return 'shadow-range'; },
              configurable: true
            });
            const rangeShadow = range.setStart(text, 2);
            const rangeOffset = range.startOffset;
            const rangeDeleted = delete range.setStart;
            const rangeGone = range.setStart === undefined;

            const selection = document.getSelection();
            selection.removeAllRanges();
            const selectionDesc = Object.getOwnPropertyDescriptor(selection, 'addRange');
            const selectionSummary = [
              selectionDesc.value.name,
              selectionDesc.value.length,
              selectionDesc.enumerable,
              Object.keys(selection).includes('addRange'),
              Object.getOwnPropertyNames(selection).includes('addRange')
            ].join(':');
            Object.defineProperty(selection, 'addRange', {
              value() { return 'shadow-selection'; },
              configurable: true
            });
            const selectionShadow = selection.addRange(range);
            const selectionCount = selection.rangeCount;
            const selectionDeleted = delete selection.addRange;
            const selectionGone = selection.addRange === undefined;

            document.getElementById('result').textContent = [
              docSummary,
              docShadow,
              docDeleted,
              docGone,
              rangeSummary,
              rangeShadow,
              rangeOffset,
              rangeDeleted,
              rangeGone,
              selectionSummary,
              selectionShadow,
              selectionCount,
              selectionDeleted,
              selectionGone
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "getElementById:1:false:false:true|shadow-doc:host|true|true|setStart:2:false:false:true|shadow-range|0|true|true|addRange:1:false:false:true|shadow-selection|0|true|true",
    )?;
    Ok(())
}

#[test]
fn selection_select_all_children_delete_from_document_and_empty() -> Result<()> {
    let html = r#"
        <div id='box'><span>ab</span><span>cd</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const selection = document.getSelection();

            selection.selectAllChildren(box);
            const rangeBefore = selection.getRangeAt(0);
            const before = [
              box.textContent,
              selection.rangeCount,
              selection.anchorOffset,
              selection.focusOffset,
              rangeBefore.startOffset,
              rangeBefore.endOffset,
              selection.containsNode(box.children[0], true),
              selection.getComposedRanges().length
            ].join(',');

            selection.deleteFromDocument();
            const rangeAfterDelete = selection.getRangeAt(0);
            const afterDelete = [
              selection.type,
              selection.anchorOffset,
              selection.focusOffset,
              rangeAfterDelete.startOffset,
              rangeAfterDelete.endOffset,
              box.textContent,
              selection.rangeCount
            ].join(',');

            selection.empty();
            const afterEmpty = [
              selection.rangeCount,
              selection.type,
              selection.anchorNode === null
            ].join(',');

            document.getElementById('result').textContent =
              [before, afterDelete, afterEmpty].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "abcd,1,0,2,0,2,true,1|Caret,0,0,0,0,,1|0,None,true",
    )?;
    Ok(())
}
