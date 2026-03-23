use browser_tester_next::Harness;

#[test]
fn attribute_reflection_updates_selectors_and_form_state_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><button id='button'>First</button><input id='name'><input id='agree' type='checkbox'><select id='mode'><option value='a'>A</option><option id='selected' value='b'>B</option></select><div id='out'></div><script>const button = document.getElementById('button'); button.setAttribute('class', 'primary'); button.setAttribute('data-label', 'Hello'); button.toggleAttribute('data-flag'); button.removeAttribute('data-label'); const name = document.getElementById('name'); name.setAttribute('value', 'Alice'); const agree = document.getElementById('agree'); agree.setAttribute('checked', ''); const selected = document.getElementById('selected'); selected.setAttribute('selected', ''); document.getElementById('out').textContent = String(document.querySelectorAll('.primary').length) + ':' + String(document.querySelectorAll('[data-flag]').length) + ':' + String(button.getAttribute('data-label')) + ':' + name.value + ':' + String(agree.checked) + ':' + document.querySelector('option:checked').value;</script></main>",
    )?;

    harness.assert_text("#out", "1:1:null:Alice:true:b")?;
    harness.assert_exists(".primary")?;
    harness.assert_exists("[data-flag]")?;
    harness.assert_checked("#agree", true)?;
    harness.assert_value("#name", "Alice")?;
    harness.assert_exists("option:checked")?;
    Ok(())
}

#[test]
fn option_selected_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.selected; second.selected = true; const selected = document.getElementById('mode').selectedOptions; document.getElementById('out').textContent = String(before) + ':' + String(second.selected) + ':' + String(selected.length) + ':' + selected.item(0).getAttribute('id') + ':' + document.querySelector('option:checked').getAttribute('id');</script></main>",
    )?;

    harness.assert_text("#out", "false:true:1:second:second")?;
    harness.assert_exists("option:checked")?;
    harness.assert_exists("#second[selected]")?;
    Ok(())
}

#[test]
fn option_default_selected_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.defaultSelected; second.defaultSelected = true; const selected = document.getElementById('mode').selectedOptions; document.getElementById('out').textContent = String(before) + ':' + String(second.defaultSelected) + ':' + String(selected.length) + ':' + selected.item(0).getAttribute('id') + ':' + document.querySelector('option:checked').getAttribute('id');</script></main>",
    )?;

    harness.assert_text("#out", "false:true:1:second:second")?;
    harness.assert_exists("option:checked")?;
    harness.assert_exists("#second[selected]")?;
    Ok(())
}

#[test]
fn select_selected_index_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b'>B</option><option id='third' value='c'>C</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.selectedIndex; select.selectedIndex = 2; document.getElementById('out').textContent = String(before) + ':' + String(select.selectedIndex) + ':' + String(document.querySelector('option:checked').getAttribute('id'));</script></main>",
    )?;

    harness.assert_text("#out", "0:2:third")?;
    harness.assert_exists("option:checked")?;
    harness.assert_exists("#third[selected]")?;
    Ok(())
}

#[test]
fn option_index_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const second = document.getElementById('second'); const before = second.index; select.insertAdjacentHTML('afterbegin', '<option id=\"zero\" value=\"z\">Z</option>'); document.getElementById('out').textContent = String(before) + ':' + String(second.index) + ':' + String(document.getElementById('zero').index);</script></main>",
    )?;

    harness.assert_text("#out", "1:2:0")?;
    harness.assert_exists("#zero")?;
    Ok(())
}

#[test]
fn option_form_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><form id='owner'><select id='mode'><option id='first' value='a'>A</option><option id='second' value='b'>B</option></select></form><div id='out'></div><script>const second = document.getElementById('second'); const before = second.form; document.getElementById('mode').remove(); document.getElementById('out').textContent = before.getAttribute('id') + ':' + String(second.form);</script></main>",
    )?;

    harness.assert_text("#out", "owner:null")?;
    Ok(())
}

#[test]
fn option_disabled_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.disabled; second.disabled = true; const afterSet = second.disabled; second.disabled = false; const afterClear = second.disabled; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterClear) + ':' + String(document.querySelector('option:disabled'));</script></main>",
    )?;

    harness.assert_text("#out", "false:true:false:null")?;
    harness.assert_exists("#second")?;
    Ok(())
}

#[test]
fn form_control_disabled_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='name'><button id='action'>Go</button><div id='out'></div><script>const input = document.getElementById('name'); const button = document.getElementById('action'); const before = String(input.disabled) + '|' + String(button.disabled) + '|' + String(document.querySelectorAll(':disabled').length) + '|' + String(document.querySelectorAll(':enabled').length); input.disabled = true; button.disabled = true; const afterSet = String(input.disabled) + '|' + String(button.disabled) + '|' + String(document.querySelectorAll(':disabled').length) + '|' + String(document.querySelectorAll(':enabled').length); input.disabled = false; button.disabled = false; const afterClear = String(input.disabled) + '|' + String(button.disabled) + '|' + String(document.querySelectorAll(':disabled').length) + '|' + String(document.querySelectorAll(':enabled').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></main>",
    )?;

    harness.assert_text("#out", "false|false|0|2;true|true|2|0;false|false|0|2")?;
    harness.assert_exists("#name")?;
    harness.assert_exists("#action")?;
    Ok(())
}

#[test]
fn checkbox_indeterminate_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let mut harness = Harness::from_html(
        "<main id='root'><input id='agree' type='checkbox'><div id='out'></div><script>const agree = document.getElementById('agree'); agree.indeterminate = true; agree.addEventListener('change', () => { const current = document.getElementById('agree'); document.getElementById('out').textContent = String(current.indeterminate) + ':' + String(current.checked); });</script></main>",
    )?;

    harness.assert_exists("#agree:indeterminate")?;
    harness.click("#agree")?;
    harness.assert_text("#out", "false:true")?;
    harness.assert_checked("#agree", true)?;
    Ok(())
}

#[test]
fn checkbox_default_checked_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='agree' type='checkbox'><div id='out'></div><script>const agree = document.getElementById('agree'); const before = agree.defaultChecked; agree.defaultChecked = true; document.getElementById('out').textContent = String(before) + ':' + String(agree.defaultChecked) + ':' + String(agree.checked) + ':' + String(document.querySelectorAll(':default').length);</script></main>",
    )?;

    harness.assert_text("#out", "false:true:true:1")?;
    harness.assert_exists("#agree:default")?;
    harness.assert_checked("#agree", true)?;
    Ok(())
}

#[test]
fn option_label_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.label; second.label = 'Bee'; const afterSet = second.label; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(second.textContent) + ':' + String(document.querySelector('option[label=Bee]').getAttribute('id'));</script></main>",
    )?;

    harness.assert_text("#out", "B:Bee:B:second")?;
    harness.assert_exists("#second")?;
    Ok(())
}

#[test]
fn option_text_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const second = document.getElementById('second'); const before = second.text; second.text = 'Bee'; const afterSet = second.text; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(second.textContent) + ':' + String(second.label);</script></main>",
    )?;

    harness.assert_text("#out", "B:Bee:Bee:Bee")?;
    harness.assert_exists("#second")?;
    Ok(())
}

#[test]
fn select_multiple_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='second' value='b'>B</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.multiple; select.multiple = true; const afterSet = select.multiple; const afterSetAttr = select.getAttribute('multiple'); select.multiple = false; const afterClear = select.multiple; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterSetAttr) + ':' + String(afterClear);</script></main>",
    )?;

    harness.assert_text("#out", "false:true::false")?;
    harness.assert_exists("#mode")?;
    Ok(())
}

#[test]
fn select_type_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='first' value='a'>A</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.type; select.multiple = true; const after = select.type; document.getElementById('out').textContent = before + ':' + after;</script></main>",
    )?;

    harness.assert_text("#out", "select-one:select-multiple")?;
    harness.assert_exists("#mode")?;
    Ok(())
}

#[test]
fn input_and_button_type_are_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='field'><button id='action'></button><div id='out'></div><script>const input = document.getElementById('field'); const button = document.getElementById('action'); const before = input.type + '|' + button.type; input.type = 'email'; button.type = 'reset'; const afterSet = input.type + '|' + button.type + '|' + input.getAttribute('type') + '|' + button.getAttribute('type'); input.type = 'bogus'; button.type = 'bogus'; const afterInvalid = input.type + '|' + button.type + '|' + input.getAttribute('type') + '|' + button.getAttribute('type'); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterInvalid;</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "text|submit;email|reset|email|reset;text|submit|bogus|bogus",
    )?;
    harness.assert_exists("#field")?;
    harness.assert_exists("#action")?;
    Ok(())
}

#[test]
fn select_size_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.size; select.size = 3; const afterSet = select.size; const afterSetAttr = select.getAttribute('size'); select.size = 0; const afterZero = select.size; const afterZeroAttr = select.getAttribute('size'); document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterSetAttr) + ':' + String(afterZero) + ':' + String(afterZeroAttr);</script></main>",
    )?;

    harness.assert_text("#out", "0:3:3:0:0")?;
    harness.assert_exists("#mode")?;
    Ok(())
}

#[test]
fn select_required_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'></select><div id='out'></div><script>const select = document.getElementById('mode'); const before = select.required; select.required = true; const afterSet = select.required; const afterSetAttr = select.getAttribute('required'); select.required = false; const afterClear = select.required; document.getElementById('out').textContent = String(before) + ':' + String(afterSet) + ':' + String(afterSetAttr) + ':' + String(afterClear) + ':' + String(document.querySelector('select:required'));</script></main>",
    )?;

    harness.assert_text("#out", "false:true::false:null")?;
    harness.assert_exists("#mode")?;
    Ok(())
}

