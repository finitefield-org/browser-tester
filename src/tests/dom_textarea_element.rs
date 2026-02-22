use super::*;

#[test]
fn textarea_has_textbox_role_and_value_selection_validity_work() -> Result<()> {
    let html = r#"
        <label for='story'>Tell us your story:</label>
        <textarea
          id='story'
          name='story'
          rows='5'
          cols='33'
          required
          minlength='4'
          maxlength='20'>It was</textarea>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const story = document.getElementById('story');
            const initial =
              story.role + ':' +
              story.tagName + ':' +
              story.value + ':' +
              story.selectionStart + ':' +
              story.selectionEnd + ':' +
              story.required + ':' +
              story.readOnly + ':' +
              story.disabled;

            story.setSelectionRange(1, 4, 'forward');
            const selected =
              story.selectionStart + ':' +
              story.selectionEnd + ':' +
              story.selectionDirection;

            story.value = 'ok';
            const tooShort =
              story.checkValidity() + ':' +
              story.validity.tooShort + ':' +
              story.validity.valid;

            story.value = 'enough text';
            const valid =
              story.checkValidity() + ':' +
              story.validity.tooShort + ':' +
              story.validity.tooLong + ':' +
              story.validity.valueMissing + ':' +
              story.validity.valid;

            document.getElementById('result').textContent =
              initial + '|' + selected + '|' + tooShort + '|' + valid;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "textbox:TEXTAREA:It was:6:6:true:false:false|1:4:forward|false:true:false|true:false:false:false:true",
    )?;
    Ok(())
}

#[test]
fn textarea_attributes_and_role_override_roundtrip_work() -> Result<()> {
    let html = r#"
        <textarea
          id='msg'
          name='msg'
          rows='3'
          cols='15'
          autocomplete='on'
          autocapitalize='sentences'
          autocorrect='on'
          dirname='msg.dir'
          minlength='5'
          maxlength='12'
          placeholder='Comment text.'
          spellcheck='false'
          wrap='soft'></textarea>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const msg = document.getElementById('msg');
            const initial =
              msg.role + ':' +
              msg.getAttribute('rows') + ':' +
              msg.getAttribute('cols') + ':' +
              msg.getAttribute('autocomplete') + ':' +
              msg.getAttribute('autocapitalize') + ':' +
              msg.getAttribute('autocorrect') + ':' +
              msg.getAttribute('dirname') + ':' +
              msg.getAttribute('minlength') + ':' +
              msg.getAttribute('maxlength') + ':' +
              msg.getAttribute('placeholder') + ':' +
              msg.getAttribute('spellcheck') + ':' +
              msg.getAttribute('wrap');

            msg.setAttribute('rows', '5');
            msg.setAttribute('cols', '33');
            msg.setAttribute('autocomplete', 'off');
            msg.setAttribute('autocapitalize', 'none');
            msg.setAttribute('autocorrect', 'off');
            msg.setAttribute('dirname', 'story.dir');
            msg.setAttribute('minlength', '4');
            msg.setAttribute('maxlength', '8');
            msg.setAttribute('placeholder', 'Tell us your story');
            msg.setAttribute('spellcheck', 'true');
            msg.setAttribute('wrap', 'hard');

            msg.readOnly = true;
            msg.required = true;

            const updated =
              msg.getAttribute('rows') + ':' +
              msg.getAttribute('cols') + ':' +
              msg.getAttribute('autocomplete') + ':' +
              msg.getAttribute('autocapitalize') + ':' +
              msg.getAttribute('autocorrect') + ':' +
              msg.getAttribute('dirname') + ':' +
              msg.getAttribute('minlength') + ':' +
              msg.getAttribute('maxlength') + ':' +
              msg.getAttribute('placeholder') + ':' +
              msg.getAttribute('spellcheck') + ':' +
              msg.getAttribute('wrap');

            const flags =
              msg.readOnly + ':' +
              (msg.getAttribute('readonly') !== null) + ':' +
              msg.required + ':' +
              (msg.getAttribute('required') !== null);

            msg.role = 'none';
            const assignedRole = msg.role + ':' + msg.getAttribute('role');
            msg.removeAttribute('role');
            const restoredRole = msg.role + ':' + (msg.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + flags + '|' + assignedRole + '|' + restoredRole;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "textbox:3:15:on:sentences:on:msg.dir:5:12:Comment text.:false:soft|5:33:off:none:off:story.dir:4:8:Tell us your story:true:hard|true:true:true:true|none:none|textbox:true",
    )?;
    Ok(())
}
