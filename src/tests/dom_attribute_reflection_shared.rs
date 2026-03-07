use super::*;

#[test]
fn attribute_reflection_html_2_3_1_boolean_attributes_use_presence_not_token_value() -> Result<()> {
    let html = r#"
        <input id='field' required='false'>
        <video id='media' controls='false'></video>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const field = document.getElementById('field');
            const media = document.getElementById('media');

            const initial =
              field.required + ':' +
              field.hasAttribute('required') + ':' +
              field.getAttribute('required') + ':' +
              media.controls + ':' +
              media.hasAttribute('controls');

            field.required = false;
            media.controls = false;
            const removed =
              field.required + ':' +
              field.hasAttribute('required') + ':' +
              media.controls + ':' +
              media.hasAttribute('controls');

            field.required = true;
            media.controls = true;
            const restored =
              field.required + ':' +
              field.hasAttribute('required') + ':' +
              media.controls + ':' +
              media.hasAttribute('controls');

            document.getElementById('result').textContent =
              initial + '|' + removed + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:false:true:true|false:false:false:false|true:true:true:true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_2_enumerated_content_editable_normalizes_and_rejects_invalid_assignment()
-> Result<()> {
    let html = r#"
        <div id='box' contenteditable='bogus'>editable</div>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initial =
              document.getElementById('box').contentEditable + ':' +
              document.getElementById('box').getAttribute('contenteditable');

            document.getElementById('box').contentEditable = 'plaintext-only';
            const valid =
              document.getElementById('box').contentEditable + ':' +
              document.getElementById('box').getAttribute('contenteditable');

            let invalid = 'no-error';
            try {
              document.getElementById('box').contentEditable = 'definitely-invalid';
            } catch (err) {
              invalid = String(err).indexOf('SyntaxError') >= 0 ? 'syntax' : String(err);
            }

            const afterInvalid =
              document.getElementById('box').contentEditable + ':' +
              document.getElementById('box').getAttribute('contenteditable');

            document.getElementById('result').textContent =
              initial + '|' + valid + '|' + invalid + '|' + afterInvalid;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "inherit:bogus|plaintext-only:plaintext-only|syntax|plaintext-only:plaintext-only",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_tabindex_parses_and_serializes_through_shared_logic()
-> Result<()> {
    let html = r#"
        <div id='box' tabindex='not-a-number'>panel</div>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const box = document.getElementById('box');
            const initial = box.tabIndex + ':' + box.getAttribute('tabindex');

            box.tabIndex = 7.9;
            const fromFloat = box.tabIndex + ':' + box.getAttribute('tabindex');

            box.tabIndex = '12';
            const fromString = box.tabIndex + ':' + box.getAttribute('tabindex');

            document.getElementById('result').textContent =
              initial + '|' + fromFloat + '|' + fromString;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "-1:not-a-number|7:7|12:12")?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_6_1_url_cite_getter_resolves_relative_against_document_base()
-> Result<()> {
    let html = r#"
        <base href='https://app.local/'>
        <blockquote id='quote' cite='/docs/rfc.html'>Quote</blockquote>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const quote = document.getElementById('quote');
            const initial = quote.cite + ':' + quote.getAttribute('cite');

            quote.cite = 'notes/spec.html';
            const assigned = quote.cite + ':' + quote.getAttribute('cite');

            quote.removeAttribute('cite');
            const removed = quote.cite + ':' + (quote.getAttribute('cite') === null);

            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + removed;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/docs/rfc.html:/docs/rfc.html|https://app.local/notes/spec.html:notes/spec.html|:true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_2_draggable_enumerated_defaults_and_keyword_normalization_work()
-> Result<()> {
    let html = r#"
        <a id='link' href='/docs/spec'>spec</a>
        <img id='img' alt='preview'>
        <div id='box' draggable='auto'>box</div>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const link = document.getElementById('link');
            const img = document.getElementById('img');
            const box = document.getElementById('box');

            const initial = link.draggable + ':' + img.draggable + ':' + box.draggable;

            box.setAttribute('draggable', 'TRUE');
            const normalizedTrue = box.draggable;

            box.setAttribute('draggable', 'invalid-token');
            const fallbackDefault = box.draggable;

            link.draggable = false;
            const assignedFalse = link.draggable + ':' + link.getAttribute('draggable');

            link.draggable = true;
            const assignedTrue = link.draggable + ':' + link.getAttribute('draggable');

            document.getElementById('result').textContent =
              initial + '|' + normalizedTrue + '|' + fallbackDefault + '|' +
              assignedFalse + '|' + assignedTrue;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:false|true|false|false:false|true:true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_2_spellcheck_and_translate_enumerated_defaults_and_keyword_normalization_work()
-> Result<()> {
    let html = r#"
        <textarea id='ta'></textarea>
        <div id='editable' contenteditable='true'>editable</div>
        <div id='plain'>plain</div>
        <p id='trans' translate='maybe'>text</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ta = document.getElementById('ta');
            const editable = document.getElementById('editable');
            const plain = document.getElementById('plain');
            const trans = document.getElementById('trans');

            const initial =
              ta.spellcheck + ':' +
              editable.spellcheck + ':' +
              plain.spellcheck + ':' +
              trans.translate;

            plain.spellcheck = true;
            const spellcheckAssigned = plain.spellcheck + ':' + plain.getAttribute('spellcheck');

            trans.translate = false;
            const translateFalse = trans.translate + ':' + trans.getAttribute('translate');

            trans.setAttribute('translate', 'YES');
            const translateTrue = trans.translate + ':' + trans.getAttribute('translate');

            document.getElementById('result').textContent =
              initial + '|' + spellcheckAssigned + '|' + translateFalse + '|' + translateTrue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:false:true|true:true|false:no|true:YES",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_6_1_url_form_action_getter_resolves_submitter_owner_and_document_defaults()
-> Result<()> {
    let html = r#"
        <form id='f' action='/submit'></form>
        <button id='owned-button' type='submit' form='f'>submit</button>
        <input id='owned-input' type='submit' form='f' value='send'>
        <button id='orphan-button' type='submit'>orphan</button>
        <input id='orphan-input' type='submit' value='orphan-input'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('f');
            const ownedButton = document.getElementById('owned-button');
            const ownedInput = document.getElementById('owned-input');
            const orphanButton = document.getElementById('orphan-button');
            const orphanInput = document.getElementById('orphan-input');

            const initial = [
              ownedButton.formAction + ':' + (ownedButton.getAttribute('formaction') === null),
              ownedInput.formAction + ':' + (ownedInput.getAttribute('formaction') === null),
              orphanButton.formAction + ':' + (orphanButton.getAttribute('formaction') === null),
              orphanInput.formAction + ':' + (orphanInput.getAttribute('formaction') === null)
            ].join(',');

            form.action = 'next';
            const linked = [ownedButton.formAction, ownedInput.formAction].join(',');

            ownedButton.formAction = 'button-submit';
            ownedInput.formAction = 'input-submit';
            const assigned = [
              ownedButton.formAction + ':' + ownedButton.getAttribute('formaction'),
              ownedInput.formAction + ':' + ownedInput.getAttribute('formaction')
            ].join(',');

            ownedButton.removeAttribute('formaction');
            ownedInput.removeAttribute('formaction');
            form.removeAttribute('action');
            const removed = [
              ownedButton.formAction + ':' + (ownedButton.getAttribute('formaction') === null),
              ownedInput.formAction + ':' + (ownedInput.getAttribute('formaction') === null),
              orphanButton.formAction,
              orphanInput.formAction
            ].join(',');

            document.getElementById('result').textContent =
              initial + '|' + linked + '|' + assigned + '|' + removed;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/page/index.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/submit:true,https://app.local/submit:true,https://app.local/page/index.html:true,https://app.local/page/index.html:true|https://app.local/page/next,https://app.local/page/next|https://app.local/page/button-submit:button-submit,https://app.local/page/input-submit:input-submit|https://app.local/page/index.html:true,https://app.local/page/index.html:true,https://app.local/page/index.html,https://app.local/page/index.html",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_size_span_canvas_dimensions_boundary_rules_work()
-> Result<()> {
    let html = r#"
        <table>
          <col id='col' span='0'>
          <tr><td>cell</td></tr>
        </table>
        <select id='sel' size='-2'>
          <option>One</option>
        </select>
        <canvas id='canvas' width='oops' height='-5'></canvas>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const sel = document.getElementById('sel');
            const col = document.getElementById('col');
            const canvas = document.getElementById('canvas');

            const initial =
              sel.size + ':' + sel.getAttribute('size') + ':' +
              col.span + ':' + col.getAttribute('span') + ':' +
              canvas.width + 'x' + canvas.height + ':' +
              canvas.getAttribute('width') + ':' + canvas.getAttribute('height');

            sel.size = -3;
            col.span = 0;
            canvas.width = -7;
            canvas.height = 5.9;

            const updated =
              sel.size + ':' + sel.getAttribute('size') + ':' +
              col.span + ':' + col.getAttribute('span') + ':' +
              canvas.width + 'x' + canvas.height + ':' +
              canvas.getAttribute('width') + ':' + canvas.getAttribute('height');

            document.getElementById('result').textContent = initial + '|' + updated;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:-2:1:0:300x150:oops:-5|1:0:1:1:0x5:0:5")?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_2_enumerated_invalid_empty_and_case_variants_follow_default_matrix()
-> Result<()> {
    let html = r#"
        <a id='link' href='/docs'>docs</a>
        <div id='box' draggable='TrUe'>box</div>
        <div id='plain'>plain</div>
        <p id='trans'>translate</p>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const link = document.getElementById('link');
            const box = document.getElementById('box');
            const plain = document.getElementById('plain');
            const trans = document.getElementById('trans');

            const before = [
              link.draggable,
              box.draggable,
              plain.spellcheck,
              trans.translate
            ].join(':');

            box.setAttribute('draggable', 'FALSE');
            const dragFalse = box.draggable + ':' + box.getAttribute('draggable');
            box.setAttribute('draggable', 'invalid');
            const dragInvalid = box.draggable;
            box.setAttribute('draggable', '');
            const dragEmpty = box.draggable;

            plain.setAttribute('spellcheck', '');
            const spellEmpty = plain.spellcheck + ':' + plain.getAttribute('spellcheck');
            plain.setAttribute('spellcheck', 'FaLsE');
            const spellFalse = plain.spellcheck + ':' + plain.getAttribute('spellcheck');
            plain.setAttribute('spellcheck', 'invalid');
            const spellInvalid = plain.spellcheck;

            trans.setAttribute('translate', 'NO');
            const transNo = trans.translate + ':' + trans.getAttribute('translate');
            trans.setAttribute('translate', '');
            const transEmpty = trans.translate + ':' + trans.getAttribute('translate');
            trans.setAttribute('translate', 'invalid');
            const transInvalid = trans.translate;

            document.getElementById('result').textContent = [
              before,
              dragFalse,
              dragInvalid,
              dragEmpty,
              spellEmpty,
              spellFalse,
              spellInvalid,
              transNo,
              transEmpty,
              transInvalid
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:false:true|false:FALSE|false|false|true:|false:FaLsE|false|false:NO|true:|true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_6_1_url_action_src_form_action_poster_and_cite_getters_resolve_relative_and_empty_states()
-> Result<()> {
    let html = r#"
        <form id='f' action='/submit'></form>
        <img id='img' src='/img/start.png' alt='preview'>
        <button id='submit' type='submit' formaction='/override'>send</button>
        <video id='video' poster='/img/poster.png'></video>
        <blockquote id='quote' cite='/spec/rfc'></blockquote>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const f = document.getElementById('f');
            const img = document.getElementById('img');
            const submit = document.getElementById('submit');
            const video = document.getElementById('video');
            const quote = document.getElementById('quote');

            const before = [
              f.action + ':' + f.getAttribute('action'),
              img.src + ':' + img.getAttribute('src'),
              submit.formAction + ':' + submit.getAttribute('formaction'),
              video.poster + ':' + video.getAttribute('poster'),
              quote.cite + ':' + quote.getAttribute('cite')
            ].join(',');

            f.action = 'next';
            img.src = 'img/next.png';
            submit.formAction = 'send';
            video.poster = 'img/poster-next.png';
            quote.cite = 'spec/next';
            const assigned = [
              f.action + ':' + f.getAttribute('action'),
              img.src + ':' + img.getAttribute('src'),
              submit.formAction + ':' + submit.getAttribute('formaction'),
              video.poster + ':' + video.getAttribute('poster'),
              quote.cite + ':' + quote.getAttribute('cite')
            ].join(',');

            f.removeAttribute('action');
            img.removeAttribute('src');
            submit.removeAttribute('formaction');
            video.removeAttribute('poster');
            quote.removeAttribute('cite');
            const removed = [
              f.action + ':' + (f.getAttribute('action') === null),
              img.src + ':' + (img.getAttribute('src') === null),
              submit.formAction + ':' + (submit.getAttribute('formaction') === null),
              video.poster + ':' + (video.getAttribute('poster') === null),
              quote.cite + ':' + (quote.getAttribute('cite') === null)
            ].join(',');

            document.getElementById('result').textContent =
              before + '|' + assigned + '|' + removed;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/base/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/submit:/submit,https://app.local/img/start.png:/img/start.png,https://app.local/override:/override,https://app.local/img/poster.png:/img/poster.png,https://app.local/spec/rfc:/spec/rfc|https://app.local/base/next:next,https://app.local/base/img/next.png:img/next.png,https://app.local/base/send:send,https://app.local/base/img/poster-next.png:img/poster-next.png,https://app.local/base/spec/next:spec/next|https://app.local/base/page.html:true,:true,https://app.local/base/page.html:true,:true,:true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_col_row_span_and_select_size_clamp_rules_work()
-> Result<()> {
    let html = r#"
        <table>
          <colgroup>
            <col id='col' span='5000'>
          </colgroup>
          <tbody>
            <tr><td id='cell' colspan='2000' rowspan='70000'>cell</td></tr>
          </tbody>
        </table>
        <select id='sel'>
          <option>One</option>
        </select>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const col = document.getElementById('col');
            const cell = document.getElementById('cell');
            const sel = document.getElementById('sel');

            const initial = [
              col.span + ':' + col.getAttribute('span'),
              cell.colSpan + ':' + cell.getAttribute('colspan'),
              cell.rowSpan + ':' + cell.getAttribute('rowspan'),
              sel.size + ':' + (sel.getAttribute('size') === null)
            ].join(',');

            col.span = 5000;
            cell.colSpan = 0;
            cell.rowSpan = 0;
            sel.size = -99;
            const clampedFloor = [
              col.span + ':' + col.getAttribute('span'),
              cell.colSpan + ':' + cell.getAttribute('colspan'),
              cell.rowSpan + ':' + cell.getAttribute('rowspan'),
              sel.size + ':' + sel.getAttribute('size')
            ].join(',');

            cell.colSpan = 4096;
            cell.rowSpan = 999999;
            sel.size = 3.9;
            const clampedUpper = [
              cell.colSpan + ':' + cell.getAttribute('colspan'),
              cell.rowSpan + ':' + cell.getAttribute('rowspan'),
              sel.size + ':' + sel.getAttribute('size')
            ].join(',');

            document.getElementById('result').textContent =
              initial + '|' + clampedFloor + '|' + clampedUpper;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1000:5000,1000:2000,65534:70000,1:true|1000:1000,1:1,0:0,1:0|1000:1000,65534:65534,3:3",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_6_1_url_missing_defaults_for_form_action_img_src_and_anchor_href_work()
-> Result<()> {
    let html = r#"
        <form id='f'></form>
        <img id='img' alt='preview'>
        <a id='link'>link</a>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const f = document.getElementById('f');
            const img = document.getElementById('img');
            const link = document.getElementById('link');

            const initial = [
              f.action,
              img.src,
              link.href,
              f.getAttribute('action') === null,
              img.getAttribute('src') === null,
              link.getAttribute('href') === null
            ].join(':');

            f.setAttribute('action', '');
            img.setAttribute('src', '');
            link.setAttribute('href', '');
            const emptyAttrs = [
              f.action + ':' + f.getAttribute('action'),
              img.src + ':' + img.getAttribute('src'),
              link.href + ':' + link.getAttribute('href')
            ].join(',');

            f.removeAttribute('action');
            img.removeAttribute('src');
            link.removeAttribute('href');
            const removed = [
              f.action,
              img.src,
              link.href,
              f.getAttribute('action') === null,
              img.getAttribute('src') === null,
              link.getAttribute('href') === null
            ].join(':');

            document.getElementById('result').textContent =
              initial + '|' + emptyAttrs + '|' + removed;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/base/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/base/page.html:::true:true:true|https://app.local/base/page.html:,https://app.local/base/page.html:,https://app.local/base/page.html:|https://app.local/base/page.html:::true:true:true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_parser_fast_path_matches_col_row_span_reflection_rules()
-> Result<()> {
    let html = r#"
        <table>
          <colgroup>
            <col id='col' span='4'>
          </colgroup>
          <tbody>
            <tr><td id='cell' colspan='2' rowspan='3'>cell</td></tr>
          </tbody>
        </table>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const cell = document.getElementById('cell');
            const col = document.getElementById('col');

            const initial = [
              cell.colSpan,
              cell.rowSpan,
              col.span
            ].join(':');

            cell.colSpan = 0;
            cell.rowSpan = 70000;
            col.span = 0;

            const updated = [
              cell.colSpan + ':' + cell.getAttribute('colspan'),
              cell.rowSpan + ':' + cell.getAttribute('rowspan'),
              col.span + ':' + col.getAttribute('span')
            ].join(',');

            document.getElementById('result').textContent = initial + '|' + updated;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:3:4|1:1,65534:65534,1:1")?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_input_textarea_and_select_size_rows_cols_and_maxlength_work()
-> Result<()> {
    let html = r#"
        <input id='field' type='text' size='0' maxlength='oops'>
        <textarea id='ta' rows='0' cols='-4' maxlength='nan'></textarea>
        <select id='sel' size='0'>
          <option>One</option>
        </select>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const field = document.getElementById('field');
            const ta = document.getElementById('ta');
            const sel = document.getElementById('sel');

            const initial = [
              field.size + ':' + field.getAttribute('size'),
              field.maxLength + ':' + field.getAttribute('maxlength'),
              ta.rows + ':' + ta.getAttribute('rows'),
              ta.cols + ':' + ta.getAttribute('cols'),
              ta.maxLength + ':' + ta.getAttribute('maxlength'),
              sel.size + ':' + sel.getAttribute('size')
            ].join(',');

            field.size = -7;
            field.maxLength = 12;
            ta.rows = 0;
            ta.cols = -3;
            ta.maxLength = 9;
            sel.size = -2;

            const updated = [
              field.size + ':' + field.getAttribute('size'),
              field.maxLength + ':' + field.getAttribute('maxlength'),
              ta.rows + ':' + ta.getAttribute('rows'),
              ta.cols + ':' + ta.getAttribute('cols'),
              ta.maxLength + ':' + ta.getAttribute('maxlength'),
              sel.size + ':' + sel.getAttribute('size')
            ].join(',');

            document.getElementById('result').textContent = initial + '|' + updated;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "20:0,-1:oops,2:0,20:-4,-1:nan,1:0|1:1,12:12,1:1,1:1,9:9,1:0",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_input_minlength_and_maxlength_boundary_and_roundtrip_work()
-> Result<()> {
    let html = r#"
        <input id='field' type='text' minlength='oops' maxlength='oops'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const field = document.getElementById('field');

            const initial = [
              field.minLength + ':' + field.getAttribute('minlength'),
              field.maxLength + ':' + field.getAttribute('maxlength')
            ].join(',');

            field.minLength = -5;
            field.maxLength = -9;
            const negative = [
              field.minLength + ':' + field.getAttribute('minlength'),
              field.maxLength + ':' + field.getAttribute('maxlength')
            ].join(',');

            field.minLength = NaN;
            field.maxLength = NaN;
            const nan = [
              field.minLength + ':' + field.getAttribute('minlength'),
              field.maxLength + ':' + field.getAttribute('maxlength')
            ].join(',');

            field.minLength = 'token';
            field.maxLength = 'token';
            const nonNumericString = [
              field.minLength + ':' + field.getAttribute('minlength'),
              field.maxLength + ':' + field.getAttribute('maxlength')
            ].join(',');

            field.minLength = '7';
            field.maxLength = '12';
            const numericString = [
              field.minLength + ':' + field.getAttribute('minlength'),
              field.maxLength + ':' + field.getAttribute('maxlength')
            ].join(',');

            field.setAttribute('minlength', '-3');
            field.setAttribute('maxlength', 'bad');
            const fromInvalidAttr = [
              field.minLength + ':' + field.getAttribute('minlength'),
              field.maxLength + ':' + field.getAttribute('maxlength')
            ].join(',');

            document.getElementById('result').textContent = [
              initial,
              negative,
              nan,
              nonNumericString,
              numericString,
              fromInvalidAttr
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "-1:oops,-1:oops|-1:-1,-1:-1|0:0,0:0|0:0,0:0|7:7,12:12|-1:-3,-1:bad",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_textarea_rows_cols_upper_bound_and_token_matrix_work()
-> Result<()> {
    let html = r#"
        <textarea id='ta' rows='NaN' cols='0'></textarea>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const ta = document.getElementById('ta');

            const initial = [
              ta.rows + ':' + ta.getAttribute('rows'),
              ta.cols + ':' + ta.getAttribute('cols')
            ].join(',');

            ta.setAttribute('rows', '-5');
            ta.setAttribute('cols', 'words');
            const tokenMatrix = [
              ta.rows + ':' + ta.getAttribute('rows'),
              ta.cols + ':' + ta.getAttribute('cols')
            ].join(',');

            ta.rows = 0;
            ta.cols = -2;
            const clampedFloor = [
              ta.rows + ':' + ta.getAttribute('rows'),
              ta.cols + ':' + ta.getAttribute('cols')
            ].join(',');

            ta.rows = 2147483647;
            ta.cols = 2147483647;
            const upperBound = [
              ta.rows + ':' + ta.getAttribute('rows'),
              ta.cols + ':' + ta.getAttribute('cols')
            ].join(',');

            ta.rows = 4294967295;
            ta.cols = 9000000000;
            const clampedUpper = [
              ta.rows + ':' + ta.getAttribute('rows'),
              ta.cols + ':' + ta.getAttribute('cols')
            ].join(',');

            document.getElementById('result').textContent = [
              initial,
              tokenMatrix,
              clampedFloor,
              upperBound,
              clampedUpper
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "2:NaN,20:0|2:-5,20:words|1:1,1:1|2147483647:2147483647,2147483647:2147483647|2147483647:2147483647,2147483647:2147483647",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_2_enumerated_dir_autocapitalize_autocomplete_missing_invalid_and_case_variants_work()
-> Result<()> {
    let html = r#"
        <div id='plain'>plain</div>
        <bdi id='bdi'>bdi</bdi>
        <input id='field' type='text'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const plain = document.getElementById('plain');
            const bdi = document.getElementById('bdi');
            const field = document.getElementById('field');

            const initial = [
              plain.dir + ':' + (plain.getAttribute('dir') === null),
              bdi.dir + ':' + (bdi.getAttribute('dir') === null),
              field.autocapitalize + ':' + (field.getAttribute('autocapitalize') === null),
              field.autocomplete + ':' + (field.getAttribute('autocomplete') === null)
            ].join(',');

            plain.setAttribute('dir', 'RtL');
            field.setAttribute('autocapitalize', 'WoRdS');
            field.setAttribute('autocomplete', 'ON');
            const caseVariants = [
              plain.dir + ':' + plain.getAttribute('dir'),
              field.autocapitalize + ':' + field.getAttribute('autocapitalize'),
              field.autocomplete + ':' + field.getAttribute('autocomplete')
            ].join(',');

            plain.setAttribute('dir', 'invalid-dir');
            field.setAttribute('autocapitalize', 'unexpected');
            field.setAttribute('autocomplete', 'not-a-keyword');
            const invalid = [
              plain.dir + ':' + plain.getAttribute('dir'),
              field.autocapitalize + ':' + field.getAttribute('autocapitalize'),
              field.autocomplete + ':' + field.getAttribute('autocomplete')
            ].join(',');

            plain.removeAttribute('dir');
            field.removeAttribute('autocapitalize');
            field.removeAttribute('autocomplete');
            const removed = [
              plain.dir + ':' + (plain.getAttribute('dir') === null),
              field.autocapitalize + ':' + (field.getAttribute('autocapitalize') === null),
              field.autocomplete + ':' + (field.getAttribute('autocomplete') === null)
            ].join(',');

            document.getElementById('result').textContent = [
              initial,
              caseVariants,
              invalid,
              removed
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        ":true,auto:true,:true,:true|RtL:RtL,WoRdS:WoRdS,ON:ON|invalid-dir:invalid-dir,unexpected:unexpected,not-a-keyword:not-a-keyword|:true,:true,:true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_6_1_url_anchor_search_hash_and_credentials_delimiter_normalization_work()
-> Result<()> {
    let html = r#"
        <a id='link' href='https://u:p@example.com/path?x=1#start'>link</a>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const link = document.getElementById('link');

            const initial = [
              link.search,
              link.hash,
              link.href
            ].join(',');

            link.search = '';
            link.hash = '';
            const cleared = [
              link.search,
              link.hash,
              link.href
            ].join(',');

            link.search = 'q=2';
            link.hash = 'frag';
            const prefixed = [
              link.search,
              link.hash,
              link.href
            ].join(',');

            link.search = '?';
            link.hash = '#';
            const delimitersOnly = [
              link.search,
              link.hash,
              link.href
            ].join(',');

            link.username = '';
            link.password = 'secret';
            const passwordOnly = [
              link.username,
              link.password,
              link.href
            ].join(',');

            link.password = '';
            const credentialsCleared = [
              link.username,
              link.password,
              link.href
            ].join(',');

            document.getElementById('result').textContent = [
              initial,
              cleared,
              prefixed,
              delimitersOnly,
              passwordOnly,
              credentialsCleared
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "?x=1,#start,https://u:p@example.com/path?x=1#start|,,https://u:p@example.com/path|?q=2,#frag,https://u:p@example.com/path?q=2#frag|,,https://u:p@example.com/path?#|,secret,https://:secret@example.com/path?#|,,https://example.com/path?#",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_6_1_url_base_href_changes_update_baseuri_form_action_and_submitter_formaction()
-> Result<()> {
    let html = r#"
        <base id='base' href='https://app.local/v1/'>
        <form id='f' action='submit'></form>
        <button id='btn' type='submit' form='f'>button</button>
        <input id='inp' type='submit' form='f' value='input'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const base = document.getElementById('base');
            const form = document.getElementById('f');
            const btn = document.getElementById('btn');
            const inp = document.getElementById('inp');

            const initial = [
              document.baseURI,
              form.action,
              btn.formAction,
              inp.formAction
            ].join(',');

            base.setAttribute('href', '/v2/');
            const movedBase = [
              document.baseURI,
              form.action,
              btn.formAction,
              inp.formAction
            ].join(',');

            btn.formAction = 'override';
            inp.formAction = 'send';
            const override = [
              btn.formAction + ':' + btn.getAttribute('formaction'),
              inp.formAction + ':' + inp.getAttribute('formaction')
            ].join(',');

            btn.removeAttribute('formaction');
            inp.removeAttribute('formaction');
            base.setAttribute('href', '');
            const resetBase = [
              document.baseURI,
              form.action,
              btn.formAction,
              inp.formAction
            ].join(',');

            form.action = 'next';
            const formAssigned = [
              form.action,
              btn.formAction,
              inp.formAction
            ].join(',');

            document.getElementById('result').textContent = [
              initial,
              movedBase,
              override,
              resetBase,
              formAssigned
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/root/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/v1/,https://app.local/v1/submit,https://app.local/v1/submit,https://app.local/v1/submit|https://app.local/v2/,https://app.local/v2/submit,https://app.local/v2/submit,https://app.local/v2/submit|https://app.local/v2/override:override,https://app.local/v2/send:send|https://app.local/root/page.html,https://app.local/root/submit,https://app.local/root/submit,https://app.local/root/submit|https://app.local/root/next,https://app.local/root/next,https://app.local/root/next",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_input_min_max_step_invalid_tokens_nan_infinity_and_boundaries_affect_validity()
-> Result<()> {
    let html = r#"
        <input id='num' type='number' min='foo' max='Infinity' step='NaN' value='3'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const num = document.getElementById('num');

            const initial = [
              num.validity.rangeUnderflow,
              num.validity.rangeOverflow,
              num.validity.stepMismatch,
              num.validity.badInput,
              num.checkValidity()
            ].join(':');

            num.value = '3.5';
            const defaultStep = [
              num.validity.stepMismatch,
              num.checkValidity()
            ].join(':');

            num.step = '0.5';
            const explicitStep = [
              num.validity.stepMismatch,
              num.checkValidity()
            ].join(':');

            num.min = '2';
            num.max = '4';
            num.value = '1.5';
            const underflow = [
              num.validity.rangeUnderflow,
              num.validity.rangeOverflow,
              num.validity.stepMismatch,
              num.checkValidity()
            ].join(':');

            num.value = '4.5';
            const overflow = [
              num.validity.rangeUnderflow,
              num.validity.rangeOverflow,
              num.validity.stepMismatch,
              num.checkValidity()
            ].join(':');

            num.step = 'any';
            num.value = '3.3';
            const stepAny = [
              num.validity.stepMismatch,
              num.checkValidity()
            ].join(':');

            num.min = 'NaN';
            num.max = '-Infinity';
            num.step = 'Infinity';
            num.value = '3.7';
            const invalidTokens = [
              num.validity.rangeUnderflow,
              num.validity.rangeOverflow,
              num.validity.stepMismatch,
              num.checkValidity()
            ].join(':');

            document.getElementById('result').textContent = [
              initial,
              defaultStep,
              explicitStep,
              underflow,
              overflow,
              stepAny,
              invalidTokens
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:false:false:false:true|true:false|false:true|true:false:false:false|false:true:false:false|false:true|false:false:true:false",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_minlength_maxlength_threshold_boundaries_for_input_and_textarea_validity()
-> Result<()> {
    let html = r#"
        <input id='field' type='text' minlength='3' maxlength='5'>
        <textarea id='ta' minlength='2' maxlength='4'></textarea>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const field = document.getElementById('field');
            const ta = document.getElementById('ta');

            const initial = [
              field.validity.tooShort + ':' + field.validity.tooLong + ':' + field.checkValidity(),
              ta.validity.tooShort + ':' + ta.validity.tooLong + ':' + ta.checkValidity()
            ].join(',');

            field.value = 'ab';
            ta.value = 'a';
            const belowMin = [
              field.validity.tooShort + ':' + field.validity.tooLong + ':' + field.checkValidity(),
              ta.validity.tooShort + ':' + ta.validity.tooLong + ':' + ta.checkValidity()
            ].join(',');

            field.value = 'abc';
            ta.value = 'ab';
            const atMin = [
              field.validity.tooShort + ':' + field.validity.tooLong + ':' + field.checkValidity(),
              ta.validity.tooShort + ':' + ta.validity.tooLong + ':' + ta.checkValidity()
            ].join(',');

            field.value = 'abcdef';
            ta.value = 'abcde';
            const aboveMax = [
              field.validity.tooShort + ':' + field.validity.tooLong + ':' + field.checkValidity(),
              ta.validity.tooShort + ':' + ta.validity.tooLong + ':' + ta.checkValidity()
            ].join(',');

            field.value = 'abcde';
            ta.value = 'abcd';
            const atMax = [
              field.validity.tooShort + ':' + field.validity.tooLong + ':' + field.checkValidity(),
              ta.validity.tooShort + ':' + ta.validity.tooLong + ':' + ta.checkValidity()
            ].join(',');

            document.getElementById('result').textContent = [
              initial,
              belowMin,
              atMin,
              aboveMax,
              atMax
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:false:true,false:false:true|true:false:false,true:false:false|false:false:true,false:false:true|false:true:false,false:true:false|false:false:true,false:false:true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_parser_fast_path_matches_action_formaction_autocomplete_size_and_length_reflection()
-> Result<()> {
    let html = r#"
        <form id='f' action='/submit'></form>
        <button id='btn' type='submit' form='f'>button</button>
        <input id='field' type='text' size='0' minlength='oops' maxlength='oops' autocomplete='off'>
        <textarea id='ta' rows='0' cols='0'></textarea>
        <select id='sel' size='0'>
          <option>One</option>
        </select>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initial = [
              document.getElementById('f').action,
              document.getElementById('btn').formAction,
              document.getElementById('field').autocomplete,
              document.getElementById('field').size,
              document.getElementById('field').minLength,
              document.getElementById('field').maxLength,
              document.getElementById('ta').rows,
              document.getElementById('ta').cols,
              document.getElementById('sel').size
            ].join(',');

            document.getElementById('f').action = 'next';
            document.getElementById('btn').formAction = 'send';
            document.getElementById('field').autocomplete = 'on';
            document.getElementById('field').size = -3;
            document.getElementById('field').minLength = '7';
            document.getElementById('field').maxLength = '12';
            document.getElementById('ta').rows = 0;
            document.getElementById('ta').cols = -1;
            document.getElementById('sel').size = -2;

            const updated = [
              document.getElementById('f').action,
              document.getElementById('btn').formAction,
              document.getElementById('field').autocomplete,
              document.getElementById('field').size,
              document.getElementById('field').minLength,
              document.getElementById('field').maxLength,
              document.getElementById('ta').rows,
              document.getElementById('ta').cols,
              document.getElementById('sel').size
            ].join(',');

            document.getElementById('btn').removeAttribute('formaction');
            const removed = [
              document.getElementById('f').action,
              document.getElementById('btn').formAction,
              document.getElementById('btn').getAttribute('formaction') === null
            ].join(',');

            document.getElementById('result').textContent = [
              initial,
              updated,
              removed
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/base/page.html", html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/submit,https://app.local/submit,off,20,-1,-1,2,20,1|https://app.local/base/next,https://app.local/base/send,on,1,7,12,1,1,1|https://app.local/base/next,https://app.local/base/next,true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_6_1_url_anchor_setter_special_and_opaque_protocol_host_port_pathname_matrix_work()
-> Result<()> {
    let html = r#"
        <a id='special' href='https://user:pw@example.com:8443/a/b?x=1#h'>special</a>
        <a id='opaque' href='mailto:person@example.com?subject=Hi#frag'>opaque</a>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const special = document.getElementById('special');
            const opaque = document.getElementById('opaque');

            const initial = [
              special.protocol + ',' + special.host + ',' + special.pathname + ',' + special.href,
              opaque.protocol + ',' + opaque.host + ',' + opaque.pathname + ',' + opaque.href
            ].join(';');

            special.protocol = 'http:';
            special.host = 'api.example.test:9090';
            special.hostname = 'cdn.example.test';
            special.port = '7070';
            special.pathname = 'docs';

            opaque.protocol = 'https:';
            opaque.host = 'ignored.test:1234';
            opaque.hostname = 'ignored2.test';
            opaque.port = '5678';
            opaque.pathname = 'new/path';

            const updated = [
              special.protocol + ',' + special.host + ',' + special.pathname + ',' + special.href,
              opaque.protocol + ',' + opaque.host + ',' + opaque.pathname + ',' + opaque.href
            ].join(';');

            document.getElementById('result').textContent = [
              initial,
              updated
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https:,example.com:8443,/a/b,https://user:pw@example.com:8443/a/b?x=1#h;mailto:,,person@example.com,mailto:person@example.com?subject=Hi#frag|http:,cdn.example.test:7070,/docs,http://user:pw@cdn.example.test:7070/docs?x=1#h;https:,,new/path,https:new/path?subject=Hi#frag",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_6_1_url_anchor_username_password_setter_is_noop_for_no_host_and_file_urls()
-> Result<()> {
    let html = r#"
        <a id='mail' href='mailto:m.bluth@example.com?subject=Hi'>mail</a>
        <a id='data' href='data:text/plain,hello'>data</a>
        <a id='file' href='file:///Users/kazuyoshitoshiya/report.txt'>file</a>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const mail = document.getElementById('mail');
            const data = document.getElementById('data');
            const file = document.getElementById('file');

            const initial = [
              mail.username + ':' + mail.password + ':' + mail.href,
              data.username + ':' + data.password + ':' + data.href,
              file.username + ':' + file.password + ':' + file.href
            ].join(';');

            mail.username = 'alice';
            mail.password = 'secret';
            data.username = 'alice';
            data.password = 'secret';
            file.username = 'alice';
            file.password = 'secret';

            const updated = [
              mail.username + ':' + mail.password + ':' + mail.href,
              data.username + ':' + data.password + ':' + data.href,
              file.username + ':' + file.password + ':' + file.href
            ].join(';');

            document.getElementById('result').textContent = [
              initial,
              updated
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "::mailto:m.bluth@example.com?subject=Hi;::data:text/plain,hello;::file:///Users/kazuyoshitoshiya/report.txt|::mailto:m.bluth@example.com?subject=Hi;::data:text/plain,hello;::file:///Users/kazuyoshitoshiya/report.txt",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_validity_recomputes_after_min_max_step_mutations_across_supported_types()
-> Result<()> {
    let html = r#"
        <input id='n' type='number' value='5' min='0' max='10' step='1'>
        <input id='r' type='range' value='7' min='0' max='10' step='5'>
        <input id='d' type='date' value='2026-01-10' min='2026-01-01' max='2026-01-31' step='1'>
        <input id='t' type='time' value='10:37' min='09:00' max='11:00' step='60'>
        <input id='dt' type='datetime-local' value='2026-01-01T10:37' min='2026-01-01T09:00' max='2026-01-01T12:00' step='60'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          function state(id) {
            const el = document.getElementById(id);
            return [
              el.validity.rangeUnderflow,
              el.validity.rangeOverflow,
              el.validity.stepMismatch,
              el.checkValidity()
            ].join(':');
          }

          document.getElementById('run').addEventListener('click', () => {
            const initial = [
              state('n'),
              state('r'),
              state('d'),
              state('t'),
              state('dt')
            ].join(',');

            const n = document.getElementById('n');
            n.min = '6';
            const nUnderflow = state('n');
            n.min = '0';
            n.max = '4';
            const nOverflow = state('n');
            n.max = '10';
            n.step = '2';
            const nStepMismatch = state('n');
            n.step = 'any';
            const nStepAny = state('n');

            const r = document.getElementById('r');
            const rInitialMismatch = state('r');
            r.min = '2';
            const rMinBaseAligned = state('r');

            const d = document.getElementById('d');
            d.min = '2026-01-15';
            const dUnderflow = state('d');
            d.min = '2026-01-01';
            d.max = '2026-01-05';
            const dOverflow = state('d');
            d.max = '2026-01-31';
            d.step = '7';
            const dStepMismatch = state('d');
            d.min = '2026-01-03';
            const dStepAligned = state('d');

            const t = document.getElementById('t');
            t.min = '10:40';
            const tUnderflow = state('t');
            t.min = '09:00';
            t.max = '10:00';
            const tOverflow = state('t');
            t.max = '11:00';
            t.step = '900';
            const tStepMismatch = state('t');
            t.min = '10:07';
            const tStepAligned = state('t');

            const dt = document.getElementById('dt');
            dt.min = '2026-01-01T10:40';
            const dtUnderflow = state('dt');
            dt.min = '2026-01-01T09:00';
            dt.max = '2026-01-01T10:00';
            const dtOverflow = state('dt');
            dt.max = '2026-01-01T12:00';
            dt.step = '900';
            const dtStepMismatch = state('dt');
            dt.min = '2026-01-01T10:07';
            const dtStepAligned = state('dt');

            document.getElementById('result').textContent = [
              initial,
              nUnderflow,
              nOverflow,
              nStepMismatch,
              nStepAny,
              rInitialMismatch,
              rMinBaseAligned,
              dUnderflow,
              dOverflow,
              dStepMismatch,
              dStepAligned,
              tUnderflow,
              tOverflow,
              tStepMismatch,
              tStepAligned,
              dtUnderflow,
              dtOverflow,
              dtStepMismatch,
              dtStepAligned
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:false:false:true,false:false:false:true,false:false:false:true,false:false:false:true,false:false:false:true|true:false:false:false|false:true:false:false|false:false:true:false|false:false:false:true|false:false:false:true|false:false:false:true|true:false:false:false|false:true:false:false|false:false:true:false|false:false:false:true|true:false:false:false|false:true:false:false|false:false:true:false|false:false:false:true|true:false:false:false|false:true:false:false|false:false:true:false|false:false:false:true",
    )?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_numeric_step_base_prefers_min_then_value_attribute_and_rounding_boundary_work()
-> Result<()> {
    let html = r#"
        <input id='n' type='number' value='0.2' step='0.1'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const n = document.getElementById('n');

            n.value = '0.3';
            const valueBase = [
              n.validity.stepMismatch,
              n.checkValidity()
            ].join(':');

            n.min = '0.25';
            const minBase = [
              n.validity.stepMismatch,
              n.checkValidity()
            ].join(':');

            n.value = '0.35000000005';
            const nearBoundary = [
              n.validity.stepMismatch,
              n.checkValidity()
            ].join(':');

            n.value = '0.3501';
            const farFromBoundary = [
              n.validity.stepMismatch,
              n.checkValidity()
            ].join(':');

            document.getElementById('result').textContent = [
              valueBase,
              minBase,
              nearBoundary,
              farFromBoundary
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true|true:false|false:true|true:false")?;
    Ok(())
}

#[test]
fn attribute_reflection_html_2_3_3_parser_fast_path_matches_min_max_step_reflection_with_bracket_and_member_chain_access()
-> Result<()> {
    let html = r#"
        <input id='num' type='number' min='1' max='9' step='2' value='5'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const initial = [
              document.getElementById('num').min,
              document.getElementById('num').max,
              document.getElementById('num').step,
              document.getElementById('num').validity['stepMismatch']
            ].join(',');

            document.getElementById('num').min = '3';
            document.getElementById('num').max = '11';
            document.getElementById('num').step = '4';
            const updated = [
              document.getElementById('num').min + ':' + document.getElementById('num').getAttribute('min'),
              document.getElementById('num').max + ':' + document.getElementById('num').getAttribute('max'),
              document.getElementById('num').step + ':' + document.getElementById('num').getAttribute('step'),
              document.getElementById('num').validity['stepMismatch']
            ].join(',');

            document.getElementById('num').min = '';
            document.getElementById('num').step = 'any';
            const cleared = [
              document.getElementById('num').min + ':' + document.getElementById('num').getAttribute('min'),
              document.getElementById('num').step + ':' + document.getElementById('num').getAttribute('step'),
              document.getElementById('num').validity['stepMismatch']
            ].join(',');

            document.getElementById('result').textContent = [
              initial,
              updated,
              cleared
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1,9,2,false|3:3,11:11,4:4,true|:,any:any,false")?;
    Ok(())
}