#[test]
fn form_no_validate_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><form id='signup'><button id='submit'>Submit</button><input id='field'></form><div id='out'></div><script>const form = document.getElementById('signup'); const button = document.getElementById('submit'); const field = document.getElementById('field'); const before = String(form.noValidate) + '|' + String(button.formNoValidate) + '|' + String(field.formNoValidate); form.noValidate = true; button.formNoValidate = true; field.formNoValidate = true; const afterSet = String(form.noValidate) + '|' + String(button.formNoValidate) + '|' + String(field.formNoValidate) + '|' + String(form.getAttribute('novalidate')) + '|' + String(button.getAttribute('formnovalidate')) + '|' + String(field.getAttribute('formnovalidate')); form.noValidate = false; button.formNoValidate = false; field.formNoValidate = false; const afterClear = String(form.noValidate) + '|' + String(button.formNoValidate) + '|' + String(field.formNoValidate) + '|' + String(form.hasAttribute('novalidate')) + '|' + String(button.hasAttribute('formnovalidate')) + '|' + String(field.hasAttribute('formnovalidate')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "false|false|false;true|true|true|||;false|false|false|false|false|false",
    )?;
    harness.assert_exists("#signup")?;
    Ok(())
}

#[test]
fn form_submission_metadata_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><form id='signup' action='/submit?from=form#frag' method='BOGUS' enctype='BOGUS'><button id='submit'>Submit</button><input id='field' type='submit' formaction='/field-preview?x=1#field'></form><div id='out'></div><script>const form = document.getElementById('signup'); const button = document.getElementById('submit'); const field = document.getElementById('field'); const before = String(form.action) + '|' + String(button.formAction) + '|' + String(field.formAction) + '|' + String(form.method) + '|' + String(form.enctype) + '|' + String(form.target) + '|' + String(button.formMethod) + '|' + String(button.formEnctype) + '|' + String(button.formTarget) + '|' + String(field.formMethod) + '|' + String(field.formEnctype) + '|' + String(field.formTarget); form.action = '/override-form?x=1#form'; form.method = 'Dialog'; form.enctype = 'Multipart/Form-Data'; form.target = '_blank'; button.formAction = '/override-button?button=1#button'; button.formMethod = 'PoSt'; button.formEnctype = 'Text/Plain'; button.formTarget = 'preview'; field.formAction = '/override-field?field=1#field'; field.formMethod = 'GET'; field.formEnctype = 'application/x-www-form-urlencoded'; field.formTarget = 'field-preview'; const afterSet = String(form.action) + '|' + String(button.formAction) + '|' + String(field.formAction) + '|' + String(form.method) + '|' + String(form.enctype) + '|' + String(form.target) + '|' + String(button.formMethod) + '|' + String(button.formEnctype) + '|' + String(button.formTarget) + '|' + String(field.formMethod) + '|' + String(field.formEnctype) + '|' + String(field.formTarget) + '|' + String(form.getAttribute('action')) + '|' + String(form.getAttribute('method')) + '|' + String(form.getAttribute('enctype')) + '|' + String(form.getAttribute('target')) + '|' + String(button.getAttribute('formaction')) + '|' + String(button.getAttribute('formmethod')) + '|' + String(button.getAttribute('formenctype')) + '|' + String(button.getAttribute('formtarget')) + '|' + String(field.getAttribute('formaction')) + '|' + String(field.getAttribute('formmethod')) + '|' + String(field.getAttribute('formenctype')) + '|' + String(field.getAttribute('formtarget')); document.getElementById('out').textContent = before + ';' + afterSet;</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "https://app.local/submit?from=form#frag|https://app.local/submit?from=form#frag|https://app.local/field-preview?x=1#field|get|application/x-www-form-urlencoded||get|application/x-www-form-urlencoded||get|application/x-www-form-urlencoded|;https://app.local/override-form?x=1#form|https://app.local/override-button?button=1#button|https://app.local/override-field?field=1#field|dialog|multipart/form-data|_blank|post|text/plain|preview|get|application/x-www-form-urlencoded|field-preview|/override-form?x=1#form|dialog|multipart/form-data|_blank|/override-button?button=1#button|post|text/plain|preview|/override-field?field=1#field|get|application/x-www-form-urlencoded|field-preview",
    )?;
    harness.assert_exists("#signup")?;
    Ok(())
}

#[test]
fn element_dir_and_lang_are_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = String(box.dir) + '|' + String(box.lang); box.dir = 'rtl'; box.lang = 'fr'; const afterSet = String(box.dir) + '|' + String(box.lang) + '|' + String(box.getAttribute('dir')) + '|' + String(box.getAttribute('lang')); document.getElementById('out').textContent = before + ';' + afterSet;</script></main>",
    )?;

    harness.assert_text("#out", "|;rtl|fr|rtl|fr")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_title_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' title='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.title; box.title = 'Updated'; document.getElementById('out').textContent = before + ':' + box.title + ':' + box.getAttribute('title');</script></main>",
    )?;

    harness.assert_text("#out", "Initial:Updated:Updated")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_role_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' role='button'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.role; box.role = 'menu'; document.getElementById('out').textContent = before + ':' + box.role + ':' + box.getAttribute('role');</script></main>",
    )?;

    harness.assert_text("#out", "button:menu:menu")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_tab_index_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box'></div><button id='button'></button><div id='out'></div><script>const box = document.getElementById('box'); const button = document.getElementById('button'); const before = String(box.tabIndex) + '|' + String(button.tabIndex); box.tabIndex = 4; button.tabIndex = -1; document.getElementById('out').textContent = before + ':' + String(box.tabIndex) + '|' + String(button.tabIndex) + ':' + box.getAttribute('tabindex') + '|' + button.getAttribute('tabindex');</script></main>",
    )?;

    harness.assert_text("#out", "-1|0:4|-1:4|-1")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_aria_label_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' aria-label='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaLabel; box.ariaLabel = 'Updated'; document.getElementById('out').textContent = before + ':' + box.ariaLabel + ':' + box.getAttribute('aria-label');</script></main>",
    )?;

    harness.assert_text("#out", "Initial:Updated:Updated")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_aria_description_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' aria-description='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaDescription; box.ariaDescription = 'Updated'; document.getElementById('out').textContent = before + ':' + box.ariaDescription + ':' + box.getAttribute('aria-description');</script></main>",
    )?;

    harness.assert_text("#out", "Initial:Updated:Updated")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_aria_role_description_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' aria-roledescription='Initial'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaRoleDescription; box.ariaRoleDescription = 'Updated'; document.getElementById('out').textContent = before + ':' + box.ariaRoleDescription + ':' + box.getAttribute('aria-roledescription');</script></main>",
    )?;

    harness.assert_text("#out", "Initial:Updated:Updated")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_aria_hidden_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' aria-hidden='true'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.ariaHidden; box.ariaHidden = false; document.getElementById('out').textContent = before + ':' + box.ariaHidden + ':' + box.getAttribute('aria-hidden');</script></main>",
    )?;

    harness.assert_text("#out", "true:false:false")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_access_key_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' accesskey='x'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.accessKey; box.accessKey = 'y'; document.getElementById('out').textContent = before + ':' + box.accessKey + ':' + box.getAttribute('accesskey');</script></main>",
    )?;

    harness.assert_text("#out", "x:y:y")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_slot_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' slot='start'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.slot; box.slot = 'end'; document.getElementById('out').textContent = before + ':' + box.slot + ':' + box.getAttribute('slot');</script></main>",
    )?;

    harness.assert_text("#out", "start:end:end")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_autocapitalize_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' autocapitalize='sentences'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.autocapitalize; box.autocapitalize = 'words'; document.getElementById('out').textContent = before + ':' + box.autocapitalize + ':' + box.getAttribute('autocapitalize');</script></main>",
    )?;

    harness.assert_text("#out", "sentences:words:words")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_spellcheck_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='parent' spellcheck='false'><span id='box'>Edit</span></div><div id='out'></div><script>const box = document.getElementById('box'); const before = String(box.spellcheck); box.spellcheck = true; document.getElementById('out').textContent = before + ':' + String(box.spellcheck) + ':' + box.getAttribute('spellcheck');</script></main>",
    )?;

    harness.assert_text("#out", "false:true:true")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_input_mode_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' inputmode='text'></div><div id='out'></div><script>const box = document.getElementById('box'); const before = box.inputMode; box.inputMode = 'numeric'; document.getElementById('out').textContent = before + ':' + box.inputMode + ':' + box.getAttribute('inputmode');</script></main>",
    )?;

    harness.assert_text("#out", "text:numeric:numeric")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_translate_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='parent' translate='no'><span id='box'>Edit</span></div><div id='out'></div><script>const box = document.getElementById('box'); const before = String(box.translate); box.translate = true; const afterTrue = String(box.translate) + ':' + box.getAttribute('translate'); box.translate = false; const afterFalse = String(box.translate) + ':' + box.getAttribute('translate'); document.getElementById('out').textContent = before + ':' + afterTrue + ':' + afterFalse;</script></main>",
    )?;

    harness.assert_text("#out", "false:true:yes:false:no")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn element_hidden_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='box' hidden></div><div id='out'></div><script>const box = document.getElementById('box'); const before = String(box.hidden); box.hidden = false; const afterClear = String(box.hidden) + ':' + String(box.hasAttribute('hidden')); box.hidden = true; document.getElementById('out').textContent = before + ':' + afterClear + ':' + String(box.hidden) + ':' + String(box.hasAttribute('hidden'));</script></main>",
    )?;

    harness.assert_text("#out", "true:false:false:true:true")?;
    harness.assert_exists("#box")?;
    Ok(())
}

