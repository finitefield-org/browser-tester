use super::*;

#[test]
fn outer_html_and_element_member_methods_work() -> Result<()> {
    let html = r#"
        <div id='box' data-x='1'>
          <span class='item' id='i1'>A</span>
          <span class='item' id='i2'>B</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            const names = box.getAttributeNames();
            const hasBefore = box.hasAttributes();
            const toggledOn = box.toggleAttribute('hidden');
            const toggledOff = box.toggleAttribute('hidden', false);
            const picked = box.querySelector('.item').id;
            const allItems = box.querySelectorAll('.item');
            const total = allItems.length;
            const byClassNodes = box.getElementsByClassName('item');
            const byTagNodes = box.getElementsByTagName('span');
            const byClass = byClassNodes.length;
            const byTag = byTagNodes.length;
            const visible = box.checkVisibility();
            const beforeOuter = box.outerHTML.includes('id="box"');

            box.outerHTML = '<section id="box" data-next="1"><em id="neo">N</em></section>';
            const after = document.getElementById('box');
            const afterOuter = after.outerHTML.includes('data-next="1"');

            document.getElementById('result').textContent =
              hasBefore + ':' +
              names.length + ':' +
              toggledOn + ':' +
              toggledOff + ':' +
              picked + ':' +
              total + ':' +
              byClass + ':' +
              byTag + ':' +
              visible + ':' +
              beforeOuter + ':' +
              document.getElementById('neo').textContent + ':' +
              afterOuter + ':' +
              after.getAttribute('data-next');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:2:true:false:i1:2:2:2:true:true:N:true:1")?;
    Ok(())
}

