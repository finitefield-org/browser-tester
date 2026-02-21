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
fn html_input_email_basic_and_required_validation_work() -> Result<()> {
    let html = r#"
        <input id='email' type='email' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('email');
          document.getElementById('run').addEventListener('click', () => {
            input.value = '';
            const a = input.checkValidity() + ':' + input.validity.valueMissing + ':' + input.validity.typeMismatch;

            input.value = 'me';
            const b = input.checkValidity() + ':' + input.validity.typeMismatch;

            input.value = 'me@example.org';
            const c = input.checkValidity() + ':' + input.validity.valid;

            input.value = 'me @example.org';
            const d = input.checkValidity() + ':' + input.validity.typeMismatch;

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true:false|false:true|true:true|false:true",
    )?;
    Ok(())
}

#[test]
fn html_input_email_multiple_and_pattern_validation_work() -> Result<()> {
    let html = r#"
        <input id='emails' type='email' multiple required pattern='.+@example\.com'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('emails');
          document.getElementById('run').addEventListener('click', () => {
            input.value = '';
            const a = input.checkValidity() + ':' + input.validity.valueMissing + ':' + input.validity.valid;

            input.value = '   ';
            const b = input.checkValidity() + ':' + input.validity.typeMismatch + ':' + input.validity.valid;

            input.value = 'me@example.com, you@example.com';
            const c = input.checkValidity() + ':' + input.validity.valid;

            input.value = 'me@example.com, you@other.com';
            const d = input.checkValidity() + ':' + input.validity.patternMismatch + ':' + input.validity.typeMismatch;

            input.value = ',';
            const e = input.checkValidity() + ':' + input.validity.typeMismatch;

            input.value = 'me@example.com,,you@example.com';
            const f = input.checkValidity() + ':' + input.validity.typeMismatch;

            input.value = 'me@example.com you@example.com';
            const g = input.checkValidity() + ':' + input.validity.typeMismatch;

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f + '|' + g;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:false:true|true:false:true|true:true|false:true:false|false:true|false:true|false:true",
    )?;
    Ok(())
}

#[test]
fn html_input_email_readonly_ignores_required_but_still_checks_type_mismatch() -> Result<()> {
    let html = r#"
        <input id='email' type='email' required readonly>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('email');
          document.getElementById('run').addEventListener('click', () => {
            const a = input.checkValidity() + ':' + input.validity.valueMissing + ':' + input.validity.valid;

            input.value = 'bad';
            const b = input.checkValidity() + ':' + input.validity.typeMismatch + ':' + input.validity.valueMissing;

            input.value = 'good@example.com';
            const c = input.checkValidity() + ':' + input.validity.valid;

            document.getElementById('result').textContent = a + '|' + b + '|' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false:true|false:true:false|true:true")?;
    Ok(())
}

#[test]
fn html_input_file_value_files_and_script_assignment_work() -> Result<()> {
    let html = r#"
        <input id='upload' type='file' accept='image/png, image/jpeg' required>
        <input id='text' type='text'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('upload');
          const text = document.getElementById('text');
          const events = [];
          input.addEventListener('input', () => {
            events.push('i:' + input.value + ':' + input.files.length);
          });
          input.addEventListener('change', () => {
            events.push('c:' + input.value + ':' + input.files.length);
          });
          document.getElementById('run').addEventListener('click', () => {
            const files = input.files;
            const first = files[0];
            const a = input.value + ':' + files.length + ':' + first.name + ':' + first.size + ':' + first.type + ':' + first.lastModified;

            input.value = 'foo.txt';
            const b = input.value + ':' + input.files.length;

            const c = (text.files === null) + ':' + input.checkValidity() + ':' + input.validity.valueMissing;
            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + events.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_input_files(
        "#upload",
        &[
            MockFile {
                name: "/tmp/photo.JPG".to_string(),
                size: 1234,
                mime_type: "image/jpeg".to_string(),
                last_modified: 1_700_000_000_000,
                webkit_relative_path: String::new(),
            },
            MockFile::new("ignored.png"),
        ],
    )?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "C:\\fakepath\\photo.JPG:1:photo.JPG:1234:image/jpeg:1700000000000|C:\\fakepath\\photo.JPG:1|true:true:false|i:C:\\fakepath\\photo.JPG:1,c:C:\\fakepath\\photo.JPG:1",
    )?;
    Ok(())
}

#[test]
fn html_input_file_multiple_required_and_cancel_event_work() -> Result<()> {
    let html = r#"
        <input id='docs' type='file' multiple required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('docs');
          const events = [];
          input.addEventListener('input', () => events.push('i:' + input.files.length));
          input.addEventListener('change', () => events.push('c:' + input.files.length));
          input.addEventListener('cancel', () => events.push('x:' + input.files.length));
          document.getElementById('run').addEventListener('click', () => {
            const files = input.files;
            const names = files.map((f) => f.name).join(',');
            const a = input.value + ':' + files.length + ':' + names + ':' + input.checkValidity() + ':' + input.validity.valueMissing;

            input.value = '';
            const b = '[' + input.value + ']:' + input.files.length + ':' + input.checkValidity() + ':' + input.validity.valueMissing;
            document.getElementById('result').textContent = a + '|' + b + '|' + events.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let files = vec![
        MockFile::new("a.txt"),
        MockFile {
            name: "nested/b.txt".to_string(),
            size: 7,
            mime_type: "text/plain".to_string(),
            last_modified: 99,
            webkit_relative_path: "nested/b.txt".to_string(),
        },
    ];
    h.set_input_files("#docs", &files)?;
    h.set_input_files("#docs", &files)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "C:\\fakepath\\a.txt:2:a.txt,b.txt:true:false|[]:0:false:true|i:2,c:2,x:2",
    )?;
    Ok(())
}

#[test]
fn html_input_file_ignores_value_attribute_updates() -> Result<()> {
    let html = r#"
        <input id='upload' type='file' value='secret.txt' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const input = document.getElementById('upload');
          document.getElementById('run').addEventListener('click', () => {
            const a = '[' + input.value + ']:' + input.files.length + ':' + input.checkValidity() + ':' + input.validity.valueMissing;
            input.setAttribute('value', 'another.txt');
            const b = '[' + input.value + ']:' + input.files.length;
            input.setAttribute('value', '');
            const c = '[' + input.value + ']:' + input.files.length;
            document.getElementById('result').textContent = a + '|' + b + '|' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "[]:0:false:true|[]:0|[]:0")?;
    Ok(())
}

#[test]
fn html_input_hidden_cannot_focus_and_user_typing_is_ignored() -> Result<()> {
    let html = r#"
        <input id='token' type='hidden' name='token' value='token-1' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const hidden = document.getElementById('token');
          const events = [];
          hidden.addEventListener('input', () => events.push('i'));
          hidden.addEventListener('change', () => events.push('c'));
          hidden.addEventListener('focus', () => events.push('f'));

          document.getElementById('run').addEventListener('click', () => {
            hidden.focus();
            const active = document.activeElement === null ? 'none' : 'has';
            const validity = hidden.checkValidity() + ':' + hidden.validity.valid + ':' + hidden.validity.valueMissing;
            document.getElementById('result').textContent =
              '[' + hidden.value + ']:' + active + ':' + events.join(',') + ':' + validity;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#token", "changed-by-user")?;
    h.click("#run")?;
    h.assert_text("#result", "[token-1]:none::true:true:false")?;
    Ok(())
}

#[test]
fn html_input_hidden_is_submitted_and_charset_name_is_overridden() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input type='text' name='title' value='My excellent blog post'>
          <input id='postId' type='hidden' name='postId' value='34657'>
          <input id='charset' type='hidden' name='_charset_' value='shift-jis'>
        </form>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('f');
            const first = new FormData(form);
            const a = first.get('title') + ':' + first.get('postId') + ':' + first.get('_charset_');

            document.getElementById('postId').value = '999';
            document.getElementById('charset').value = 'windows-31j';

            const second = new FormData(form);
            const b = second.get('postId') + ':' + second.get('_charset_');

            document.getElementById('result').textContent = a + '|' + b;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "My excellent blog post:34657:UTF-8|999:UTF-8")?;
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
fn html_input_color_defaults_and_normalizes_values() -> Result<()> {
    let html = r#"
        <input id='empty' type='color'>
        <input id='invalid' type='color' value='123'>
        <input id='hex' type='color' value='#A1B2C3'>
        <input id='short' type='color' value='#AbC'>
        <input id='func' type='color' value='oklab(50% 0.1 0.1 / 0.5)' colorspace='display-p3' alpha>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const empty = document.getElementById('empty');
          const invalid = document.getElementById('invalid');
          const hex = document.getElementById('hex');
          const short = document.getElementById('short');
          const func = document.getElementById('func');
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent =
              empty.value + '|' +
              invalid.value + '|' +
              hex.value + '|' +
              short.value + '|' +
              func.value + '|' +
              func.getAttribute('colorspace') + ':' + func.hasAttribute('alpha');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "#000000|#000000|#a1b2c3|#aabbcc|oklab(50% 0.1 0.1 / 0.5)|display-p3:true",
    )?;
    Ok(())
}

#[test]
fn html_input_color_input_and_change_events_track_updates() -> Result<()> {
    let html = r#"
        <p id='first'>First</p>
        <p id='second'>Second</p>
        <input id='picker' type='color' value='#0000ff'>
        <p id='result'></p>
        <script>
          const first = document.getElementById('first');
          const second = document.getElementById('second');
          const picker = document.getElementById('picker');
          const result = document.getElementById('result');

          picker.addEventListener('input', (event) => {
            first.style.color = event.target.value;
            result.textContent = 'input:' + first.style.color + ':' + second.style.color;
          });

          picker.addEventListener('change', (event) => {
            document.querySelectorAll('p').forEach((p) => {
              p.style.color = event.target.value;
            });
            result.textContent = 'change:' + first.style.color + ':' + second.style.color;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#picker", "#00FF00")?;
    h.assert_text("#result", "input:#00ff00:")?;

    h.dispatch("#picker", "change")?;
    h.assert_text("#result", "change:#00ff00:#00ff00")?;
    Ok(())
}

#[test]
fn html_input_color_value_assignment_and_select_behavior_work() -> Result<()> {
    let html = r#"
        <input id='picker' type='color' value='#123456'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const picker = document.getElementById('picker');
          document.getElementById('run').addEventListener('click', () => {
            picker.value = 'rgb(255 0 0 / 0.5)';
            const a = picker.value;

            picker.value = '!!invalid!!';
            const b = picker.value;

            picker.setAttribute('value', '#ABCDEF');
            const c = picker.value;

            picker.removeAttribute('value');
            const d = picker.value;

            const before = picker.selectionStart + ':' + picker.selectionEnd;
            picker.select();
            const after = picker.selectionStart + ':' + picker.selectionEnd;

            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + before + '|' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "rgb(255 0 0 / 0.5)|#000000|#abcdef|#000000|7:7|7:7",
    )?;
    Ok(())
}

#[test]
fn html_input_date_value_normalization_and_value_as_number_work() -> Result<()> {
    let html = r#"
        <input id='date' type='date' value='2018-07-22'>
        <input id='invalid' type='date' value='2018-13-40'>
        <input id='empty' type='date'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const date = document.getElementById('date');
          const invalid = document.getElementById('invalid');
          const empty = document.getElementById('empty');
          document.getElementById('run').addEventListener('click', () => {
            const initial = date.value + ':' + invalid.value + ':' + empty.value;

            date.value = '2017-06-01';
            const a = date.value + ':' + date.valueAsNumber;

            date.value = 'invalid-date';
            const b = '[' + date.value + ']:' + String(isNaN(date.valueAsNumber));

            date.valueAsNumber = 1496275200000;
            const c = date.value + ':' + date.valueAsNumber;

            date.valueAsNumber = NaN;
            const d = '[' + date.value + ']:' + String(isNaN(date.valueAsNumber));

            document.getElementById('result').textContent =
              initial + '|' + a + '|' + b + '|' + c + '|' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "2018-07-22::|2017-06-01:1496275200000|[]:true|2017-06-01:1496275200000|[]:true",
    )?;
    Ok(())
}

#[test]
fn html_input_date_min_max_step_and_required_validity_work() -> Result<()> {
    let html = r#"
        <input id='party' type='date' min='2017-04-01' max='2017-04-20' step='2' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const party = document.getElementById('party');
          document.getElementById('run').addEventListener('click', () => {
            party.value = '';
            const a = party.checkValidity() + ':' + party.validity.valueMissing;

            party.value = '2017-03-31';
            const b = party.validity.rangeUnderflow + ':' + party.checkValidity();

            party.value = '2017-04-21';
            const c = party.validity.rangeOverflow + ':' + party.reportValidity();

            party.value = '2017-04-02';
            const d = party.validity.stepMismatch + ':' + party.checkValidity();

            party.value = '2017-04-03';
            const e = party.validity.valid + ':' + party.reportValidity();

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true|true:false|true:false|true:false|true:true",
    )?;
    Ok(())
}

#[test]
fn html_input_date_step_methods_and_value_as_date_work() -> Result<()> {
    let html = r#"
        <input id='when' type='date' min='2017-04-01' max='2017-04-10' step='2' value='2017-04-03'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const when = document.getElementById('when');
          document.getElementById('run').addEventListener('click', () => {
            when.stepUp();
            const a = when.value;

            when.stepDown(2);
            const b = when.value;

            when.value = '';
            when.stepUp();
            const c = when.value;

            const dateObj = when.valueAsDate;
            const d = dateObj === null ? 'null' : dateObj.toISOString();

            when.valueAsDate = new Date('2017-04-09T15:00:00Z');
            const e = when.value + ':' + when.valueAsNumber;

            when.valueAsDate = null;
            const f = '[' + when.value + ']:' + (when.valueAsDate === null);

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "2017-04-05|2017-04-01|2017-04-03|2017-04-03T00:00:00.000Z|2017-04-09:1491696000000|[]:true",
    )?;
    Ok(())
}

#[test]
fn html_input_datetime_local_value_normalization_and_value_as_number_work() -> Result<()> {
    let html = r#"
        <input id='dt' type='datetime-local' value='2018-06-12T19:30'>
        <input id='invalid' type='datetime-local' value='2018-06-12 19:30'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const dt = document.getElementById('dt');
          const invalid = document.getElementById('invalid');
          document.getElementById('run').addEventListener('click', () => {
            const first = dt.value + ':[' + invalid.value + ']';

            dt.value = '2017-06-01T08:30';
            const a = dt.value + ':' + dt.valueAsNumber;

            dt.value = 'invalid';
            const b = '[' + dt.value + ']:' + String(isNaN(dt.valueAsNumber));

            dt.valueAsNumber = 1496305800000;
            const c = dt.value + ':' + dt.valueAsNumber;

            dt.valueAsNumber = NaN;
            const d = '[' + dt.value + ']:' + String(isNaN(dt.valueAsNumber));

            document.getElementById('result').textContent =
              first + '|' + a + '|' + b + '|' + c + '|' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "2018-06-12T19:30:[]|2017-06-01T08:30:1496305800000|[]:true|2017-06-01T08:30:1496305800000|[]:true",
    )?;
    Ok(())
}

#[test]
fn html_input_datetime_local_min_max_step_and_required_validity_work() -> Result<()> {
    let html = r#"
        <input id='party' type='datetime-local' min='2017-06-01T08:30' max='2017-06-30T16:30' step='120' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const party = document.getElementById('party');
          document.getElementById('run').addEventListener('click', () => {
            party.value = '';
            const a = party.checkValidity() + ':' + party.validity.valueMissing;

            party.value = '2017-06-01T08:29';
            const b = party.validity.rangeUnderflow + ':' + party.checkValidity();

            party.value = '2017-06-30T16:31';
            const c = party.validity.rangeOverflow + ':' + party.reportValidity();

            party.value = '2017-06-01T08:31';
            const d = party.validity.stepMismatch + ':' + party.checkValidity();

            party.value = '2017-06-01T08:32';
            const e = party.validity.valid + ':' + party.reportValidity();

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true|true:false|true:false|true:false|true:true",
    )?;
    Ok(())
}

#[test]
fn html_input_datetime_local_step_methods_and_value_as_date_work() -> Result<()> {
    let html = r#"
        <input id='when' type='datetime-local' min='2017-06-12T19:30' max='2017-06-12T20:00' step='120' value='2017-06-12T19:30'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const when = document.getElementById('when');
          document.getElementById('run').addEventListener('click', () => {
            when.stepUp();
            const a = when.value;

            when.stepDown(2);
            const b = when.value;

            when.value = '';
            when.stepUp();
            const c = when.value;

            const dateObj = when.valueAsDate;
            const d = dateObj === null ? 'null' : dateObj.toISOString();

            when.valueAsDate = new Date('2017-06-12T19:58:00Z');
            const e = when.value + ':' + when.valueAsNumber;

            when.valueAsDate = null;
            const f = '[' + when.value + ']:' + (when.valueAsDate === null);

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "2017-06-12T19:32|2017-06-12T19:30|2017-06-12T19:32|2017-06-12T19:32:00.000Z|2017-06-12T19:58:1497297480000|[]:true",
    )?;
    Ok(())
}

#[test]
fn html_input_time_value_normalization_and_value_as_number_work() -> Result<()> {
    let html = r#"
        <input id='t' type='time' value='13:30'>
        <input id='invalid' type='time' value='25:61'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const t = document.getElementById('t');
          const invalid = document.getElementById('invalid');
          document.getElementById('run').addEventListener('click', () => {
            const first = t.value + ':[' + invalid.value + ']';

            t.value = '15:30';
            const a = t.value + ':' + t.valueAsNumber;

            t.value = '09:00:05';
            const b = t.value + ':' + t.valueAsNumber;

            t.value = 'invalid';
            const c = '[' + t.value + ']:' + String(isNaN(t.valueAsNumber));

            t.valueAsNumber = 55800000;
            const d = t.value + ':' + t.valueAsNumber;

            t.valueAsNumber = 32405000;
            const e = t.value + ':' + t.valueAsNumber;

            t.valueAsNumber = NaN;
            const f = '[' + t.value + ']:' + String(isNaN(t.valueAsNumber));

            document.getElementById('result').textContent =
              first + '|' + a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "13:30:[]|15:30:55800000|09:00:05:32405000|[]:true|15:30:55800000|09:00:05:32405000|[]:true",
    )?;
    Ok(())
}

#[test]
fn html_input_time_min_max_step_and_required_validity_work() -> Result<()> {
    let html = r#"
        <input id='office' type='time' min='12:00' max='18:00' step='120' required>
        <input id='wrap' type='time' min='23:00' max='01:00'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const office = document.getElementById('office');
          const wrap = document.getElementById('wrap');
          document.getElementById('run').addEventListener('click', () => {
            office.value = '';
            const a = office.checkValidity() + ':' + office.validity.valueMissing;

            office.value = '11:59';
            const b = office.validity.rangeUnderflow + ':' + office.checkValidity();

            office.value = '18:01';
            const c = office.validity.rangeOverflow + ':' + office.reportValidity();

            office.value = '12:01';
            const d = office.validity.stepMismatch + ':' + office.checkValidity();

            office.value = '12:02';
            const e = office.validity.valid + ':' + office.reportValidity();

            wrap.value = '23:59';
            const f = wrap.validity.valid + ':' + wrap.checkValidity();

            wrap.value = '00:30';
            const g = wrap.validity.valid + ':' + wrap.checkValidity();

            wrap.value = '12:00';
            const h = wrap.validity.valid + ':' + wrap.checkValidity() + ':' + wrap.validity.rangeUnderflow + ':' + wrap.validity.rangeOverflow;

            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f + '|' + g + '|' + h;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true|true:false|true:false|true:false|true:true|true:true|true:true|false:false:true:true",
    )?;
    Ok(())
}

#[test]
fn html_input_time_step_methods_and_value_as_date_work() -> Result<()> {
    let html = r#"
        <input id='when' type='time' min='09:00' max='09:10' step='2' value='09:00'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const when = document.getElementById('when');
          document.getElementById('run').addEventListener('click', () => {
            when.stepUp();
            const a = when.value;

            when.stepDown(2);
            const b = when.value;

            when.value = '';
            when.stepUp();
            const c = when.value;

            const dateObj = when.valueAsDate;
            const d = dateObj === null ? 'null' : dateObj.toISOString();

            when.valueAsDate = new Date('1970-01-01T09:00:05Z');
            const e = when.value + ':' + when.valueAsNumber;

            when.valueAsDate = null;
            const f = '[' + when.value + ']:' + (when.valueAsDate === null);

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "09:00:02|09:00|09:00:02|1970-01-01T09:00:02.000Z|09:00:05:32405000|[]:true",
    )?;
    Ok(())
}

#[test]
fn html_input_time_readonly_ignores_required() -> Result<()> {
    let html = r#"
        <input id='time' type='time' required readonly>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const time = document.getElementById('time');
          document.getElementById('run').addEventListener('click', () => {
            const a = time.checkValidity() + ':' + time.validity.valueMissing + ':' + time.validity.valid;
            time.value = '12:34';
            const b = time.checkValidity() + ':' + time.validity.valid;
            document.getElementById('result').textContent = a + '|' + b;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false:true|true:true")?;
    Ok(())
}

#[test]
fn html_input_number_value_normalization_and_value_as_number_work() -> Result<()> {
    let html = r#"
        <input id='num' type='number' value='oops'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const num = document.getElementById('num');
          document.getElementById('run').addEventListener('click', () => {
            const a = '[' + num.value + ']:' + String(isNaN(num.valueAsNumber));

            num.value = '42.5';
            const b = num.value + ':' + num.valueAsNumber;

            num.value = 'bad';
            const c =
              '[' + num.value + ']:' +
              String(isNaN(num.valueAsNumber)) + ':' +
              num.validity.badInput + ':' +
              num.checkValidity();

            num.valueAsNumber = 10;
            const d = num.value + ':' + num.valueAsNumber;

            num.valueAsNumber = NaN;
            const e = '[' + num.value + ']:' + String(isNaN(num.valueAsNumber));

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "[]:true|42.5:42.5|[]:true:false:true|10:10|[]:true",
    )?;
    Ok(())
}

#[test]
fn html_input_number_required_readonly_step_and_pattern_behavior_work() -> Result<()> {
    let html = r#"
        <input id='num' type='number' required readonly pattern='\\d{3}' min='0' max='10' step='0.5'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const num = document.getElementById('num');
          document.getElementById('run').addEventListener('click', () => {
            const a =
              num.checkValidity() + ':' +
              num.validity.valueMissing + ':' +
              num.validity.patternMismatch;

            num.readOnly = false;
            const b = num.checkValidity() + ':' + num.validity.valueMissing;

            num.value = '1.1';
            const c = num.validity.stepMismatch + ':' + num.checkValidity();

            num.value = '7';
            const d = num.validity.patternMismatch + ':' + num.checkValidity();

            num.value = '11';
            const e = num.validity.rangeOverflow + ':' + num.checkValidity();

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:false:false|false:true|true:false|false:true|true:false",
    )?;
    Ok(())
}

#[test]
fn html_input_range_default_clamp_and_rounding_work() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='a' type='range' name='a'>
          <input id='b' type='range' name='b' min='0' max='11'>
          <input id='c' type='range' name='c' min='10' max='5'>
          <input id='d' type='range' name='d' min='0' max='100' value='90' step='10'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const a = document.getElementById('a');
          const b = document.getElementById('b');
          const c = document.getElementById('c');
          const d = document.getElementById('d');
          document.getElementById('run').addEventListener('click', () => {
            const first = a.value + ':' + b.value + ':' + c.value + ':' + d.value;

            a.value = '';
            const p1 = a.value;

            b.value = '20';
            const p2 = b.value;
            b.value = '-1';
            const p3 = b.value;
            b.value = 'bad';
            const p4 = b.value;

            c.value = '8';
            const p5 = c.value;

            d.value = '95';
            const p6 = d.value;
            d.value = '94';
            const p7 = d.value;

            const fd = new FormData(form);
            const p8 = fd.get('a') + ':' + fd.get('b') + ':' + fd.get('c') + ':' + fd.get('d');

            document.getElementById('result').textContent =
              first + '|' + p1 + '|' + p2 + '|' + p3 + '|' + p4 + '|' + p5 + '|' + p6 + '|' + p7 + '|' + p8;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "50:5.5:10:90|50|11|0|5.5|10|100|90|50:5.5:10:90")?;
    Ok(())
}

#[test]
fn html_input_range_value_as_number_and_step_methods_work() -> Result<()> {
    let html = r#"
        <input id='r' type='range' min='2' max='10' step='2' value='4'>
        <input id='plain' type='range'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const r = document.getElementById('r');
          const plain = document.getElementById('plain');
          document.getElementById('run').addEventListener('click', () => {
            r.stepUp();
            const a = r.value;

            r.stepDown(2);
            const b = r.value;

            r.value = '';
            const c = r.value;

            r.stepUp();
            const d = r.value;

            r.valueAsNumber = 9;
            const e = r.value + ':' + r.valueAsNumber;

            r.valueAsNumber = NaN;
            const f = r.value + ':' + r.valueAsNumber;

            plain.valueAsNumber = 150;
            const g = plain.value;

            plain.valueAsNumber = -5;
            const h = plain.value;

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f + '|' + g + '|' + h;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "6|2|6|8|10:10|6:6|100|0")?;
    Ok(())
}

#[test]
fn html_input_range_attr_mutation_resanitizes_value() -> Result<()> {
    let html = r#"
        <input id='r' type='range' min='0' max='100' value='50' step='10' required pattern='\\d{3}'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const r = document.getElementById('r');
          document.getElementById('run').addEventListener('click', () => {
            const first = r.checkValidity() + ':' + r.validity.valueMissing + ':' + r.validity.patternMismatch;

            r.setAttribute('min', '80');
            const a = r.value;

            r.setAttribute('max', '85');
            const b = r.value;

            r.setAttribute('step', '3');
            r.value = '84';
            const c = r.value + ':' + r.validity.stepMismatch + ':' + r.checkValidity();

            r.removeAttribute('value');
            const d = r.value;

            r.removeAttribute('min');
            const e = r.value;

            r.removeAttribute('max');
            const f = r.value;

            r.removeAttribute('step');
            r.value = '5.4';
            const g = r.value + ':' + r.validity.stepMismatch + ':' + r.checkValidity();

            document.getElementById('result').textContent =
              first + '|' + a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f + '|' + g;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:false:false|80|80|83:false:true|82.5|84|84|5:false:true",
    )?;
    Ok(())
}

#[test]
fn html_input_search_value_and_form_data_work() -> Result<()> {
    let html = r#"
        <form id='f'>
          <label for='q'>Search:</label>
          <input id='q' type='search' name='q'>
          <input id='other' type='search' value='preset'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const q = document.getElementById('q');
          const other = document.getElementById('other');
          document.getElementById('run').addEventListener('click', () => {
            const first = new FormData(form).get('q');
            q.value = 'cats dogs';
            other.value = 'ignored';
            const second = new FormData(form).get('q');
            document.getElementById('result').textContent =
              '[' + first + ']|[' + second + ']|' + q.value + ':' + other.value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "[]|[cats dogs]|cats dogs:ignored")?;
    Ok(())
}

#[test]
fn html_input_search_required_length_and_pattern_validation_work() -> Result<()> {
    let html = r#"
        <input id='len' type='search' required minlength='4' maxlength='8'>
        <input id='pat' type='search' required pattern='[A-z]{2}[0-9]{4}'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const len = document.getElementById('len');
          const pat = document.getElementById('pat');
          document.getElementById('run').addEventListener('click', () => {
            len.value = '';
            const a = len.checkValidity() + ':' + len.validity.valueMissing;

            len.value = 'abc';
            const b = len.validity.tooShort + ':' + len.checkValidity();

            len.value = 'abcdefghi';
            const c = len.validity.tooLong + ':' + len.checkValidity();

            len.value = 'abcd';
            const d = len.validity.valid + ':' + len.checkValidity();

            pat.value = 'AB1234';
            const e = pat.validity.valid + ':' + pat.checkValidity();

            pat.value = 'AB12X4';
            const f = pat.validity.patternMismatch + ':' + pat.checkValidity();

            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true|true:false|true:false|true:true|true:true|true:false",
    )?;
    Ok(())
}

#[test]
fn html_input_search_readonly_ignores_required_but_keeps_pattern_check() -> Result<()> {
    let html = r#"
        <input id='s' type='search' required readonly pattern='[a-z]+'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const s = document.getElementById('s');
          document.getElementById('run').addEventListener('click', () => {
            const a = s.checkValidity() + ':' + s.validity.valueMissing + ':' + s.validity.valid;

            s.value = 'BAD123';
            const b = s.checkValidity() + ':' + s.validity.patternMismatch + ':' + s.validity.valueMissing;

            s.value = 'okay';
            const c = s.checkValidity() + ':' + s.validity.valid;

            document.getElementById('result').textContent = a + '|' + b + '|' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false:true|false:true:false|true:true")?;
    Ok(())
}

#[test]
fn html_input_search_selection_methods_work_like_text_inputs() -> Result<()> {
    let html = r#"
        <input id='s' type='search' value='abcdef'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const s = document.getElementById('s');
          document.getElementById('run').addEventListener('click', () => {
            s.setSelectionRange(1, 4, 'forward');
            const a = s.selectionStart + ':' + s.selectionEnd + ':' + s.selectionDirection;

            s.setRangeText('ZZ');
            const b = s.value + ':' + s.selectionStart + ':' + s.selectionEnd;

            s.setSelectionRange(2, 2);
            s.setRangeText('Q', 1, 3, 'select');
            const c = s.value + ':' + s.selectionStart + ':' + s.selectionEnd;

            s.select();
            const d = s.selectionStart + ':' + s.selectionEnd;

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
fn html_input_tel_value_and_form_data_work() -> Result<()> {
    let html = r#"
        <form id='f'>
          <label for='tel'>Phone:</label>
          <input id='tel' type='tel' name='phone'>
          <input id='other' type='tel' value='preset'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const tel = document.getElementById('tel');
          const other = document.getElementById('other');
          document.getElementById('run').addEventListener('click', () => {
            const first = new FormData(form).get('phone');
            tel.value = '+1-212-555-3151';
            other.value = 'ignored';
            const second = new FormData(form).get('phone');
            document.getElementById('result').textContent =
              '[' + first + ']|[' + second + ']|' + tel.value + ':' + other.value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "[]|[+1-212-555-3151]|+1-212-555-3151:ignored")?;
    Ok(())
}

#[test]
fn html_input_tel_required_length_and_pattern_validation_work() -> Result<()> {
    let html = r#"
        <input id='len' type='tel' required minlength='4' maxlength='8'>
        <input id='pat' type='tel' required pattern='[0-9]{3}-[0-9]{3}-[0-9]{4}'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const len = document.getElementById('len');
          const pat = document.getElementById('pat');
          document.getElementById('run').addEventListener('click', () => {
            len.value = '';
            const a = len.checkValidity() + ':' + len.validity.valueMissing;

            len.value = 'abc';
            const b = len.validity.tooShort + ':' + len.checkValidity();

            len.value = 'abcdefghi';
            const c = len.validity.tooLong + ':' + len.checkValidity();

            len.value = 'abcd';
            const d = len.validity.valid + ':' + len.checkValidity();

            pat.value = '800-MDN-ROCKS';
            const e = pat.validity.patternMismatch + ':' + pat.checkValidity();

            pat.value = '865-555-6502';
            const f = pat.validity.valid + ':' + pat.checkValidity();

            const free = document.createElement('input');
            free.type = 'tel';
            free.required = true;
            free.value = 'not-a-number';
            const g = free.validity.typeMismatch + ':' + free.checkValidity();

            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f + '|' + g;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true|true:false|true:false|true:true|true:false|true:true|false:true",
    )?;
    Ok(())
}

#[test]
fn html_input_tel_readonly_ignores_required_but_keeps_pattern_check() -> Result<()> {
    let html = r#"
        <input id='tel' type='tel' required readonly pattern='[0-9]+'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const tel = document.getElementById('tel');
          document.getElementById('run').addEventListener('click', () => {
            const a = tel.checkValidity() + ':' + tel.validity.valueMissing + ':' + tel.validity.valid;

            tel.value = 'BAD123';
            const b = tel.checkValidity() + ':' + tel.validity.patternMismatch + ':' + tel.validity.valueMissing;

            tel.value = '123456';
            const c = tel.checkValidity() + ':' + tel.validity.valid;

            document.getElementById('result').textContent = a + '|' + b + '|' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false:true|false:true:false|true:true")?;
    Ok(())
}

#[test]
fn html_input_tel_selection_methods_work_like_text_inputs() -> Result<()> {
    let html = r#"
        <input id='tel' type='tel' value='1234567890'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const tel = document.getElementById('tel');
          document.getElementById('run').addEventListener('click', () => {
            tel.setSelectionRange(1, 4, 'forward');
            const a = tel.selectionStart + ':' + tel.selectionEnd + ':' + tel.selectionDirection;

            tel.setRangeText('ZZ');
            const b = tel.value + ':' + tel.selectionStart + ':' + tel.selectionEnd;

            tel.setSelectionRange(2, 2);
            tel.setRangeText('Q', 1, 3, 'select');
            const c = tel.value + ':' + tel.selectionStart + ':' + tel.selectionEnd;

            tel.select();
            const d = tel.selectionStart + ':' + tel.selectionEnd;

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "1:4:forward|1ZZ567890:3:3|1Q567890:1:2|0:8")?;
    Ok(())
}

#[test]
fn html_input_text_value_and_form_data_work() -> Result<()> {
    let html = r#"
        <form id='f'>
          <label for='name'>Name:</label>
          <input id='name' type='text' name='name'>
          <input id='other' type='text' value='preset'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const name = document.getElementById('name');
          const other = document.getElementById('other');
          document.getElementById('run').addEventListener('click', () => {
            const first = new FormData(form).get('name');
            name.value = 'Chris';
            other.value = 'ignored';
            const second = new FormData(form).get('name');
            document.getElementById('result').textContent =
              '[' + first + ']|[' + second + ']|' + name.value + ':' + other.value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "[]|[Chris]|Chris:ignored")?;
    Ok(())
}

#[test]
fn html_input_text_required_length_and_pattern_validation_work() -> Result<()> {
    let html = r#"
        <input id='len' type='text' required minlength='4' maxlength='8'>
        <input id='pat' type='text' required pattern='[a-z]{4,8}'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const len = document.getElementById('len');
          const pat = document.getElementById('pat');
          document.getElementById('run').addEventListener('click', () => {
            len.value = '';
            const a = len.checkValidity() + ':' + len.validity.valueMissing;

            len.value = 'abc';
            const b = len.validity.tooShort + ':' + len.checkValidity();

            len.value = 'abcdefghi';
            const c = len.validity.tooLong + ':' + len.checkValidity();

            len.value = 'abcd';
            const d = len.validity.valid + ':' + len.checkValidity();

            pat.value = 'AB12';
            const e = pat.validity.patternMismatch + ':' + pat.checkValidity();

            pat.value = 'abcd';
            const f = pat.validity.valid + ':' + pat.checkValidity();

            const free = document.createElement('input');
            free.type = 'text';
            free.required = true;
            free.value = 'anything';
            const g = free.validity.typeMismatch + ':' + free.checkValidity();

            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f + '|' + g;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true|true:false|true:false|true:true|true:false|true:true|false:true",
    )?;
    Ok(())
}

#[test]
fn html_input_text_readonly_ignores_required_but_keeps_pattern_check() -> Result<()> {
    let html = r#"
        <input id='name' type='text' required readonly pattern='[a-z]+'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const name = document.getElementById('name');
          document.getElementById('run').addEventListener('click', () => {
            const a = name.checkValidity() + ':' + name.validity.valueMissing + ':' + name.validity.valid;

            name.value = 'BAD123';
            const b = name.checkValidity() + ':' + name.validity.patternMismatch + ':' + name.validity.valueMissing;

            name.value = 'okay';
            const c = name.checkValidity() + ':' + name.validity.valid;

            document.getElementById('result').textContent = a + '|' + b + '|' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false:true|false:true:false|true:true")?;
    Ok(())
}

#[test]
fn html_input_url_value_and_form_data_work() -> Result<()> {
    let html = r#"
        <form id='f'>
          <label for='u'>URL:</label>
          <input id='u' type='url' name='url'>
          <input id='other' type='url' value='https://preset.example'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const u = document.getElementById('u');
          const other = document.getElementById('other');
          document.getElementById('run').addEventListener('click', () => {
            const first = new FormData(form).get('url');
            u.value = 'https://example.com';
            other.value = 'https://ignored.example';
            const second = new FormData(form).get('url');
            document.getElementById('result').textContent =
              '[' + first + ']|[' + second + ']|' + u.value + ':' + other.value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "[]|[https://example.com]|https://example.com:https://ignored.example",
    )?;
    Ok(())
}

#[test]
fn html_input_url_required_type_length_and_pattern_validation_work() -> Result<()> {
    let html = r#"
        <input id='url' type='url' required minlength='10' maxlength='30' pattern='https://.*'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const url = document.getElementById('url');
          document.getElementById('run').addEventListener('click', () => {
            url.value = '';
            const a = url.checkValidity() + ':' + url.validity.valueMissing + ':' + url.validity.typeMismatch;

            url.value = 'example.com/not-absolute';
            const b = url.validity.typeMismatch + ':' + url.checkValidity();

            url.value = 'https://x.co';
            const c = url.validity.valid + ':' + url.checkValidity();

            url.value = 'http://example.com';
            const d = url.validity.patternMismatch + ':' + url.checkValidity();

            url.value = 'https://a.co/this/path/is/very/long';
            const e = url.validity.tooLong + ':' + url.checkValidity();

            url.value = 'https://x';
            const f = url.validity.tooShort + ':' + url.checkValidity();

            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true:false|true:false|true:true|true:false|true:false|true:false",
    )?;
    Ok(())
}

#[test]
fn html_input_url_readonly_ignores_required_but_still_checks_type_mismatch() -> Result<()> {
    let html = r#"
        <input id='url' type='url' required readonly>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const url = document.getElementById('url');
          document.getElementById('run').addEventListener('click', () => {
            const a = url.checkValidity() + ':' + url.validity.valueMissing + ':' + url.validity.valid;

            url.value = 'bad';
            const b = url.checkValidity() + ':' + url.validity.typeMismatch + ':' + url.validity.valueMissing;

            url.value = 'https://example.com';
            const c = url.checkValidity() + ':' + url.validity.valid;

            document.getElementById('result').textContent = a + '|' + b + '|' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:false:true|false:true:false|true:true")?;
    Ok(())
}

#[test]
fn html_input_url_selection_methods_work_like_text_inputs() -> Result<()> {
    let html = r#"
        <input id='url' type='url' value='https://abc.def'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const url = document.getElementById('url');
          document.getElementById('run').addEventListener('click', () => {
            url.setSelectionRange(1, 4, 'forward');
            const a = url.selectionStart + ':' + url.selectionEnd + ':' + url.selectionDirection;

            url.setRangeText('ZZ');
            const b = url.value + ':' + url.selectionStart + ':' + url.selectionEnd;

            url.setSelectionRange(2, 2);
            url.setRangeText('Q', 1, 3, 'select');
            const c = url.value + ':' + url.selectionStart + ':' + url.selectionEnd;

            url.select();
            const d = url.selectionStart + ':' + url.selectionEnd;

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "1:4:forward|hZZs://abc.def:3:3|hQs://abc.def:1:2|0:13",
    )?;
    Ok(())
}

#[test]
fn html_input_password_strips_newlines_and_supports_selection_work() -> Result<()> {
    let html = r#"
        <input id='pwd' type='password' value='ab&#10;cd&#13;ef'>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const pwd = document.getElementById('pwd');
          const events = [];
          pwd.addEventListener('input', () => events.push('i:' + pwd.value));

          document.getElementById('run').addEventListener('click', () => {
            const a = '[' + pwd.value + ']';

            pwd.value = 'x\nY\r\nZ';
            const b = '[' + pwd.value + ']';

            pwd.setAttribute('value', '12\n34\r56');
            const c = '[' + pwd.value + ']';

            pwd.select();
            const d = pwd.selectionStart + ':' + pwd.selectionEnd;

            document.getElementById('result').textContent =
              a + '|' + b + '|' + c + '|' + d + '|' + events.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#pwd", "a\nb\rc")?;
    h.click("#run")?;
    h.assert_text("#result", "[abc]|[xYZ]|[123456]|0:6|i:abc")?;
    Ok(())
}

#[test]
fn html_input_password_validation_required_pattern_and_readonly_work() -> Result<()> {
    let html = r#"
        <input id='pass' type='password' minlength='4' maxlength='8' pattern='[0-9a-fA-F]{4,8}' required>
        <input id='readonly' type='password' required readonly>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const pass = document.getElementById('pass');
          const ro = document.getElementById('readonly');
          document.getElementById('run').addEventListener('click', () => {
            pass.value = '';
            const a = pass.checkValidity() + ':' + pass.validity.valueMissing;

            pass.value = '12';
            const b = pass.validity.tooShort + ':' + pass.checkValidity();

            pass.value = '123456789';
            const c = pass.validity.tooLong + ':' + pass.checkValidity();

            pass.value = 'zzzz';
            const d = pass.validity.patternMismatch + ':' + pass.checkValidity();

            pass.value = '1a2B';
            const e = pass.validity.valid + ':' + pass.reportValidity();

            const f = ro.checkValidity() + ':' + ro.validity.valueMissing + ':' + ro.validity.valid;

            document.getElementById('result').textContent = a + '|' + b + '|' + c + '|' + d + '|' + e + '|' + f;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true|true:false|true:false|true:false|true:true|true:false:true",
    )?;
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
fn html_input_submit_click_submits_form_only_when_valid() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' name='name' required>
          <input id='send' type='submit' value='Send'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          let submits = 0;
          document.getElementById('f').addEventListener('submit', (event) => {
            event.preventDefault();
            submits += 1;
          });
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = String(submits);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#send")?;
    h.click("#run")?;
    h.assert_text("#result", "0")?;
    h.type_text("#name", "Alice")?;
    h.click("#send")?;
    h.click("#run")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn html_input_submit_disabled_ignores_click() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' name='name' value='ok'>
          <input id='send' type='submit' value='Send' disabled>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          let clicks = 0;
          let submits = 0;
          document.getElementById('send').addEventListener('click', () => {
            clicks += 1;
          });
          document.getElementById('f').addEventListener('submit', (event) => {
            event.preventDefault();
            submits += 1;
          });
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = clicks + ':' + submits;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#send")?;
    h.click("#run")?;
    h.assert_text("#result", "0:0")?;
    Ok(())
}

#[test]
fn html_input_submit_does_not_participate_in_constraint_validation() -> Result<()> {
    let html = r#"
        <input id='send' type='submit' value='Send' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const send = document.getElementById('send');
          document.getElementById('run').addEventListener('click', () => {
            send.setCustomValidity('custom');
            const a = send.checkValidity() + ':' + send.reportValidity();
            const b = send.validity.valid + ':' + send.validity.valueMissing + ':' + send.validity.customError;
            const c = '[' + send.validationMessage + ']';
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
fn html_input_submit_formnovalidate_bypasses_validation() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' required>
          <input id='normal' type='submit' value='Send'>
          <input id='skip' type='submit' value='Skip' formnovalidate>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          let submits = 0;
          document.getElementById('f').addEventListener('submit', (event) => {
            event.preventDefault();
            submits += 1;
          });
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = String(submits);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#normal")?;
    h.click("#run")?;
    h.assert_text("#result", "0")?;
    h.click("#skip")?;
    h.click("#run")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn html_input_submit_novalidate_form_bypasses_validation() -> Result<()> {
    let html = r#"
        <form id='f' novalidate>
          <input id='name' required>
          <input id='send' type='submit'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          let submits = 0;
          const send = document.getElementById('send');
          document.getElementById('f').addEventListener('submit', (event) => {
            event.preventDefault();
            submits += 1;
          });
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent =
              '[' + send.value + ']:' + submits;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#send")?;
    h.click("#run")?;
    h.assert_text("#result", "[]:1")?;
    Ok(())
}

#[test]
fn html_input_reset_click_dispatches_reset_and_restores_defaults() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' value='Alice'>
          <input id='agree' type='checkbox' checked>
          <input id='plan-a' type='radio' name='plan' checked>
          <input id='plan-b' type='radio' name='plan'>
          <input id='resetter' type='reset' value='Reset'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const name = document.getElementById('name');
          const agree = document.getElementById('agree');
          const planA = document.getElementById('plan-a');
          const planB = document.getElementById('plan-b');
          const resetter = document.getElementById('resetter');
          const logs = [];

          form.addEventListener('reset', () => logs.push('reset'));

          document.getElementById('run').addEventListener('click', () => {
            name.value = 'Bob';
            agree.checked = false;
            planB.checked = true;
            resetter.click();
            document.getElementById('result').textContent =
              logs.join(',') + '|' +
              name.value + ':' +
              agree.checked + ':' +
              planA.checked + ':' +
              planB.checked;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "reset|Alice:true:true:false")?;
    Ok(())
}

#[test]
fn html_input_reset_click_prevent_default_skips_form_reset() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' value='Alice'>
          <input id='resetter' type='reset' value='Reset'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const name = document.getElementById('name');
          const resetter = document.getElementById('resetter');
          let clickCount = 0;
          let resetCount = 0;

          resetter.addEventListener('click', (event) => {
            clickCount += 1;
            event.preventDefault();
          });
          form.addEventListener('reset', () => {
            resetCount += 1;
          });

          document.getElementById('run').addEventListener('click', () => {
            name.value = 'Bob';
            resetter.click();
            document.getElementById('result').textContent =
              name.value + ':' + clickCount + ':' + resetCount;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "Bob:1:0")?;
    Ok(())
}

#[test]
fn html_input_reset_disabled_ignores_click_and_does_not_reset() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='name' value='Alice'>
          <input id='resetter' type='reset' value='Reset' disabled>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const name = document.getElementById('name');
          const resetter = document.getElementById('resetter');
          let clickCount = 0;
          let resetCount = 0;

          resetter.addEventListener('click', () => {
            clickCount += 1;
          });
          form.addEventListener('reset', () => {
            resetCount += 1;
          });

          document.getElementById('run').addEventListener('click', () => {
            name.value = 'Bob';
            resetter.click();
            document.getElementById('result').textContent =
              name.value + ':' + clickCount + ':' + resetCount;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "Bob:0:0")?;
    Ok(())
}

#[test]
fn html_input_reset_does_not_participate_in_constraint_validation() -> Result<()> {
    let html = r#"
        <input id='resetter' type='reset' value='Reset' required>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const resetter = document.getElementById('resetter');
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent =
              resetter.checkValidity() + ':' +
              resetter.validity.valid + ':' +
              resetter.validity.valueMissing;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:true:false")?;
    Ok(())
}

#[test]
fn html_input_image_submits_form_and_ignores_value_input() -> Result<()> {
    let html = r#"
        <form id='f' action=''>
          <input id='img' type='image' name='position' value='bad' required alt='Login' src='/login.png'>
        </form>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const img = document.getElementById('img');
          const events = [];
          img.addEventListener('click', () => events.push('k'));
          img.addEventListener('input', () => events.push('i'));
          img.addEventListener('change', () => events.push('c'));

          form.addEventListener('submit', (event) => {
            event.preventDefault();
            const a = '[' + img.value + ']:' + img.checkValidity() + ':' + img.validity.valueMissing;
            document.getElementById('result').textContent = a + ':' + events.join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#img", "typed-by-user")?;
    h.click("#img")?;
    h.assert_text("#result", "[]:true:false:k")?;
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
fn html_input_radio_form_data_and_default_on_value_work() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='email' type='radio' name='contact' value='email'>
          <input id='phone' type='radio' name='contact' value='phone'>
          <input id='sms' type='radio' name='contact'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const email = document.getElementById('email');
          const phone = document.getElementById('phone');
          const sms = document.getElementById('sms');

          document.getElementById('run').addEventListener('click', () => {
            const first = new FormData(form);
            const a = first.has('contact') + ':' + first.get('contact');

            sms.checked = true;
            const second = new FormData(form);
            const b = second.has('contact') + ':' + second.get('contact');

            phone.checked = true;
            const third = new FormData(form);
            const c = third.get('contact') + ':' + email.checked + ':' + phone.checked + ':' + sms.checked;

            document.getElementById('result').textContent = a + '|' + b + '|' + c;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:|true:on|phone:false:true:false")?;
    Ok(())
}

#[test]
fn html_input_radio_required_group_and_label_click_work() -> Result<()> {
    let html = r#"
        <form>
          <input id='a' type='radio' name='plan' required>
          <input id='b' type='radio' name='plan'>
          <label id='b-label' for='b'>Plan B</label>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const a = document.getElementById('a');
          const b = document.getElementById('b');
          const events = [];

          b.addEventListener('input', () => events.push('i:' + a.checked + ':' + b.checked));
          b.addEventListener('change', () => events.push('c:' + a.checked + ':' + b.checked));

          document.getElementById('run').addEventListener('click', () => {
            const first = a.checkValidity() + ':' + a.validity.valueMissing;
            document.getElementById('b-label').click();
            const second = a.checkValidity() + ':' + a.validity.valueMissing + ':' + a.checked + ':' + b.checked;
            document.getElementById('b-label').click();
            const third = a.checked + ':' + b.checked + ':' + events.join(',');
            document.getElementById('result').textContent = first + '|' + second + '|' + third;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "false:true|true:false:false:true|false:true:i:false:true,c:false:true",
    )?;
    Ok(())
}

#[test]
fn html_input_radio_set_attribute_checked_preserves_group_exclusive() -> Result<()> {
    let html = r#"
        <form>
          <input id='r1' type='radio' name='plan'>
          <input id='r2' type='radio' name='plan'>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const r1 = document.getElementById('r1');
          const r2 = document.getElementById('r2');
          document.getElementById('run').addEventListener('click', () => {
            r1.setAttribute('checked', '');
            r2.setAttribute('checked', '');
            document.getElementById('result').textContent =
              r1.checked + ':' + r2.checked + ':' + r1.hasAttribute('checked') + ':' + r2.hasAttribute('checked');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:true:true")?;
    Ok(())
}

#[test]
fn html_input_radio_type_change_with_checked_attribute_keeps_group_exclusive() -> Result<()> {
    let html = r#"
        <form id='f'>
          <input id='r1' type='radio' name='plan' value='a' checked>
          <input id='r2' type='checkbox' name='plan' value='b' checked>
        </form>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const form = document.getElementById('f');
          const r1 = document.getElementById('r1');
          const r2 = document.getElementById('r2');
          document.getElementById('run').addEventListener('click', () => {
            r2.setAttribute('type', 'radio');
            const fd = new FormData(form);
            document.getElementById('result').textContent =
              r1.checked + ':' + r2.checked + ':' + fd.get('plan');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "false:true:b")?;
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
fn type_text_input_handler_supports_current_target_closest_dataset_chain() -> Result<()> {
    let html = r#"
        <div id='root'>
          <div data-plan-id='0'>
            <input data-field='notation' />
          </div>
        </div>
        <p id='result'></p>
        <script>
          document.querySelector("[data-field='notation']").addEventListener('input', (event) => {
            const target = event.currentTarget;
            const planId = Number(target.closest("[data-plan-id]").dataset.planId);
            document.getElementById('result').textContent = String(planId);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("[data-field='notation']", "2/10 net 30")?;
    h.assert_text("#result", "0")?;
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