#[test]
fn input_textarea_readonly_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='name' value='Ada'><textarea id='bio'>Hello</textarea><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.readOnly) + '|' + String(textarea.readOnly); input.readOnly = true; textarea.readOnly = true; const afterSet = String(input.readOnly) + '|' + String(textarea.readOnly); const afterAttr = String(input.hasAttribute('readonly')) + '|' + String(textarea.hasAttribute('readonly')); const afterReadOnly = String(input.matches(':read-only')) + '|' + String(textarea.matches(':read-only')); input.readOnly = false; textarea.readOnly = false; const afterClear = String(input.readOnly) + '|' + String(textarea.readOnly); const afterClearReadOnly = String(input.matches(':read-only')) + '|' + String(textarea.matches(':read-only')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterAttr + ';' + afterReadOnly + ';' + afterClear + ';' + afterClearReadOnly;</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "false|false;true|true;true|true;true|true;false|false;false|false",
    )?;
    harness.assert_exists("#name")?;
    harness.assert_exists("#bio")?;
    Ok(())
}

#[test]
fn input_textarea_placeholder_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='name' placeholder='Name'><textarea id='bio' placeholder='Bio'></textarea><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.placeholder) + '|' + String(textarea.placeholder) + '|' + String(document.querySelectorAll(':placeholder-shown').length); input.placeholder = 'Full name'; textarea.placeholder = 'Biography'; const afterSet = String(input.placeholder) + '|' + String(textarea.placeholder) + '|' + String(input.getAttribute('placeholder')) + '|' + String(textarea.getAttribute('placeholder')) + '|' + String(document.querySelectorAll(':placeholder-shown').length); input.value = 'Alice'; textarea.value = 'Bio text'; const afterValue = String(document.querySelectorAll(':placeholder-shown').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterValue;</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "Name|Bio|2;Full name|Biography|Full name|Biography|2;0",
    )?;
    harness.assert_exists("#name")?;
    harness.assert_exists("#bio")?;
    Ok(())
}

#[test]
fn input_textarea_autocomplete_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='name'><textarea id='bio'></textarea><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.autocomplete) + '|' + String(textarea.autocomplete); input.autocomplete = 'email'; textarea.autocomplete = 'off'; const afterSet = String(input.autocomplete) + '|' + String(textarea.autocomplete); const afterAttr = String(input.getAttribute('autocomplete')) + '|' + String(textarea.getAttribute('autocomplete')); input.autocomplete = ''; textarea.autocomplete = ''; const afterClear = String(input.autocomplete) + '|' + String(textarea.autocomplete) + '|' + String(input.hasAttribute('autocomplete')) + '|' + String(textarea.hasAttribute('autocomplete')); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterAttr + ';' + afterClear;</script></main>",
    )?;

    harness.assert_text("#out", "|;email|off;email|off;||true|true")?;
    harness.assert_exists("#name")?;
    harness.assert_exists("#bio")?;
    Ok(())
}

#[test]
fn input_file_accept_and_multiple_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='upload' type='file'><div id='out'></div><script>const upload = document.getElementById('upload'); const before = String(upload.accept) + '|' + String(upload.multiple) + '|' + String(upload.hasAttribute('accept')) + '|' + String(upload.hasAttribute('multiple')); upload.accept = 'image/*'; upload.multiple = true; const afterSet = String(upload.accept) + '|' + String(upload.multiple) + '|' + String(upload.getAttribute('accept')) + '|' + String(upload.hasAttribute('multiple')); document.getElementById('out').textContent = before + ';' + afterSet;</script></main>",
    )?;

    harness.assert_text("#out", "|false|false|false;image/*|true|image/*|true")?;
    harness.assert_exists("#upload[accept='image/*']")?;
    harness.assert_exists("#upload[multiple]")?;
    Ok(())
}

#[test]
fn input_textarea_length_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='name' minlength='2' maxlength='4' value='Ada'><textarea id='bio' minlength='2' maxlength='4'>Bio</textarea><div id='out'></div><script>const input = document.getElementById('name'); const textarea = document.getElementById('bio'); const before = String(input.minLength) + '|' + String(input.maxLength) + '|' + String(textarea.minLength) + '|' + String(textarea.maxLength) + '|' + String(document.querySelectorAll(':invalid').length); input.minLength = 4; textarea.maxLength = 2; const afterSet = String(input.minLength) + '|' + String(input.maxLength) + '|' + String(textarea.minLength) + '|' + String(textarea.maxLength) + '|' + String(document.querySelectorAll(':invalid').length); input.value = 'Jane'; textarea.value = 'No'; const afterValue = String(document.querySelectorAll(':invalid').length) + ':' + String(document.querySelectorAll(':valid').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterValue;</script></main>",
    )?;

    harness.assert_text("#out", "2|4|2|4|0;4|4|2|2|2;0:2")?;
    harness.assert_exists("#name")?;
    harness.assert_exists("#bio")?;
    Ok(())
}

#[test]
fn input_range_bounds_are_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='low' type='number' value='1'><input id='high' type='number' value='4'><div id='out'></div><script>const low = document.getElementById('low'); const high = document.getElementById('high'); const before = String(low.min) + '|' + String(low.max) + '|' + String(high.min) + '|' + String(high.max) + '|' + String(document.querySelectorAll(':in-range').length) + '|' + String(document.querySelectorAll(':out-of-range').length); low.min = 2; low.max = 6; high.min = 2; high.max = 6; const afterSet = String(low.min) + '|' + String(low.max) + '|' + String(high.min) + '|' + String(high.max) + '|' + String(document.querySelectorAll(':in-range').length) + '|' + String(document.querySelectorAll(':out-of-range').length) + '|' + document.querySelectorAll(':in-range').item(0).id + '|' + document.querySelectorAll(':out-of-range').item(0).id; low.removeAttribute('min'); low.removeAttribute('max'); high.removeAttribute('min'); high.removeAttribute('max'); const afterClear = String(low.min) + '|' + String(low.max) + '|' + String(high.min) + '|' + String(high.max) + '|' + String(document.querySelectorAll(':in-range').length) + '|' + String(document.querySelectorAll(':out-of-range').length); document.getElementById('out').textContent = before + ';' + afterSet + ';' + afterClear;</script></main>",
    )?;

    harness.assert_text("#out", "||||0|0;2|6|2|6|1|1|high|low;||||0|0")?;
    harness.assert_exists("#low")?;
    harness.assert_exists("#high")?;
    Ok(())
}

#[test]
fn input_pattern_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='name' pattern='A[a-z]+' value='Ada'><div id='out'></div><script>const input = document.getElementById('name'); const before = input.pattern; input.pattern = 'B[a-z]+'; const afterSet = input.pattern; document.getElementById('out').textContent = before + ':' + afterSet + ':' + String(document.querySelectorAll(':invalid').length);</script></main>",
    )?;

    harness.assert_text("#out", "A[a-z]+:B[a-z]+:1")?;
    harness.assert_exists("#name")?;
    harness.assert_exists("input:invalid")?;
    Ok(())
}

#[test]
fn attribute_reflection_rejects_empty_attribute_names_end_to_end() -> browser_tester_next::Result<()>
{
    let error = Harness::from_html(
        "<div id='root'></div><script>document.getElementById('root').setAttribute('', 'x');</script>",
    )
    .expect_err("empty attribute names should fail");

    assert!(
        error
            .to_string()
            .contains("attribute name must not be empty")
    );
    Ok(())
}

#[test]
fn class_views_update_selectors_and_dataset_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>const button = document.getElementById('button'); button.className = 'primary secondary'; const before = button.classList.length; const contains = button.classList.contains('primary'); button.classList.add('tertiary'); button.classList.remove('secondary'); const toggled = button.classList.toggle('active'); button.dataset.userId = '42'; const out = document.getElementById('out'); out.textContent = button.className + ':' + String(before) + ':' + String(contains) + ':' + String(toggled) + ':' + button.dataset.kind + ':' + button.dataset.userId + ':' + String(button.classList) + ':' + button.classList.toString() + ':' + String(button.classList.item(1)) + ':' + String(button.classList.keys().next().value) + ':' + String(button.classList.values().next().value) + ':' + String(button.classList.entries().next().value.index) + ':' + String(button.classList.entries().next().value.value) + ':' + String(button.dataset); button.classList.forEach((token, index, list) => { out.textContent += '|F' + String(index) + ':' + token + ':' + String(list.length); });</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "primary tertiary active:2:true:true:App:42:[object DOMTokenList]:primary tertiary active:tertiary:0:primary:0:primary:[object DOMStringMap]|F0:primary:3|F1:tertiary:3|F2:active:3",
    )?;
    harness.assert_exists(".active")?;
    harness.assert_exists("[data-user-id]")?;
    harness.assert_exists("[data-kind=App]")?;
    Ok(())
}

#[test]
fn class_list_value_is_publicly_supported() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><button id='button' class='primary secondary'>First</button><div id='out'></div><script>const button = document.getElementById('button'); const before = button.classList.value; button.classList.value = 'alpha beta'; document.getElementById('out').textContent = before + ':' + button.className + ':' + button.classList.value + ':' + String(button.classList.length) + ':' + String(button.classList.contains('alpha')) + ':' + String(button.classList.contains('beta'));</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "primary secondary:alpha beta:alpha beta:2:true:true",
    )?;
    harness.assert_exists(".alpha")?;
    Ok(())
}