#[test]
fn focus_and_blur_update_active_element_and_events() -> Result<()> {
    let html = r#"
        <input id='a'>
        <input id='b'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const a = document.getElementById('a');
          const b = document.getElementById('b');
          let order = '';

          a.addEventListener('focus', () => {
            order += 'aF';
          });
          a.addEventListener('blur', () => {
            order += 'aB';
          });
          b.addEventListener('focus', () => {
            order += 'bF';
          });
          b.addEventListener('blur', () => {
            order += 'bB';
          });

          document.getElementById('btn').addEventListener('click', () => {
            a.focus();
            b.focus();
            b.blur();
            document.getElementById('result').textContent =
              order + ':' + (document.activeElement === null ? 'none' : 'active');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "aFaBbFbB:none")?;
    Ok(())
}

#[test]
fn focus_in_and_focus_out_events_are_dispatched() -> Result<()> {
    let html = r#"
        <input id='a'>
        <input id='b'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const a = document.getElementById('a');
          const b = document.getElementById('b');
          let order = '';

          a.addEventListener('focusin', () => {
            order += 'aI';
          });
          a.addEventListener('focus', () => {
            order += 'aF';
          });
          a.addEventListener('focusout', () => {
            order += 'aO';
          });
          a.addEventListener('blur', () => {
            order += 'aB';
          });

          b.addEventListener('focusin', () => {
            order += 'bI';
          });
          b.addEventListener('focus', () => {
            order += 'bF';
          });
          b.addEventListener('focusout', () => {
            order += 'bO';
          });
          b.addEventListener('blur', () => {
            order += 'bB';
          });

          document.getElementById('btn').addEventListener('click', () => {
            a.focus();
            b.focus();
            b.blur();
            document.getElementById('result').textContent =
              order + ':' + (document.activeElement === null ? 'none' : 'active');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "aIaFaOaBbIbFbObB:none")?;
    Ok(())
}

#[test]
fn html_dom_input_event_toggles_submit_button_disabled_state() -> Result<()> {
    let html = r#"
        <form action='' method='get'>
          <input type='text' id='userName'>
          <input type='submit' value='Send' id='sendButton'>
        </form>
        <p id='result'></p>
        <script>
          const nameField = document.getElementById('userName');
          const sendButton = document.getElementById('sendButton');

          sendButton.disabled = true;

          nameField.addEventListener('input', (event) => {
            const elem = event.target;
            const valid = elem.value.length !== 0;

            if (valid && sendButton.disabled) {
              sendButton.disabled = false;
            } else if (!valid && !sendButton.disabled) {
              sendButton.disabled = true;
            }

            document.getElementById('result').textContent =
              sendButton.disabled ? 'disabled' : 'enabled';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#userName", "Kaz")?;
    h.assert_text("#result", "enabled")?;
    h.type_text("#userName", "")?;
    h.assert_text("#result", "disabled")?;
    Ok(())
}

#[test]
fn html_input_check_validity_and_validity_state_work() -> Result<()> {
    let html = r#"
        <input id='name' type='text' required minlength='4' maxlength='8' pattern='[A-Za-z]+'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('name');
          document.getElementById('run').addEventListener('click', () => {
            const r1 = input.checkValidity() + ':' + input.validity.valueMissing;
            input.value = 'ab1';
            const r2 = input.checkValidity() + ':' + input.validity.patternMismatch;
            input.value = 'Abcd';
            const r3 = input.checkValidity() + ':' + input.validity.valid;
            document.getElementById('result').textContent = r1 + '|' + r2 + '|' + r3;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true|false:true|true:true")?;
    Ok(())
}

#[test]
fn html_input_custom_validity_and_report_validity_work() -> Result<()> {
    let html = r#"
        <input id='email' type='email' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('email');
          document.getElementById('run').addEventListener('click', () => {
            input.setCustomValidity('Need fix');
            const first = input.reportValidity() + ':' + input.validity.customError + ':' + input.validationMessage;
            input.setCustomValidity('');
            input.value = 'a@example.com';
            const second = input.reportValidity() + ':' + input.validity.customError + ':' + input.validationMessage;
            document.getElementById('result').textContent = first + '|' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:Need fix|true:false:")?;
    Ok(())
}

#[test]
fn html_input_selection_and_range_text_methods_work() -> Result<()> {
    let html = r#"
        <input id='text' type='text' value='abcdef'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('text');
          document.getElementById('run').addEventListener('click', () => {
            input.setSelectionRange(1, 4, 'forward');
            const a = input.selectionStart + ':' + input.selectionEnd + ':' + input.selectionDirection;

            input.setRangeText('ZZ');
            const b = input.value + ':' + input.selectionStart + ':' + input.selectionEnd;

            input.setSelectionRange(2, 2);
            input.setRangeText('Q', 1, 3, 'select');
            const c = input.value + ':' + input.selectionStart + ':' + input.selectionEnd;

            input.select();
            const d = input.selectionStart + ':' + input.selectionEnd;

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:4:forward|aZZef:3:3|aQef:1:2|0:4")?;
    Ok(())
}

#[test]
fn html_input_step_methods_and_numeric_validity_work() -> Result<()> {
    let html = r#"
        <input id='num' type='number' min='2' max='10' step='2' value='4'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('num');
          document.getElementById('run').addEventListener('click', () => {
            input.stepUp();
            const a = input.value;

            input.stepDown(2);
            const b = input.value;

            input.value = '5';
            const c = input.validity.stepMismatch + ':' + input.checkValidity();

            input.value = '11';
            const d = input.validity.rangeOverflow + ':' + input.reportValidity();

            input.value = '1';
            const e = input.validity.rangeUnderflow + ':' + input.reportValidity();

            input.value = '8';
            input.showPicker();
            const f = input.checkValidity() + ':' + input.validity.valid + ':' + document.getElementById('num').validity.valid;

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "6|2|true:false|true:false|true:false|true:true:true",
    )?;
    Ok(())
}

#[test]
fn html_input_button_value_property_defaults_and_updates() -> Result<()> {
    let html = r#"
        <input id='empty' type='button'>
        <input id='named' type='button' value='Click Me'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const empty = document.getElementById('empty');
          const named = document.getElementById('named');
          document.getElementById('run').addEventListener('click', () => {
            const first = '[' + empty.value + ']:' + named.value;
            empty.value = 'Start machine';
            const second = empty.value;
            document.getElementById('result').textContent = first + '|' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "[]:Click Me|Start machine")?;
    Ok(())
}

#[test]
fn html_input_button_click_handler_runs_custom_logic() -> Result<()> {
    let html = r#"
        <input id='btn' type='button' value='Start machine'>
        <p id='result'>stopped</p>
        <script>
          const btn = document.getElementById('btn');
          const result = document.getElementById('result');
          btn.addEventListener('click', () => {
            if (btn.value === 'Start machine') {
              btn.value = 'Stop machine';
              result.textContent = 'started';
            } else {
              btn.value = 'Start machine';
              result.textContent = 'stopped';
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.assert_text("#result", "stopped")?;
    h.assert_value("#btn", "Start machine")?;
    h.click("#btn")?;
    h.assert_text("#result", "started")?;
    h.assert_value("#btn", "Stop machine")?;
    Ok(())
}

#[test]
fn html_input_button_click_has_no_default_submit_behavior() -> Result<()> {
    let html = r#"
        <form id='f' action=''>
          <input id='btn' type='button' value='Action'>
          <button id='submit' type='submit'>Submit</button>
        </form>
        <p id='result'></p>
        <script>
          document.getElementById('f').addEventListener('submit', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent = 'submitted';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.assert_text("#result", "")?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.click("#submit")?;
    h.assert_text("#result", "submitted")?;
    Ok(())
}

#[test]
fn html_input_button_does_not_participate_in_constraint_validation() -> Result<()> {
    let html = r#"
        <input id='button' type='button' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const button = document.getElementById('button');
          document.getElementById('run').addEventListener('click', () => {
            button.setCustomValidity('custom');
            const a = button.checkValidity() + ':' + button.reportValidity();
            const b = button.validity.valid + ':' + button.validity.valueMissing + ':' + button.validity.customError;
            const c = '[' + button.validationMessage + ']';
            document.getElementById('result').textContent = a + '|' + b + '|' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true|true:false:false|[]")?;
    Ok(())
}

#[test]
fn html_input_button_inherits_disabled_state_from_disabled_fieldset() -> Result<()> {
    let html = r#"
        <fieldset id='group' disabled>
          <input id='btn' type='button' value='Action'>
        </fieldset>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const group = document.getElementById('group');
          const btn = document.getElementById('btn');
          document.getElementById('run').addEventListener('click', () => {
            btn.focus();
            const before = document.activeElement === btn;
            group.disabled = false;
            btn.focus();
            const after = document.activeElement === btn;
            document.getElementById('result').textContent =
              before + ':' + after + ':' + btn.disabled + ':' + group.disabled;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:false:false")?;
    Ok(())
}

#[test]
fn html_input_checkbox_form_data_uses_on_default_and_omits_unchecked() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='subscribe' type='checkbox' name='subscribe' checked>
          <input id='empty' type='checkbox' name='empty' value='' checked>
          <input id='off' type='checkbox' name='off'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const fd = new FormData(document.getElementById('f'));
            document.getElementById('result').textContent =
              '[' + fd.get('subscribe') + ']:' +
              fd.has('off') + ':' +
              '[' + fd.get('empty') + ']';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "[on]:false:[]")?;
    Ok(())
}

#[test]
fn html_input_checkbox_multiple_same_name_values_are_all_submitted() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='coding' type='checkbox' name='interest' value='coding' checked>
          <input id='music' type='checkbox' name='interest' value='music' checked>
          <input id='art' type='checkbox' name='interest' value='art'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const fd = new FormData(document.getElementById('f'));
            const values = fd.getAll('interest');
            document.getElementById('result').textContent =
              values.length + ':' + values[0] + ':' + values[1] + ':' + values.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:coding:music:coding|music")?;
    Ok(())
}

#[test]
fn html_input_checkbox_required_validity_uses_value_missing() -> Result<()> {
    let html = r#"
        <input id='agree' type='checkbox' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const agree = document.getElementById('agree');
          document.getElementById('run').addEventListener('click', () => {
            const first =
              agree.checkValidity() + ':' + agree.validity.valueMissing + ':' + agree.validity.valid;
            agree.checked = true;
            const second =
              agree.checkValidity() + ':' + agree.validity.valueMissing + ':' + agree.validity.valid;
            document.getElementById('result').textContent = first + '|' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:false|true:false:true")?;
    Ok(())
}

#[test]
fn html_input_checkbox_indeterminate_is_visual_only_and_click_clears_it() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='flag' type='checkbox' name='flag'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const flag = document.getElementById('flag');
          document.getElementById('run').addEventListener('click', () => {
            flag.indeterminate = true;
            const first = flag.indeterminate + ':' + flag.checked;
            const beforeSubmit = new FormData(form).has('flag');
            flag.click();
            const second = flag.indeterminate + ':' + flag.checked;
            const submitted = new FormData(form).get('flag');
            document.getElementById('result').textContent =
              first + '|' + beforeSubmit + '|' + second + '|[' + submitted + ']';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false|false|false:true|[on]")?;
    Ok(())
}

#[test]
fn html_input_checkbox_label_click_toggles_associated_checkbox() -> Result<()> {
    let html = r#"
        <input id='opt' type='checkbox'>
        <label id='opt-label' for='opt'>Option</label>
        "#;

    let mut h = Harness::from_html(html)?;
    h.assert_checked("#opt", false)?;
    h.click("#opt-label")?;
    h.assert_checked("#opt", true)?;
    h.click("#opt-label")?;
    h.assert_checked("#opt", false)?;
    Ok(())
}

#[test]
fn html_input_checkbox_switch_keeps_checkbox_behavior() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='theme' type='checkbox' name='theme' switch>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const theme = document.getElementById('theme');
          const form = document.getElementById('f');
          document.getElementById('run').addEventListener('click', () => {
            theme.click();
            const submitted = new FormData(form).get('theme');
            document.getElementById('result').textContent = theme.checked + ':[' + submitted + ']';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:[on]")?;
    Ok(())
}

#[test]
fn selector_indeterminate_matches_checkbox_and_ignores_switch() -> Result<()> {
    let html = r#"
        <input id='a' type='checkbox'>
        <input id='b' type='checkbox' switch>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const a = document.getElementById('a');
          const b = document.getElementById('b');
          document.getElementById('run').addEventListener('click', () => {
            a.indeterminate = true;
            b.indeterminate = true;
            const matched = document.querySelectorAll('input:indeterminate');
            document.getElementById('result').textContent =
              matched.length + ':' + (matched.length ? matched[0].id : 'none');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:a")?;
    Ok(())
}

#[test]
fn html_element_hidden_and_inner_text_properties_work() -> Result<()> {
    let html = r#"
        <div id='box'>Hello <span>DOM</span></div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const box = document.getElementById('box');
          document.getElementById('run').addEventListener('click', () => {
            const before = box.hidden + ':' + box.hasAttribute('hidden');

            box.hidden = true;
            const hiddenState = box.hidden + ':' + box.hasAttribute('hidden');

            box.hidden = false;
            box.innerText = 'Replaced';

            document.getElementById('result').textContent =
              before + '|' +
              hiddenState + '|' +
              box.innerText + '|' +
              box.textContent + '|' +
              box.hasAttribute('hidden');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:false|true:true|Replaced|Replaced|false")?;
    Ok(())
}

#[test]
fn document_hidden_remains_read_only_while_element_hidden_is_writable() -> Result<()> {
    let html = r#"
        <div id='box'>x</div>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let docErr = '';
            try {
              document.hidden = true;
            } catch (e) {
              docErr = '' + e;
            }
            const box = document.getElementById('box');
            box.hidden = true;
            document.getElementById('result').textContent =
              docErr + '|' + box.hidden + ':' + box.hasAttribute('hidden');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "hidden is read-only|true:true")?;
    Ok(())
}

#[test]
fn focus_skips_disabled_element() -> Result<()> {
    let html = r#"
        <input id='name' disabled>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('name').focus();
            document.getElementById('result').textContent = document.activeElement ? 'has' : 'none';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "none")?;
    Ok(())
}

#[test]
fn selector_focus_and_focus_within_runtime() -> Result<()> {
    let html = r#"
        <div id='scope'>
          <input id='child'>
        </div>
        <input id='outside'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const child = document.getElementById('child');
            const outside = document.getElementById('outside');
            child.focus();
            const before = document.querySelector('input:focus').id + ':' +
              (document.querySelectorAll('#scope:focus-within').length ? 'yes' : 'no');
            outside.focus();
            const after = document.querySelector('input:focus').id + ':' +
              (document.querySelectorAll('#scope:focus-within').length ? 'yes' : 'no');
            document.getElementById('result').textContent = before + ':' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "child:yes:outside:no")?;
    Ok(())
}

#[test]
fn selector_active_is_set_during_click_and_cleared_after() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const during = document.querySelectorAll('#btn:active').length ? 'yes' : 'no';
            setTimeout(() => {
              const after = document.querySelectorAll('#btn:active').length ? 'yes' : 'no';
              document.getElementById('result').textContent = during + ':' + after;
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.advance_time(0)?;
    h.assert_text("#result", "yes:no")?;
    Ok(())
}

#[test]
fn active_element_assignment_is_read_only() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.activeElement = null;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("activeElement should be read-only");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("read-only"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn style_empty_value_removes_attribute_when_last_property() -> Result<()> {
    let html = r#"
        <div id='box' style='color: blue;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.style.color = '';
            document.getElementById('result').textContent =
              box.getAttribute('style') === '' ? 'none' : 'some';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "none")?;
    Ok(())
}

#[test]
fn style_overwrite_updates_existing_declaration_without_duplicate() -> Result<()> {
    let html = r#"
        <div id='box' style='color: blue; border-color: black;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.style.color = 'red';
            box.style.backgroundColor = 'white';
            document.getElementById('result').textContent = box.getAttribute('style');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "color: red; border-color: black; background-color: white;",
    )?;
    Ok(())
}

#[test]
fn get_computed_style_property_value_works() -> Result<()> {
    let html = r#"
        <div id='box' style='color: blue; background-color: transparent;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.style.color = 'red';
            const color = getComputedStyle(box).getPropertyValue('color');
            const missing = getComputedStyle(box).getPropertyValue('padding-top');
            document.getElementById('result').textContent = color + ':' + missing;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "red:")?;
    Ok(())
}

#[test]
fn style_parser_supports_quoted_colon_and_semicolon() -> Result<()> {
    let html = r#"
        <div id='box' style='content: "a:b;c"; color: blue;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.style.content + ':' + box.style.color + ':' + box.getAttribute('style');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "\"a:b;c\":blue:content: \"a:b;c\"; color: blue;")?;
    Ok(())
}

#[test]
fn style_parser_supports_parentheses_values() -> Result<()> {
    let html = r#"
        <div id='box' style='background-image: url("a;b:c"); font-family: Arial, sans-serif;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.style.backgroundImage + ':' + box.style.fontFamily + ':' + box.getAttribute('style');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
            "#result",
            "url(\"a;b:c\"):Arial, sans-serif:background-image: url(\"a;b:c\"); font-family: Arial, sans-serif;",
        )?;
    Ok(())
}

#[test]
fn element_reference_expression_assignment_works() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            const box = document.getElementById('box');
            const second = document.querySelectorAll('.item')[1];
            box.textContent = second.textContent + ':' + event.target.id;
            box.dataset.state = 'ok';
            document.getElementById('result').textContent =
              box.dataset.state + ':' + box.textContent;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok:B:btn")?;
    Ok(())
}

#[test]
fn event_properties_and_stop_immediate_propagation_work() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn'>run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.getElementById('result').textContent =
              event.type + ':' + event.target.id + ':' + event.currentTarget.id;
            event.stopImmediatePropagation();
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'second';
          });
          document.getElementById('root').addEventListener('click', () => {
            document.getElementById('result').textContent = 'root';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "click:btn:btn")?;
    Ok(())
}

#[test]
fn event_trusted_and_target_subproperties_are_accessible() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn' name='target-name'>run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent =
              event.isTrusted + ':' +
              event.target.name + ':' +
              event.currentTarget.name;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:target-name:target-name")?;
    Ok(())
}

#[test]
fn event_bubbles_and_cancelable_properties_are_available() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn' name='target-name'>run</button>
        </div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.getElementById('result').textContent =
              event.bubbles + ':' + event.cancelable + ':' + event.isTrusted;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:true:true")?;
    Ok(())
}

#[test]
fn dispatch_event_origin_is_untrusted_and_supports_event_methods() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='box'></div>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('root').addEventListener('custom', (event) => {
            document.getElementById('result').textContent = 'root:' + event.target.id;
          });
          document.getElementById('box').addEventListener('custom', (event) => {
            event.preventDefault();
            event.stopPropagation();
            document.getElementById('result').textContent =
              event.isTrusted + ':' + event.defaultPrevented;
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').dispatchEvent('custom');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:false")?;
    Ok(())
}

#[test]
fn dispatch_event_custom_non_bubbling_does_not_reach_ancestor_bubble_listeners() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div id='box'></div>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('root').addEventListener('custom', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'root';
          });
          document.getElementById('box').addEventListener('custom', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'box';
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = '';
            document.getElementById('box').dispatchEvent('custom');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "box")?;
    Ok(())
}

#[test]
fn prevent_default_does_not_flip_default_prevented_when_event_is_not_cancelable() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('box').addEventListener('custom', (event) => {
            event.preventDefault();
            document.getElementById('result').textContent =
              String(event.defaultPrevented) + ':' + String(event.cancelable);
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').dispatchEvent('custom');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false:false")?;
    Ok(())
}

#[test]
fn add_event_listener_deduplicates_same_handler_and_capture() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          let count = 0;
          const button = document.getElementById('btn');
          const onClick = () => {
            count = count + 1;
            document.getElementById('result').textContent = String(count);
          };
          button.addEventListener('click', onClick);
          button.addEventListener('click', onClick);
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn add_event_listener_does_not_deduplicate_distinct_inline_callbacks() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          let count = 0;
          const button = document.getElementById('btn');
          button.addEventListener('click', () => {
            count = count + 1;
          });
          button.addEventListener('click', () => {
            count = count + 1;
          });
          button.addEventListener('click', () => {
            document.getElementById('result').textContent = String(count);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2")?;
    Ok(())
}

#[test]
fn add_event_listener_supports_options_object_capture_and_defaults() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn'>run</button>
        </div>
        <p id='result'></p>
        <script>
          const order = [];
          const root = document.getElementById('root');
          const btn = document.getElementById('btn');

          root.addEventListener('click', () => {
            order.push('capture');
          }, { "capture": true, once: false });

          root.addEventListener('click', () => {
            order.push('bubble-opt');
          }, { passive: true });

          btn.addEventListener('click', () => {
            order.push('target');
          });

          root.addEventListener('click', () => {
            order.push('bubble');
            document.getElementById('result').textContent = order.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "capture,target,bubble-opt,bubble")?;
    Ok(())
}

#[test]
fn remove_event_listener_supports_options_object_capture_true() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn'>run</button>
        </div>
        <p id='result'></p>
        <script>
          let count = 0;
          const root = document.getElementById('root');
          const btn = document.getElementById('btn');

          const onCapture = () => {
            count = count + 1;
          };
          root.addEventListener('click', onCapture, { 'capture': true });
          root.removeEventListener('click', onCapture, { 'capture': true });

          btn.addEventListener('click', () => {
            document.getElementById('result').textContent = String(count);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0")?;
    Ok(())
}

#[test]
fn listener_options_object_capture_rejects_non_boolean_literal() {
    let err = Harness::from_html(
        r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {}, { capture: 1 });
        </script>
        "#,
    )
    .expect_err("non-boolean options.capture should fail");

    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("options.capture must be true/false"), "{msg}");
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn event_default_prevented_property_reflects_prevent_default() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.getElementById('result').textContent =
              event.defaultPrevented + ',';
            event.preventDefault();
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + event.defaultPrevented;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "false,true")?;
    Ok(())
}

#[test]
fn event_phase_and_timestamp_are_available_in_handler() -> Result<()> {
    let html = r#"
        <div id='root'>
          <button id='btn'>run</button>
        </div>
        <p id='result'></p>
        <script>
          let phases = '';
          document.getElementById('root').addEventListener('click', (event) => {
            phases = phases + (phases === '' ? '' : ',') + event.eventPhase + ':' + event.timeStamp;
          }, true);
          document.getElementById('btn').addEventListener('click', (event) => {
            phases = phases + ',' + event.eventPhase + ':' + event.timeStamp;
          }, true);
          document.getElementById('btn').addEventListener('click', (event) => {
            phases = phases + ',' + event.eventPhase + ':' + event.timeStamp;
          });
          document.getElementById('root').addEventListener('click', (event) => {
            phases = phases + ',' + event.eventPhase + ':' + event.timeStamp;
            document.getElementById('result').textContent = phases;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:0,2:0,2:0,3:0")?;
    Ok(())
}

#[test]
fn remove_event_listener_works_for_matching_handler() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = 'A';
          });
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + 'B';
          });
          document.getElementById('btn').removeEventListener('click', () => {
            document.getElementById('result').textContent = 'A';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "B")?;
    Ok(())
}

#[test]
fn dispatch_event_statement_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('box').addEventListener('custom', (event) => {
            document.getElementById('result').textContent =
              event.type + ':' + event.target.id + ':' + event.currentTarget.id;
          });
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.dispatchEvent('custom');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "custom:box:box")?;
    Ok(())
}

#[test]
fn dynamic_add_event_listener_inside_handler_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.addEventListener('custom', () => {
              document.getElementById('result').textContent = 'ok';
            });
            box.dispatchEvent('custom');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn dynamic_remove_event_listener_inside_handler_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <div id='box'></div>
        <p id='result'></p>
        <script>
          document.getElementById('box').addEventListener('custom', () => {
            document.getElementById('result').textContent = 'A';
          });
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.removeEventListener('custom', () => {
              document.getElementById('result').textContent = 'A';
            });
            box.dispatchEvent('custom');
            if (document.getElementById('result').textContent === '')
              document.getElementById('result').textContent = 'none';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "none")?;
    Ok(())
}

#[test]
fn set_timeout_runs_on_flush_and_captures_env() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.textContent = 'A';
            setTimeout(() => {
              result.textContent = result.textContent + 'B';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A")?;
    h.flush()?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn timer_arguments_support_additional_parameters_and_comments() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            // comment: schedule timer with extra arg and inline delay comment
            setTimeout((message) => {
              document.getElementById('result').textContent = message;
            }, 5, 'ok');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(4)?;
    h.assert_text("#result", "")?;
    h.advance_time(1)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn timer_callback_supports_multiple_parameters() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout((first, second, third) => {
              document.getElementById('result').textContent =
                first + ':' + second + ':' + third;
            }, 5, 'A', 'B', 'C');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(5)?;
    h.assert_text("#result", "A:B:C")?;
    Ok(())
}

#[test]
fn timer_callback_assigns_undefined_for_missing_arguments() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout((first, second, third) => {
              document.getElementById('result').textContent =
                first + ':' + second + ':' + third;
            }, 5, 'only');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(5)?;
    h.assert_text("#result", "only:undefined:undefined")?;
    Ok(())
}

#[test]
fn timer_function_reference_supports_additional_parameters() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const onTimeout = (value) => {
            document.getElementById('result').textContent = value;
          };
          document.getElementById('btn').addEventListener('click', () => {
            setTimeout(onTimeout, 5, 'ref');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(5)?;
    h.assert_text("#result", "ref")?;
    Ok(())
}

#[test]
fn timer_interval_function_reference_supports_additional_parameters() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          let count = 0;
          const onTick = (value) => {
            count = count + 1;
            document.getElementById('result').textContent =
              document.getElementById('result').textContent + value;
            if (count === 2) {
              clearInterval(intervalId);
            }
          };
          let intervalId = 0;
          document.getElementById('btn').addEventListener('click', () => {
            intervalId = setInterval(onTick, 5, 'tick');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.advance_time(11)?;
    h.assert_text("#result", "ticktick")?;
    h.advance_time(10)?;
    h.assert_text("#result", "ticktick")?;
    Ok(())
}