#[test]
fn class_list_replace_is_publicly_supported() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><button id='button' class='base primary active'>First</button><div id='out'></div><script>const button = document.getElementById('button'); const first = button.classList.replace('primary', 'secondary'); const second = button.classList.replace('missing', 'delta'); const third = button.classList.replace('secondary', 'active'); document.getElementById('out').textContent = button.className + ':' + String(first) + ':' + String(second) + ':' + String(third) + ':' + String(button.classList.length) + ':' + button.classList.toString();</script></main>",
    )?;

    harness.assert_text("#out", "base active:true:false:true:2:base active")?;
    Ok(())
}

#[test]
fn class_list_rejects_whitespace_tokens_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<button id='button' class='base'></button><script>document.getElementById('button').classList.add('bad token');</script>",
    )
    .expect_err("classList tokens containing whitespace should fail");

    assert!(
        error
            .to_string()
            .contains("classList token must be a non-empty string without whitespace")
    );
    Ok(())
}

#[test]
fn collection_for_each_updates_live_script_views_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span class='item'>First</span><span class='item'>Second</span></main><div id='out'></div><script>const nodes = document.querySelectorAll('.item'); const children = document.getElementById('root').children; nodes.forEach((item, index, list) => { document.getElementById('out').textContent += 'N' + String(index) + ':' + item.textContent + ':' + String(list.length) + ';'; }, null); children.forEach((child, index, list) => { document.getElementById('out').textContent += 'H' + String(index) + ':' + child.textContent + ':' + String(list.length) + ';'; });</script>",
    )?;

    harness.assert_text("#out", "N0:First:2;N1:Second:2;H0:First:2;H1:Second:2;")?;
    Ok(())
}

#[test]
fn collection_iterator_helpers_update_live_script_views_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span class='item'>First</span><span class='item'>Second</span></main><div id='out'></div><script>const nodes = document.querySelectorAll('.item'); const nodeValues = nodes.values(); const nodeKeys = nodes.keys(); const children = document.getElementById('root').children; const childValues = children.values(); const childKeys = children.keys(); document.getElementById('root').textContent = 'gone'; const firstNode = nodeValues.next(); const secondNode = nodeValues.next(); const thirdNode = nodeValues.next(); const firstKey = nodeKeys.next(); const secondKey = nodeKeys.next(); const thirdKey = nodeKeys.next(); const firstChild = childValues.next(); const secondChild = childValues.next(); const thirdChild = childValues.next(); const childFirstKey = childKeys.next(); const childSecondKey = childKeys.next(); const childThirdKey = childKeys.next(); document.getElementById('out').textContent = firstNode.value.textContent + ':' + String(firstNode.done) + ':' + secondNode.value.textContent + ':' + String(secondNode.done) + ':' + String(thirdNode.done) + ':' + String(firstKey.value) + ':' + String(secondKey.value) + ':' + String(thirdKey.done) + ':' + firstChild.value.textContent + ':' + String(firstChild.done) + ':' + secondChild.value.textContent + ':' + String(secondChild.done) + ':' + String(thirdChild.done) + ':' + String(childFirstKey.value) + ':' + String(childSecondKey.value) + ':' + String(childThirdKey.done);</script>",
    )?;

    harness.assert_text(
        "#out",
        "First:false:Second:false:true:0:1:true:First:false:Second:false:true:0:1:true",
    )?;
    Ok(())
}

#[test]
fn collection_entries_update_live_script_views_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span class='item'>First</span><span class='item'>Second</span></main><div id='out'></div><script>const docEntries = document.childNodes.entries(); const childEntries = document.getElementById('root').children.entries(); const firstDoc = docEntries.next(); const secondDoc = docEntries.next(); const firstChild = childEntries.next(); const secondChild = childEntries.next(); document.getElementById('out').textContent = String(firstDoc.value.index) + ':' + firstDoc.value.value.nodeName + ':' + String(secondDoc.value.index) + ':' + secondDoc.value.value.nodeName + ':' + String(firstChild.value.index) + ':' + firstChild.value.value.textContent + ':' + String(secondChild.value.index) + ':' + secondChild.value.value.textContent;</script>",
    )?;

    harness.assert_text("#out", "0:main:1:div:0:First:1:Second")?;
    Ok(())
}

#[test]
fn node_clone_node_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); const clone = root.cloneNode(true); document.getElementById('out').textContent = String(clone) + ':' + String(clone.childNodes.length) + ':' + clone.childNodes.item(0).nodeName + ':' + String(clone.childNodes.item(0).nodeType) + ':' + clone.childNodes.item(0).textContent + ':' + String(root.childNodes.length);</script>",
    )?;

    harness.assert_text("#out", "[object Element]:1:#text:3:Hello:1")?;
    Ok(())
}

#[test]
fn node_same_node_and_equal_node_are_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='child'>Child</span></main><template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const root = document.getElementById('root'); const tpl = document.getElementById('tpl'); const clone = root.cloneNode(true); const fragment = tpl.content.cloneNode(true); document.getElementById('out').textContent = String(document.isSameNode(document)) + ':' + String(document.isSameNode(null)) + ':' + String(root.isSameNode(clone)) + ':' + String(root.isEqualNode(clone)) + ':' + String(tpl.content.isSameNode(fragment)) + ':' + String(tpl.content.isEqualNode(fragment));</script>",
    )?;

    harness.assert_text("#out", "true:false:false:true:false:true")?;
    Ok(())
}

#[test]
fn element_tag_name_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('root').tagName;</script>",
    )?;

    harness.assert_text("#out", "main")?;
    Ok(())
}

#[test]
fn element_local_name_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('root').localName;</script>",
    )?;

    harness.assert_text("#out", "main")?;
    Ok(())
}

#[test]
fn element_namespace_uri_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>document.getElementById('out').textContent = document.getElementById('root').namespaceURI;</script>",
    )?;

    harness.assert_text("#out", "http://www.w3.org/1999/xhtml")?;
    Ok(())
}

#[test]
fn create_element_ns_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg'); svg.setAttribute('viewBox', '0 0 10 10'); document.getElementById('root').appendChild(svg); document.getElementById('out').textContent = svg.namespaceURI + '|' + svg.localName + '|' + svg.outerHTML;</script>",
    )?;

    harness.assert_text(
        "#out",
        "http://www.w3.org/2000/svg|svg|<svg viewBox=\"0 0 10 10\"></svg>",
    )?;
    harness.assert_exists("svg")?;
    Ok(())
}

#[test]
fn element_name_is_available_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' name='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const before = document.getElementsByName('root').length; root.name = 'next'; const next = document.getElementsByName('next'); document.getElementById('out').textContent = String(before) + ':' + String(document.getElementsByName('root').length) + ':' + String(next.length) + ':' + next.item(0).getAttribute('id');</script>",
    )?;

    harness.assert_text("#out", "1:0:1:root")?;
    harness.assert_exists("[name=next]")?;
    Ok(())
}

#[test]
fn document_scripts_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><script id='first-script'></script></main><div id='out'></div><script>const out = document.getElementById('out'); const scripts = document.scripts; const before = scripts.length; const first = scripts.namedItem('first-script'); document.getElementById('root').textContent = 'gone'; out.textContent = String(before) + ':' + String(scripts.length) + ':' + String(first) + ':' + String(scripts.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:1:[object Element]:null")?;
    Ok(())
}

#[test]
fn document_style_sheets_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></main><div id='out'></div><script>const sheets = document.styleSheets; const before = sheets.length; const first = sheets.item(0); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(sheets.length) + ':' + String(first) + ':' + String(sheets.item(1));</script>",
    )?;

    harness.assert_text("#out", "2:0:[object CSSStyleSheet]:null")?;
    Ok(())
}

#[test]
fn document_style_sheets_iterator_helpers_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></main><div id='out'></div><script>const sheets = document.styleSheets; const keys = sheets.keys(); const values = sheets.values(); const entries = sheets.entries(); const key = keys.next(); const value = values.next(); const entry = entries.next(); document.getElementById('out').textContent = String(sheets.length) + ':' + String(key.value) + ':' + String(value.value) + ':' + String(entry.value.index) + ':' + String(entry.value.value);</script>",
    )?;

    harness.assert_text(
        "#out",
        "2:0:[object CSSStyleSheet]:0:[object CSSStyleSheet]",
    )?;
    Ok(())
}

#[test]
fn document_style_sheets_for_each_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'></main><div id='out'></div><script>const sheets = document.styleSheets; const out = document.getElementById('out'); sheets.forEach((sheet, index, list) => { out.textContent += String(index) + ':' + String(sheet) + ':' + String(list.length) + ';'; });</script>",
    )?;

    harness.assert_text(
        "#out",
        "0:[object CSSStyleSheet]:2;1:[object CSSStyleSheet]:2;",
    )?;
    Ok(())
}

#[test]
fn document_style_sheets_named_item_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><style id='first-style'>.primary { color: red; }</style><link id='first-link' rel='stylesheet' href='a.css'><link id='ignored-link' rel='preload' href='b.css'></main><div id='out'></div><script>const sheets = document.styleSheets; const before = sheets.length; const first = sheets.namedItem('first-style'); const second = sheets.namedItem('first-link'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(sheets.length) + ':' + String(first) + ':' + String(second) + ':' + String(sheets.namedItem('missing'));</script>",
    )?;

    harness.assert_text(
        "#out",
        "2:0:[object CSSStyleSheet]:[object CSSStyleSheet]:null",
    )?;
    Ok(())
}

#[test]
fn document_embeds_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><embed id='first-embed'><embed name='second-embed'></main><div id='out'></div><script>const embeds = document.embeds; const before = embeds.length; const first = embeds.namedItem('first-embed'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(embeds.length) + ':' + String(first) + ':' + String(embeds.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:0:[object Element]:null")?;
    Ok(())
}

#[test]
fn document_plugins_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><embed id='first-embed'><embed name='second-embed'></main><div id='out'></div><script>const plugins = document.plugins; const before = plugins.length; const first = plugins.namedItem('first-embed'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(plugins.length) + ':' + String(first) + ':' + String(plugins.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:0:[object Element]:null")?;
    Ok(())
}

#[test]
fn document_anchors_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><a name='first'>First</a><a id='ignored'>Ignored</a></main><div id='out'></div><script>const anchors = document.anchors; const before = anchors.length; const first = anchors.namedItem('first'); const root = document.getElementById('root'); root.innerHTML = root.innerHTML + '<a name=\"second\">Second</a>'; document.getElementById('out').textContent = String(before) + ':' + String(anchors.length) + ':' + first.textContent + ':' + anchors.namedItem('second').textContent + ':' + String(anchors.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "1:2:First:Second:null")?;
    harness.assert_exists("a[name=first]")?;
    harness.assert_exists("a[name=second]")?;
    Ok(())
}

#[test]
fn document_children_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span>First</span></main><div id='out'></div><script>const children = document.children; const before = children.length; const first = children.item(0); const root = children.namedItem('root'); document.getElementById('root').remove(); document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + String(first) + ':' + String(root) + ':' + String(children.namedItem('root'));</script>",
    )?;

    harness.assert_text("#out", "3:2:[object Element]:[object Element]:null")?;
    harness.assert_exists("#out")?;
    assert!(harness.assert_exists("#root").is_err());
    Ok(())
}

#[test]
fn child_nodes_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<!--pre--><main id='root'>Hello<span>World</span><!--tail--></main><div id='out'></div><script>const docNodes = document.childNodes; const rootNode = docNodes.item(1); const rootNodes = rootNode.childNodes; const docFirst = docNodes.item(0); const docSecond = docNodes.item(1); const rootValues = rootNodes.values(); const firstRoot = rootValues.next(); const secondRoot = rootValues.next(); const thirdRoot = rootValues.next(); document.getElementById('out').textContent = String(docNodes.length) + ':' + docFirst.nodeName + ':' + String(docFirst.nodeType) + ':' + String(docFirst) + ':' + docSecond.nodeName + ':' + String(docSecond.nodeType) + ':' + rootNode.nodeName + ':' + firstRoot.value.nodeName + ':' + String(firstRoot.value.nodeType) + ':' + firstRoot.value.textContent + ':' + secondRoot.value.nodeName + ':' + String(secondRoot.value.nodeType) + ':' + secondRoot.value.textContent + ':' + thirdRoot.value.nodeName + ':' + String(thirdRoot.value.nodeType) + ':' + thirdRoot.value.textContent;</script>",
    )?;

    harness.assert_text(
        "#out",
        "4:#comment:8:[object Node]:main:1:main:#text:3:Hello:span:1:World:#comment:8:",
    )?;
    Ok(())
}

#[test]
fn template_content_child_nodes_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const nodes = content.childNodes; const children = content.children; const before = nodes.length; tpl.innerHTML += '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent = String(content) + ':' + String(before) + ':' + String(nodes.length) + ':' + nodes.item(1).nodeName + ':' + String(children.length) + ':' + String(children.namedItem('second').textContent);</script>",
    )?;

    harness.assert_text("#out", "[object DocumentFragment]:1:3:#comment:2:Second")?;
    Ok(())
}

#[test]
fn template_content_inner_html_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const tpl = document.getElementById('tpl'); const content = tpl.content; const before = content.innerHTML; content.innerHTML = '<!--tail--><span id=\"second\">Second</span>'; document.getElementById('out').textContent = before + '|' + content.innerHTML + '|' + String(content.childNodes.length) + ':' + content.childNodes.item(0).nodeName + ':' + String(content.children.length) + ':' + content.children.namedItem('second').textContent;</script>",
    )?;

    harness.assert_text(
        "#out",
        "<span id=\"inner\">Inner</span>|<!--tail--><span id=\"second\">Second</span>|2:#comment:1:Second",
    )?;
    harness.assert_exists("#second")?;
    Ok(())
}

#[test]
fn template_content_text_content_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const before = content.textContent; content.textContent = 'Updated'; document.getElementById('out').textContent = before + ':' + content.textContent + ':' + content.innerHTML;</script>",
    )?;

    harness.assert_text("#out", "Inner:Updated:Updated")?;
    Ok(())
}

#[test]
fn template_content_fragment_reflection_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; document.getElementById('out').textContent = String(content.nodeType) + ':' + content.nodeName + ':' + String(content.parentNode) + ':' + String(content.ownerDocument);</script>",
    )?;

    harness.assert_text("#out", "11:#document-fragment:null:[object Document]")?;
    Ok(())
}

#[test]
fn template_content_clone_node_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const deep = content.cloneNode(true); const shallow = content.cloneNode(); document.getElementById('out').textContent = String(deep) + ':' + String(deep.childNodes.length) + ':' + deep.childNodes.item(0).nodeName + ':' + deep.childNodes.item(0).textContent + ':' + String(shallow.childNodes.length);</script>",
    )?;

    harness.assert_text("#out", "[object DocumentFragment]:1:span:Inner:0")?;
    Ok(())
}

#[test]
fn template_content_append_child_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const content = document.getElementById('tpl').content; const clone = content.cloneNode(true); const child = clone.childNodes.item(0); content.appendChild(child); document.getElementById('out').textContent = String(content.childNodes.length) + ':' + content.childNodes.item(1).textContent + ':' + String(clone.childNodes.length);</script>",
    )?;

    harness.assert_text("#out", "2:Inner:0")?;
    Ok(())
}

#[test]
fn document_create_document_fragment_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const frag = document.createDocumentFragment(); frag.appendChild(document.createTextNode('Hello')); const returned = root.appendChild(frag); document.getElementById('out').textContent = String(returned) + ':' + String(frag.childNodes.length) + ':' + root.textContent + ':' + String(root.childNodes.length);</script>",
    )?;

    harness.assert_text("#out", "[object DocumentFragment]:0:Hello:1")?;
    Ok(())
}

#[test]
fn document_import_node_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const source = document.createDocumentFragment(); source.appendChild(document.createTextNode('Hello')); const imported = document.importNode(source, true); const returned = root.appendChild(imported); document.getElementById('out').textContent = String(returned) + ':' + String(imported.childNodes.length) + ':' + root.textContent + ':' + String(root.childNodes.length);</script>",
    )?;

    harness.assert_text("#out", "[object DocumentFragment]:0:Hello:1")?;
    Ok(())
}

#[test]
fn node_normalize_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'>First</main><div id='out'></div><script>const root = document.getElementById('root'); root.appendChild(document.createTextNode('Second')); root.appendChild(document.createTextNode('Third')); root.normalize(); document.getElementById('out').textContent = String(root.childNodes.length) + ':' + String(root.firstChild.nodeType) + ':' + root.textContent;</script>",
    )?;

    harness.assert_text("#out", "1:3:FirstSecondThird")?;
    Ok(())
}

#[test]
fn node_remove_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); child.remove(); document.getElementById('out').textContent = String(child.parentNode) + ':' + String(root.childNodes.length);</script>",
    )?;

    harness.assert_text("#out", "null:0")?;
    Ok(())
}

#[test]
fn node_before_and_after_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.createTextNode('Hello'); root.appendChild(child); child.before(document.createTextNode('Before')); child.after(document.createTextNode('After')); document.getElementById('out').textContent = String(root.childNodes.length) + ':' + root.textContent;</script>",
    )?;

    harness.assert_text("#out", "3:BeforeHelloAfter")?;
    Ok(())
}

#[test]
fn template_content_rejects_non_template_elements_end_to_end() {
    let error = Harness::from_html(
        "<div id='box'></div><script>document.getElementById('box').content;</script>",
    )
    .expect_err("template.content should reject non-template elements");

    let message = error.to_string();
    assert!(message.contains("template.content"));
    assert!(message.contains("template"));
}

#[test]
fn template_content_rejects_outer_html_end_to_end() {
    let error = Harness::from_html(
        "<template id='tpl'></template><script>document.getElementById('tpl').content.outerHTML;</script>",
    )
    .expect_err("template.content.outerHTML should remain unsupported");

    assert!(error.to_string().contains("template content"));
    assert!(error.to_string().contains("outerHTML"));
}

#[test]
fn table_rows_and_cells_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<table id='table'><thead id='head'><tr id='head-row'><th id='head-cell'>H</th></tr></thead><tbody id='body'><tr id='first-row'><td id='first-cell'>A</td></tr></tbody><tfoot id='foot'><tr id='foot-row'><td id='foot-cell'>F</td></tr></tfoot></table><div id='out'></div><script>const table = document.getElementById('table'); const body = document.getElementById('body'); const row = document.getElementById('first-row'); const rows = table.rows; const bodyRows = body.rows; const cells = row.cells; const before = String(rows.length) + ':' + String(bodyRows.length) + ':' + String(cells.length) + ':' + String(rows.namedItem('first-row')) + ':' + String(cells.namedItem('first-cell')); body.innerHTML = body.innerHTML + '<tr id=\"second-row\"><td id=\"second-cell\">B</td><td id=\"third-cell\">C</td></tr>'; row.append(document.getElementById('third-cell')); document.getElementById('out').textContent = before + '|' + String(rows.length) + ':' + String(bodyRows.length) + ':' + String(cells.length) + ':' + String(rows.namedItem('second-row')) + ':' + String(bodyRows.namedItem('second-row')) + ':' + String(cells.namedItem('third-cell'));</script>",
    )?;

    harness.assert_text(
        "#out",
        "3:1:1:[object Element]:[object Element]|4:2:2:[object Element]:[object Element]:[object Element]",
    )?;
    harness.assert_exists("#second-row")?;
    Ok(())
}

#[test]
fn table_rows_reject_non_table_elements_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='bad'></div><script>document.getElementById('bad').rows.length;</script>",
    )
    .expect_err("non-table rows access should fail");

    assert!(error.to_string().contains("table.rows"));
    assert!(
        error
            .to_string()
            .contains("supported table.rows host element")
    );
    Ok(())
}

#[test]
fn document_embeds_are_not_available_on_elements_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').embeds.length;</script>",
    )
    .expect_err("non-document embeds access should fail");

    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`embeds`"));
    assert!(error.to_string().contains("element value"));
    Ok(())
}

#[test]
fn document_plugins_are_not_available_on_elements_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').plugins.length;</script>",
    )
    .expect_err("non-document plugins access should fail");

    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`plugins`"));
    assert!(error.to_string().contains("element value"));
    Ok(())
}

#[test]
fn document_style_sheets_are_not_available_on_elements_end_to_end()
-> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').styleSheets.length;</script>",
    )
    .expect_err("non-document styleSheets access should fail");

    assert!(error.to_string().contains("unsupported member access"));
    assert!(error.to_string().contains("`styleSheets`"));
    assert!(error.to_string().contains("element value"));
    Ok(())
}

#[test]
fn tree_mutation_primitives_support_append_prepend_and_remove_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='target'></section><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button><div id='out'></div><script>const target = document.getElementById('target'); const first = document.getElementById('first'); const second = document.getElementById('second'); const third = document.getElementById('third'); target.append(first, second); target.prepend(third); first.remove(); document.getElementById('out').textContent = String(target.children.length) + ':' + target.children.item(0).textContent + ':' + target.children.item(1).textContent + ':' + String(target.children.item(2));</script></main>",
    )?;

    harness.assert_text("#out", "2:Third:Second:null")?;
    harness.assert_exists("#target > #third")?;
    harness.assert_exists("#target > #second")?;
    Ok(())
}

#[test]
fn tree_mutation_primitives_support_remove_child_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='first'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const out = document.getElementById('out'); const root = document.getElementById('root'); const child = document.getElementById('first'); out.textContent = String(root.removeChild(child)) + ':' + String(root.children.length) + ':' + String(child.parentNode);</script>",
    )?;

    harness.assert_text("#out", "[object Element]:1:null")?;
    harness.assert_exists("#root > #second")?;
    Ok(())
}

#[test]
fn tree_mutation_primitives_support_insert_and_replace_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='target'><span id='placeholder'>Placeholder</span></section><button id='first'>First</button><button id='second'>Second</button><button id='third'>Third</button><div id='out'></div><script>const target = document.getElementById('target'); const first = document.getElementById('first'); const second = document.getElementById('second'); const third = document.getElementById('third'); target.replaceChildren(first, second); target.replaceChild(third, second); document.getElementById('out').textContent = String(target.children.length) + ':' + target.children.item(0).textContent + ':' + target.children.item(1).textContent + ':' + String(document.querySelector('#placeholder'));</script></main>",
    )?;

    harness.assert_text("#out", "2:First:Third:null")?;
    harness.assert_exists("#target > #first")?;
    harness.assert_exists("#target > #third")?;
    Ok(())
}

#[test]
fn tree_mutation_primitives_support_replace_with_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='first'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const first = document.getElementById('first'); first.replaceWith(document.createTextNode('Alpha'), document.createTextNode('Beta')); document.getElementById('out').textContent = String(document.getElementById('root').childNodes.length) + ':' + String(document.getElementById('root').childNodes.item(0).nodeType) + ':' + document.getElementById('root').childNodes.item(0).textContent + ':' + document.getElementById('root').childNodes.item(1).textContent + ':' + document.getElementById('root').childNodes.item(2).textContent;</script>",
    )?;

    harness.assert_text("#out", "3:3:Alpha:Beta:Second")?;
    harness.assert_exists("#root > #second")?;
    Ok(())
}

#[test]
fn tree_reflection_contains_support_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='child'>Child</span></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); const detached = document.createElement('div'); document.getElementById('out').textContent = String(document.contains(root)) + ':' + String(root.contains(child)) + ':' + String(child.contains(root)) + ':' + String(detached.contains(detached)) + ':' + String(detached.contains(root)) + ':' + String(root.contains(null));</script>",
    )?;

    harness.assert_text("#out", "true:true:false:true:false:false")?;
    Ok(())
}

#[test]
fn tree_reflection_compare_document_position_support_end_to_end() -> browser_tester_next::Result<()>
{
    let harness = Harness::from_html(
        "<main id='root'><span id='a'><em id='c'>Child</em></span><span id='b'>Sibling</span></main><div id='out'></div><script>const a = document.getElementById('a'); const b = document.getElementById('b'); const c = document.getElementById('c'); const detached = document.createElement('p'); document.getElementById('out').textContent = String(document.compareDocumentPosition(a)) + ':' + String(a.compareDocumentPosition(document)) + ':' + String(a.compareDocumentPosition(b)) + ':' + String(b.compareDocumentPosition(a)) + ':' + String(a.compareDocumentPosition(c)) + ':' + String(c.compareDocumentPosition(a)) + ':' + String(a.compareDocumentPosition(detached)) + ':' + String(detached.compareDocumentPosition(a)) + ':' + String(document.compareDocumentPosition(document));</script>",
    )?;

    harness.assert_text("#out", "20:10:4:2:20:10:37:35:0")?;
    Ok(())
}

#[test]
fn tree_reflection_has_child_nodes_support_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='child'>Child</span></main><template id='tpl'><span id='inner'>Inner</span></template><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); const tpl = document.getElementById('tpl'); document.getElementById('out').textContent = String(document.hasChildNodes()) + ':' + String(root.hasChildNodes()) + ':' + String(child.hasChildNodes()) + ':' + String(tpl.content.hasChildNodes());</script>",
    )?;

    harness.assert_text("#out", "true:true:true:true")?;
    Ok(())
}

#[test]
fn tree_reflection_first_and_last_child_support_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<!--pre--><html><head></head><body>Text<div id='out'></div><main id='root'><span id='child'>Child</span></main><template id='tpl'><span id='inner'>Inner</span><!--tail--></template><script>const html = document.documentElement; const body = document.body; const text = body.childNodes.item(0); const tpl = document.getElementById('tpl'); document.getElementById('out').textContent = String(document.firstChild) + ':' + String(document.lastChild) + ':' + String(html.firstChild) + ':' + String(html.lastChild) + ':' + String(body.firstChild) + ':' + String(body.lastChild) + ':' + String(text.firstChild) + ':' + String(text.lastChild) + ':' + String(tpl.content.firstChild) + ':' + String(tpl.content.lastChild);</script><!--body-tail--></body></html>",
    )?;

    harness.assert_text(
        "#out",
        "[object Node]:[object Element]:[object Element]:[object Element]:[object Node]:[object Node]:null:null:[object Element]:[object Node]",
    )?;
    Ok(())
}

#[test]
fn tree_reflection_sibling_support_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<!--pre--><html><head></head><body>Text<div id='out'></div><main id='root'><span id='child'>Child</span></main><template id='tpl'><span id='inner'>Inner</span><!--tail--></template><script>const html = document.documentElement; const head = document.head; const body = document.body; const tpl = document.getElementById('tpl'); const content = tpl.content; const text = body.childNodes.item(0); const out = body.childNodes.item(1); document.getElementById('out').textContent = String(document.nextSibling) + ':' + String(document.previousSibling) + ':' + String(html.previousSibling) + ':' + String(head.nextSibling) + ':' + String(body.previousSibling) + ':' + String(body.nextSibling) + ':' + String(body.firstChild.nextSibling) + ':' + String(body.lastChild.previousSibling) + ':' + String(text.nextSibling) + ':' + String(out.previousSibling) + ':' + String(tpl.nextSibling) + ':' + String(tpl.previousSibling) + ':' + String(content.nextSibling) + ':' + String(content.previousSibling) + ':' + String(content.firstChild.nextSibling) + ':' + String(content.lastChild.previousSibling);</script></body></html>",
    )?;

    harness.assert_text(
        "#out",
        "null:null:[object Node]:[object Element]:[object Element]:null:[object Element]:[object Element]:[object Element]:[object Node]:[object Element]:[object Element]:null:null:[object Node]:[object Element]",
    )?;
    Ok(())
}

#[test]
fn tree_reflection_element_sibling_support_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<!--pre--><html><head></head><body>Text<div id='out'></div><main id='root'><span id='child'>Child</span></main><script>const html = document.documentElement; const head = document.head; const body = document.body; const text = body.firstChild; const out = body.childNodes.item(1); const main = body.childNodes.item(2); const script = body.lastChild; document.getElementById('out').textContent = String(document.nextElementSibling) + ':' + String(document.previousElementSibling) + ':' + String(html.nextElementSibling) + ':' + String(html.previousElementSibling) + ':' + String(head.nextElementSibling) + ':' + String(head.previousElementSibling) + ':' + String(body.nextElementSibling) + ':' + String(body.previousElementSibling) + ':' + String(text.nextElementSibling) + ':' + String(text.previousElementSibling) + ':' + String(out.nextElementSibling) + ':' + String(out.previousElementSibling) + ':' + String(main.previousElementSibling) + ':' + String(main.nextElementSibling) + ':' + String(script.previousElementSibling) + ':' + String(script.nextElementSibling);</script></body></html>",
    )?;

    harness.assert_text(
        "#out",
        "null:null:null:null:[object Element]:null:null:[object Element]:[object Element]:null:[object Element]:null:[object Element]:[object Element]:[object Element]:null",
    )?;
    Ok(())
}

#[test]
fn tree_mutation_primitives_reject_cycles_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<main id='root'><section id='child'><span id='grandchild'>x</span></section></main><script>document.getElementById('child').appendChild(document.getElementById('root'));</script>",
    )
    .expect_err("tree mutation should reject ancestor insertion");

    assert!(error.to_string().contains("cannot insert"));
    Ok(())
}

#[test]
fn html_serialization_surfaces_support_inner_html_round_trip_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section><div id='out'></div><script>const target = document.getElementById('target'); const before = target.innerHTML; target.innerHTML = '<span id=\"first\">One</span><span id=\"second\">Two</span>'; const after = target.innerHTML; document.getElementById('out').textContent = before + '|' + after + '|' + String(target.children.length) + ':' + document.querySelector('#second').textContent;</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "<button class=\"primary\" id=\"old\">Old</button>|<span id=\"first\">One</span><span id=\"second\">Two</span>|2:Two",
    )?;
    harness.assert_exists("#target > #first")?;
    harness.assert_exists("#target > #second")?;
    harness.assert_exists("#second")?;
    Ok(())
}

#[test]
fn html_serialization_surfaces_support_outer_html_replacement_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='target'><span id='old'>Old</span></section><div id='out'></div><script>const target = document.getElementById('target'); const before = target.outerHTML; target.outerHTML = '<article id=\"replacement\"><em id=\"inner\">Inner</em></article>'; document.getElementById('out').textContent = before + '|' + document.getElementById('replacement').outerHTML + '|' + String(document.querySelector('#target')) + ':' + document.getElementById('inner').textContent;</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "<section id=\"target\"><span id=\"old\">Old</span></section>|<article id=\"replacement\"><em id=\"inner\">Inner</em></article>|null:Inner",
    )?;
    harness.assert_exists("#replacement")?;
    harness.assert_exists("#replacement > #inner")?;
    assert!(harness.assert_exists("#old").is_err());
    Ok(())
}

#[test]
fn html_serialization_surfaces_support_insert_adjacent_html_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>const target = document.getElementById('target'); target.insertAdjacentHTML('beforebegin', '<aside id=\"before\">Before</aside>'); target.insertAdjacentHTML('afterbegin', '<span id=\"first\">First</span>'); target.insertAdjacentHTML('beforeend', '<span id=\"last\">Last</span>'); target.insertAdjacentHTML('afterend', '<aside id=\"after\">After</aside>'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + target.innerHTML + '|' + String(target.children.length) + ':' + String(document.querySelector('#before')) + ':' + String(document.querySelector('#after'));</script>",
    )?;

    harness.assert_text(
        "#out",
        "<aside id=\"before\">Before</aside><section id=\"target\"><span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span></section><aside id=\"after\">After</aside>|<span id=\"first\">First</span><button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span>|3:[object Element]:[object Element]",
    )?;
    harness.assert_exists("#before")?;
    harness.assert_exists("#after")?;
    harness.assert_exists("#target > #first")?;
    harness.assert_exists("#target > #last")?;
    Ok(())
}

#[test]
fn html_serialization_surfaces_support_insert_adjacent_element_and_text_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='target'><button id='old' class='primary'>Old</button></section></main><div id='out'></div><script>const target = document.getElementById('target'); const before = target.insertAdjacentElement('beforebegin', document.createElement('aside')); before.setAttribute('id', 'before'); before.textContent = 'Before'; target.insertAdjacentText('afterbegin', 'First'); const last = target.insertAdjacentElement('beforeend', document.createElement('span')); last.setAttribute('id', 'last'); last.textContent = 'Last'; const after = target.insertAdjacentElement('afterend', document.createElement('aside')); after.setAttribute('id', 'after'); after.textContent = 'After'; target.insertAdjacentText('beforeend', 'Tail'); document.getElementById('out').textContent = document.getElementById('root').innerHTML + '|' + target.innerHTML + '|' + String(before) + ':' + String(after);</script>",
    )?;

    harness.assert_text(
        "#out",
        "<aside id=\"before\">Before</aside><section id=\"target\">First<button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span>Tail</section><aside id=\"after\">After</aside>|First<button class=\"primary\" id=\"old\">Old</button><span id=\"last\">Last</span>Tail|[object Element]:[object Element]",
    )?;
    harness.assert_exists("#before")?;
    harness.assert_exists("#after")?;
    harness.assert_exists("#target > #last")?;
    Ok(())
}

#[test]
fn html_serialization_surfaces_support_document_write_end_to_end() -> browser_tester_next::Result<()>
{
    let harness = Harness::from_html(
        "<main id='root'><div id='out'></div><script>document.write('<span id=\"written\">Written</span>'); document.getElementById('out').textContent = document.getElementById('written').textContent + ':' + document.getElementById('root').lastElementChild.getAttribute('id');</script></main>",
    )?;

    harness.assert_text("#out", "Written:written")?;
    harness.assert_exists("#written")?;
    Ok(())
}

#[test]
fn html_serialization_surfaces_support_document_writeln_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='out'></div><script>document.writeln('<span id=\"written\">Written</span>'); document.getElementById('out').textContent = document.getElementById('written').textContent + ':' + document.getElementById('root').lastElementChild.getAttribute('id') + ':' + String(document.getElementById('root').lastChild.nodeType);</script></main>",
    )?;

    harness.assert_text("#out", "Written:written:3")?;
    harness.assert_exists("#written")?;
    Ok(())
}

#[test]
fn html_serialization_surfaces_support_document_open_and_close_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<html><head><title>Title</title></head><body><main id='root'><div id='out'></div></main><script>const opened = document.open(); document.write('<span id=\"written\">Written</span><div id=\"out\"></div>'); const closed = document.close(); document.getElementById('out').textContent = String(opened) + ':' + String(closed) + ':' + String(document.documentElement) + ':' + String(document.body);</script></body></html>",
    )?;

    harness.assert_exists("#written")?;
    harness.assert_text(
        "#out",
        "[object Document]:[object Document]:[object Element]:null",
    )?;
    assert!(harness.assert_exists("#root").is_err());
    Ok(())
}

#[test]
fn html_serialization_surfaces_use_namespace_aware_names_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><svg id='icon' viewbox='0 0 10 10'><foreignobject id='foreign'><div id='html'>Text</div></foreignobject></svg><math id='formula' definitionurl='https://example.com'><mi id='symbol'>x</mi></math><div id='out'></div><script>const icon = document.getElementById('icon'); const formula = document.getElementById('formula'); document.getElementById('out').textContent = icon.outerHTML + '|' + formula.outerHTML;</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "<svg id=\"icon\" viewBox=\"0 0 10 10\"><foreignObject id=\"foreign\"><div id=\"html\">Text</div></foreignObject></svg>|<math definitionURL=\"https://example.com\" id=\"formula\"><mi id=\"symbol\">x</mi></math>",
    )?;
    harness.assert_exists("svg > foreignobject")?;
    harness.assert_exists("math > mi")?;
    Ok(())
}

#[test]
fn html_serialization_surfaces_reject_insert_adjacent_html_positions_end_to_end()
-> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<main id='root'><img id='image'><section id='target'></section></main><script>document.getElementById('target').insertAdjacentHTML('middle', '<span id=\"bad\">Bad</span>');</script>",
    )
    .expect_err("invalid insertAdjacentHTML positions should fail");

    assert!(
        error
            .to_string()
            .contains("unsupported insertAdjacentHTML position")
    );
    Ok(())
}

#[test]
fn html_serialization_surfaces_reject_insert_adjacent_element_positions_end_to_end()
-> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<main id='root'><section id='target'></section></main><script>document.getElementById('target').insertAdjacentElement('middle', document.createElement('aside'));</script>",
    )
    .expect_err("invalid insertAdjacentElement positions should fail");

    assert!(
        error
            .to_string()
            .contains("unsupported insertAdjacentElement position")
    );
    Ok(())
}

#[test]
fn html_serialization_surfaces_reject_insert_adjacent_text_on_void_elements_end_to_end()
-> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<main id='root'><img id='image'></main><script>document.getElementById('image').insertAdjacentText('beforeend', 'Bad');</script>",
    )
    .expect_err("void elements should reject insertAdjacentText beforeend");

    assert!(
        error
            .to_string()
            .contains("insertAdjacentText is not supported on void elements")
    );
    Ok(())
}

#[test]
fn html_serialization_surfaces_reject_insert_adjacent_html_on_void_elements_end_to_end()
-> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<main id='root'><img id='image'></main><script>document.getElementById('image').insertAdjacentHTML('beforeend', '<span id=\"bad\">Bad</span>');</script>",
    )
    .expect_err("void elements should reject insertAdjacentHTML beforeend");

    assert!(
        error
            .to_string()
            .contains("insertAdjacentHTML is not supported on void elements")
    );
    Ok(())
}

#[test]
fn mutation_hardening_updates_live_collections_and_selectors_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><form id='form'><input id='first' name='first' value='one'></form><select id='mode'><option value='a'>A</option></select><div id='out'></div><script>const form = document.getElementById('form'); const select = document.getElementById('mode'); const formsBefore = document.forms.length; const inputsBefore = document.querySelectorAll('input').length; form.outerHTML = '<div id=\"form-replacement\"></div>'; select.innerHTML = '<option id=\"second\" value=\"b\" selected>B</option><option id=\"third\" value=\"c\">C</option>'; document.getElementById('out').textContent = formsBefore + ':' + document.forms.length + ':' + inputsBefore + ':' + document.querySelectorAll('input').length + ':' + select.options.length + ':' + document.querySelector('option:checked').value;</script></main>",
    )?;

    harness.assert_text("#out", "1:0:1:0:2:b")?;
    harness.assert_exists("#form-replacement")?;
    harness.assert_exists("option:checked")?;
    harness.assert_exists("#third")?;
    assert!(harness.assert_exists("#form").is_err());
    Ok(())
}

#[test]
fn select_selected_options_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='first' value='a' selected>A</option><option id='second' value='b'>B</option></select><div id='out'></div><script>const select = document.getElementById('mode'); const selected = select.selectedOptions; const before = selected.length; const first = selected.item(0); select.innerHTML = '<option id=\"third\" value=\"c\" selected>C</option><option id=\"fourth\" value=\"d\" selected>D</option>'; document.getElementById('out').textContent = String(before) + ':' + String(selected.length) + ':' + first.textContent + ':' + selected.item(0).textContent + ':' + selected.item(1).textContent + ':' + String(selected.namedItem('third')) + ':' + String(selected.namedItem('missing'));</script></main>",
    )?;

    harness.assert_text("#out", "1:2:A:C:D:[object Element]:null")?;
    harness.assert_exists("#third")?;
    harness.assert_exists("#fourth")?;
    Ok(())
}

#[test]
fn select_options_collection_add_and_remove_are_live_end_to_end() -> browser_tester_next::Result<()>
{
    let harness = Harness::from_html(
        "<main id='root'><select id='mode'><option id='first' value='a'>A</option></select><option id='extra' value='b'>B</option><div id='out'></div><script>const select = document.getElementById('mode'); const extra = document.getElementById('extra'); const before = select.options.length; select.options.add(extra); const afterAdd = select.options.length; select.options.remove(0); document.getElementById('out').textContent = String(before) + ':' + String(afterAdd) + ':' + String(select.options.length) + ':' + select.options.item(0).getAttribute('id') + ':' + String(select.options.namedItem('first'));</script></main>",
    )?;

    harness.assert_text("#out", "1:2:1:extra:null")?;
    harness.assert_exists("#mode > #extra")?;
    assert!(harness.assert_exists("#first").is_err());
    Ok(())
}

#[test]
fn select_options_collection_rejects_datalist_mutation_end_to_end() {
    let error = Harness::from_html(
        "<main id='root'><select id='mode'><option id='first' value='a'>A</option></select><datalist id='list'><option id='extra' value='b'>B</option></datalist><script>document.getElementById('list').options.add(document.getElementById('extra'));</script></main>",
    )
    .expect_err("datalist options should not support select.options.add");

    assert!(error.to_string().contains("node is not a select element"));
}

#[test]
fn element_labels_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><label id='explicit-label' for='control'>Explicit</label><input id='control' value='A'><label id='implicit-label'><input id='inner-control' value='B'>Implicit</label><div id='wrapper'></div><div id='out'></div><script>const control = document.getElementById('control'); const labels = control.labels; const inner = document.getElementById('inner-control').labels; const before = labels.length; document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"control\">Second</label>'; document.getElementById('out').textContent = String(before) + ':' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).textContent + ':' + String(inner.length) + ':' + inner.item(0).getAttribute('id');</script></main>",
    )?;

    harness.assert_text("#out", "1:2:explicit-label:Second:1:implicit-label")?;
    harness.assert_exists("#second-label")?;
    Ok(())
}

#[test]
fn fieldset_elements_and_datalist_options_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><fieldset id='fieldset'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></fieldset><datalist id='list'><option name='alpha' value='a'>A</option><option id='second' value='b'>B</option></datalist><div id='out'></div><script>const elements = document.getElementById('fieldset').elements; const options = document.getElementById('list').options; const beforeElements = elements.length; const beforeOptions = options.length; const first = elements.item(0); const namedElement = elements.namedItem('first'); const namedOption = options.namedItem('second'); document.getElementById('fieldset').textContent = 'gone'; document.getElementById('list').textContent = 'gone'; document.getElementById('out').textContent = String(beforeElements) + ':' + String(elements.length) + ':' + String(beforeOptions) + ':' + String(options.length) + ':' + first.value + ':' + namedElement.value + ':' + namedOption.textContent + ':' + String(options.namedItem('missing'));</script></main>",
    )?;

    harness.assert_text("#out", "2:0:2:0:Ada:Ada:B:null")?;
    harness.assert_exists("fieldset#fieldset")?;
    harness.assert_exists("datalist#list")?;
    Ok(())
}

#[test]
fn radio_node_list_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'></form><div id='out'></div><script>const elements = document.getElementById('signup').elements; const named = elements.namedItem('mode'); const before = named.length; document.getElementById('signup').innerHTML += '<input type=\"radio\" name=\"mode\" id=\"mode-c\" value=\"c\" checked>'; document.getElementById('out').textContent = String(before) + ':' + String(named.length) + ':' + named.item(0).value + ':' + named.item(1).value + ':' + named.value + ':' + String(named);</script></main>",
    )?;

    harness.assert_text("#out", "2:3:a:b:c:[object RadioNodeList]")?;
    harness.assert_exists("#mode-c")?;
    Ok(())
}

#[test]
fn radio_node_list_value_setter_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><form id='signup'><input type='radio' name='mode' id='mode-a' value='a'><input type='radio' name='mode' id='mode-b' value='b'><input type='radio' name='mode' id='mode-c' value='c'></form><div id='out'></div><script>const named = document.getElementById('signup').elements.namedItem('mode'); named.value = 'b'; document.getElementById('out').textContent = named.value + ':' + String(document.getElementById('mode-a').checked) + ':' + String(document.getElementById('mode-b').checked) + ':' + String(document.getElementById('mode-c').checked);</script></main>",
    )?;

    harness.assert_text("#out", "b:false:true:false")?;
    harness.assert_checked("#mode-a", false)?;
    harness.assert_checked("#mode-b", true)?;
    harness.assert_checked("#mode-c", false)?;
    Ok(())
}

#[test]
fn labels_reject_non_labelable_elements_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-labelable'></div></div><script>document.getElementById('not-labelable').labels.length;</script>",
    )
    .expect_err("non-labelable labels access should fail");

    assert!(
        error
            .to_string()
            .contains("node is not a labelable element")
    );
    Ok(())
}

#[test]
fn map_areas_and_table_t_bodies_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><map id='map'><area id='first-area' name='first' href='/first'><area id='second-area' name='second' href='/second'></map><table id='table'><tbody id='first-body'><tr><td>One</td></tr></tbody></table><div id='out'></div><script>const areas = document.getElementById('map').areas; const bodies = document.getElementById('table').tBodies; const beforeAreas = areas.length; const beforeBodies = bodies.length; const firstArea = areas.item(0); const firstBody = bodies.item(0); document.getElementById('map').innerHTML += '<area id=\"third-area\" name=\"third\" href=\"/third\">'; document.getElementById('table').innerHTML += '<tbody id=\"second-body\"></tbody>'; document.getElementById('out').textContent = String(beforeAreas) + ':' + String(areas.length) + ':' + String(beforeBodies) + ':' + String(bodies.length) + ':' + String(firstArea.getAttribute('id')) + ':' + String(firstBody.getAttribute('id')) + ':' + String(areas.namedItem('third-area')) + ':' + String(bodies.namedItem('second-body')) + ':' + String(areas.namedItem('missing'));</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "2:3:1:2:first-area:first-body:[object Element]:[object Element]:null",
    )?;
    harness.assert_exists("#third-area")?;
    harness.assert_exists("#second-body")?;
    Ok(())
}

#[test]
fn select_selected_options_reject_non_select_elements_end_to_end() -> browser_tester_next::Result<()>
{
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-select'></div></div><script>document.getElementById('not-select').selectedOptions.length;</script>",
    )
    .expect_err("non-select selectedOptions access should fail");

    assert!(error.to_string().contains("node is not a select element"));
    Ok(())
}

#[test]
fn type_rejects_non_form_control_elements_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='box'></div></div><script>document.getElementById('box').type;</script>",
    )
    .expect_err("non-form-control type access should fail");

    assert!(error.to_string().contains("type"));
    Ok(())
}

#[test]
fn map_areas_reject_non_map_elements_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-map'></div></div><script>document.getElementById('not-map').areas.length;</script>",
    )
    .expect_err("non-map areas access should fail");

    assert!(error.to_string().contains("map.areas"));
    assert!(
        error
            .to_string()
            .contains("supported map.areas host element")
    );
    Ok(())
}

#[test]
fn table_t_bodies_reject_non_table_elements_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-table'></div></div><script>document.getElementById('not-table').tBodies.length;</script>",
    )
    .expect_err("non-table tBodies access should fail");

    assert!(error.to_string().contains("table.tBodies"));
    assert!(
        error
            .to_string()
            .contains("supported table.tBodies host element")
    );
    Ok(())
}

#[test]
fn html_serialization_surfaces_reject_lossy_attribute_serialization_end_to_end()
-> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<main id='root'><div id='target'></div><div id='out'></div><script>const target = document.getElementById('target'); target.setAttribute('data-label', \"a'b\\\"c\"); document.getElementById('out').textContent = String(target.outerHTML);</script></main>",
    )
    .expect_err("lossy serialization should fail explicitly");

    assert!(error.to_string().contains("contains both quote types"));
    Ok(())
}

#[test]
fn html_serialization_surfaces_reject_malformed_fragments_end_to_end()
-> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<main id='root'><section id='target'></section><script>document.getElementById('target').innerHTML = '<span></main>';</script></main>",
    )
    .expect_err("malformed HTML fragments should fail explicitly");

    assert!(error.to_string().contains("mismatched closing tag"));
    Ok(())
}
