use browser_tester::Harness;

#[test]
fn script_dom_query_selectors_work_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>scope<section><div class='primary'>inside</div></section></main><div id='out'></div><script>const docMatch = document.querySelector('.primary'); const scopedMatch = document.getElementById('root').querySelector('.primary'); const missing = document.getElementById('root').querySelector('.missing'); document.getElementById('out').textContent = docMatch.textContent + ':' + scopedMatch.textContent + ':' + String(missing);</script>",
    )?;

    harness.assert_text("#out", "scopeinside:inside:null")?;
    Ok(())
}

#[test]
fn script_document_root_head_and_body_access_work_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<html id='html'><head id='head'><title>Title</title></head><body id='body'><main id='out'></main><script>const html = document.documentElement; const head = document.head; const body = document.body; document.getElementById('out').textContent = html.getAttribute('id') + ':' + head.getAttribute('id') + ':' + body.getAttribute('id');</script></body></html>",
    )?;

    harness.assert_text("#out", "html:head:body")?;
    Ok(())
}

#[test]
fn script_document_title_access_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<html><head><title>Initial</title></head><body><main id='out'></main><script>const before = document.title; document.title = 'Updated'; const after = window.title; document.getElementById('out').textContent = before + ':' + after + ':' + document.querySelector('title').textContent;</script></body></html>",
    )?;

    harness.assert_text("#out", "Initial:Updated:Updated")?;
    Ok(())
}

#[test]
fn script_dom_query_selector_all_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>root<section><div class='primary'>inside</div></section></main><div id='out'></div><script>const all = document.querySelectorAll('.primary'); const scoped = document.getElementById('root').querySelectorAll('.primary'); document.getElementById('out').textContent = String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent + ':' + String(all.item(2)) + ':' + String(scoped.length) + ':' + scoped.item(0).textContent;</script>",
    )?;

    harness.assert_text("#out", "2:rootinside:inside:null:1:inside")?;
    Ok(())
}

#[test]
fn script_dom_selector_lists_work_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>root</main><div class='primary'>inside</div><div id='out'></div><script>const first = document.querySelector('.primary, main'); const all = document.querySelectorAll('.primary, main'); document.getElementById('out').textContent = first.textContent + ':' + String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent;</script>",
    )?;

    harness.assert_text("#out", "root:2:root:inside")?;
    Ok(())
}

#[test]
fn script_selector_escapes_and_selector_lists_handle_literal_punctuation_end_to_end()
-> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app'><button id='foo,bar' class='alpha:beta'>First</button><button id='second' class='secondary'>Second</button><div id='out'></div></main><script>const escapedId = document.querySelector('#foo\\\\,bar'); const escapedClass = document.querySelector('.alpha\\\\:beta'); const list = document.querySelectorAll('#foo\\\\,bar, .secondary'); const isMatch = document.getElementById('root').matches('main:is(#foo\\\\)bar, .app)'); const whereMatch = document.getElementById('second').closest('button:where(#foo\\\\,bar, .secondary)'); document.getElementById('out').textContent = escapedId.textContent + ':' + escapedClass.textContent + ':' + String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + String(isMatch) + ':' + whereMatch.textContent;</script>",
    )?;

    harness.assert_text("#out", "First:First:2:First:Second:true:Second")?;
    Ok(())
}

#[test]
fn script_selector_hex_escapes_work_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app'><button id='foo,bar' class='alpha:beta' data-label='foo]bar'>First</button><button id='second' class='secondary'>Second</button><div id='out'></div></main><script>const escapedId = document.querySelector('#foo\\\\2c bar'); const escapedClass = document.querySelector('.alpha\\\\3a beta'); const escapedAttr = document.querySelector('[data-label=foo\\\\5d bar]'); const list = document.querySelectorAll('#foo\\\\2c bar, .secondary'); const whereMatch = document.getElementById('second').closest('button:where(#foo\\\\2c bar, .secondary)'); document.getElementById('out').textContent = escapedId.textContent + ':' + escapedClass.textContent + ':' + escapedAttr.textContent + ':' + String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + whereMatch.textContent;</script>",
    )?;

    harness.assert_text("#out", "First:First:First:2:First:Second:Second")?;
    Ok(())
}

#[test]
fn script_selector_lists_ignore_commas_inside_quoted_attribute_values_end_to_end()
-> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app'><button id='first' data-label='A,B'>First</button><button id='second' class='secondary'>Second</button></main><div id='out'></div><script>const list = document.querySelectorAll(\"button[data-label='A,B'], .secondary\"); const isMatch = document.getElementById('root').matches(\"main:is([data-label='A,B'], .app)\"); const notMatch = document.getElementById('second').matches(\"button:not([data-label='A,B'], .blocked)\"); const whereMatch = document.getElementById('root').closest(\"main:where([data-label='A,B'], .app)\"); document.getElementById('out').textContent = String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + String(isMatch) + ':' + String(notMatch) + ':' + whereMatch.textContent;</script>",
    )?;

    harness.assert_text("#out", "2:First:Second:true:true:FirstSecond")?;
    Ok(())
}

#[test]
fn script_attribute_value_selectors_work_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' data-kind='APP-shell' lang='EN-US'><button id='first' data-role='Primary Action' data-tags='Primary Ready' data-label='Primary Action'>First</button><button id='second' data-role='Secondary Action'>Second</button><input id='toggle' disabled></main><div id='out'></div><script>const prefix = document.querySelector(\"button[data-role^=prim i]\"); const strict = document.querySelector(\"button[data-role^='Primary' s]\"); const suffix = document.querySelector(\"[data-label$='action' i]\"); const contains = document.querySelector(\"button[data-role*='ond' i]\"); const token = document.querySelector(\"[data-tags~=ready i]\"); const all = document.querySelectorAll(\"main[data-kind|=app i], button[data-role$='Action' s]\"); const second = document.getElementById('second'); const root = second.closest(\"main:is([lang|=en i], .blocked)\"); const disabled = document.querySelector(\"input[disabled='']\"); document.getElementById('out').textContent = prefix.textContent + ':' + strict.textContent + ':' + suffix.textContent + ':' + contains.textContent + ':' + token.textContent + ':' + String(all.length) + ':' + String(second.matches(\"button[data-role~=secondary i]\")) + ':' + root.textContent + ':' + String(disabled);</script>",
    )?;

    harness.assert_text(
        "#out",
        "First:First:First:Second:First:3:true:FirstSecond:[object Element]",
    )?;
    Ok(())
}

#[test]
fn script_html_collection_children_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='first'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const children = document.getElementById('root').children; const before = children.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + String(children.item(0));</script>",
    )?;

    harness.assert_text("#out", "2:0:null")?;
    Ok(())
}

#[test]
fn script_html_collection_named_item_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const children = document.getElementById('root').children; const alpha = children.namedItem('alpha'); const second = children.namedItem('second'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = alpha.textContent + ':' + second.textContent + ':' + String(children.namedItem('alpha'));</script>",
    )?;

    harness.assert_text("#out", "First:Second:null")?;
    Ok(())
}

#[test]
fn script_get_elements_by_tag_name_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const all = document.getElementsByTagName('span'); const scoped = document.getElementById('root').getElementsByTagName('span'); const alpha = all.namedItem('alpha'); const second = scoped.namedItem('second'); const before = all.length; const beforeScoped = scoped.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(beforeScoped) + ':' + String(scoped.length) + ':' + alpha.textContent + ':' + second.textContent + ':' + String(all.namedItem('alpha'));</script>",
    )?;

    harness.assert_text("#out", "2:0:2:0:First:Second:null")?;
    Ok(())
}

#[test]
fn script_get_elements_by_class_name_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='alpha'><span name='alpha' class='alpha'>First</span><span id='second' class='alpha'>Second</span></main><div id='out'></div><script>const all = document.getElementsByClassName('alpha'); const scoped = document.getElementById('root').getElementsByClassName('alpha'); const named = all.namedItem('alpha'); const root = all.item(0); const before = all.length; const beforeScoped = scoped.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(beforeScoped) + ':' + String(scoped.length) + ':' + named.textContent + ':' + String(scoped.namedItem('alpha')) + ':' + root.textContent;</script>",
    )?;

    harness.assert_text("#out", "3:1:2:0:First:null:gone")?;
    Ok(())
}

#[test]
fn script_get_elements_by_name_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span name='alpha'>First</span><span name='alpha'>Second</span></main><div id='out'></div><script>const nodes = document.getElementsByName('alpha'); const first = nodes.item(0); const before = nodes.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(nodes.length) + ':' + first.textContent + ':' + String(nodes.item(1));</script>",
    )?;

    harness.assert_text("#out", "2:0:First:null")?;
    Ok(())
}

#[test]
fn script_document_forms_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><form id='signup' name='signup'>Signup</form><form id='login' name='login'>Login</form></div><div id='out'></div><script>const forms = document.forms; const first = forms.item(0); const named = forms.namedItem('signup'); const signup = forms.signup; const login = forms.login; const before = forms.length; const firstText = first.textContent; const namedText = named.textContent; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(forms.length) + ':' + firstText + ':' + namedText + ':' + signup.textContent + ':' + login.textContent + ':' + String(forms.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:0:Signup:Signup:Signup:Login:null")?;
    Ok(())
}

#[test]
fn script_form_elements_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><form id='signup'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const first = elements.item(0); const named = elements.namedItem('first'); const before = elements.length; const firstValue = first.value; const namedValue = named.value; document.getElementById('signup').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(elements.length) + ':' + firstValue + ':' + namedValue + ':' + String(elements.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:0:Ada:Ada:null")?;
    Ok(())
}

#[test]
fn script_select_options_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><select id='mode'><option name='alpha' value='a'>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const options = document.getElementById('mode').options; const first = options.item(0); const named = options.namedItem('second'); const before = options.length; const firstText = first.textContent; const namedText = named.textContent; document.getElementById('mode').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(options.length) + ':' + firstText + ':' + namedText + ':' + String(options.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:0:A:B:null")?;
    Ok(())
}

#[test]
fn script_document_images_and_links_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><img id='hero' name='hero' alt='Hero'><img name='thumb' alt='Thumb'><a id='docs' href='/docs'>Docs</a><a id='plain'>Plain</a><area id='map' name='map' href='/map'></div><div id='out'></div><script>const images = document.images; const links = document.links; const beforeImages = images.length; const beforeLinks = links.length; const hero = images.namedItem('hero'); const thumb = images.namedItem('thumb'); const docs = links.namedItem('docs'); const map = links.namedItem('map'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(beforeImages) + ':' + String(images.length) + ':' + String(beforeLinks) + ':' + String(links.length) + ':' + String(hero) + ':' + String(thumb) + ':' + String(docs) + ':' + String(map) + ':' + String(links.namedItem('plain'));</script>",
    )?;

    harness.assert_text(
        "#out",
        "2:0:2:0:[object Element]:[object Element]:[object Element]:[object Element]:null",
    )?;
    Ok(())
}

#[test]
fn script_document_links_iterator_helpers_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><img id='hero' name='hero' alt='Hero'><img name='thumb' alt='Thumb'><a id='docs' href='/docs'>Docs</a><a id='plain'>Plain</a><area id='map' name='map' href='/map'></div><div id='out'></div><script>const links = document.links; const keys = links.keys(); const values = links.values(); const entries = links.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; links.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>",
    )?;

    harness.assert_text("#out", "0:docs:0:docs:0:docs:2;1:map:2;")?;
    Ok(())
}

#[test]
fn script_document_applets_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><applet id='first-applet' name='first-applet'>First</applet><applet name='second-applet'>Second</applet></div><div id='out'></div><script>const applets = document.applets; const before = applets.length; const first = applets.namedItem('first-applet'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(applets.length) + ':' + String(first) + ':' + String(applets.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:0:[object Element]:null")?;
    Ok(())
}

#[test]
fn script_document_applets_iterator_helpers_are_live_end_to_end() -> browser_tester::Result<()>
{
    let harness = Harness::from_html(
        "<div id='root'><applet id='first-applet' name='first-applet'>First</applet><applet id='second-applet' name='second-applet'>Second</applet></div><div id='out'></div><script>const applets = document.applets; const keys = applets.keys(); const values = applets.values(); const entries = applets.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; applets.forEach((element, index, list) => { out += String(index) + ':' + element.textContent + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.textContent + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.textContent + ':' + out;</script>",
    )?;

    harness.assert_text("#out", "0:First:0:First:0:First:2;1:Second:2;")?;
    Ok(())
}

#[test]
fn script_document_all_is_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script>const all = document.all; const before = all.length; const named = all.namedItem('second'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(named) + ':' + String(all.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "5:3:[object Element]:null")?;
    Ok(())
}

#[test]
fn script_document_all_iterator_helpers_are_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script id='script'>const all = document.all; const keys = all.keys(); const values = all.values(); const entries = all.entries(); const firstKey = keys.next(); const firstValue = values.next(); const firstEntry = entries.next(); let out = ''; all.forEach((element, index, list) => { out += String(index) + ':' + element.getAttribute('id') + ':' + String(list.length) + ';'; }); document.getElementById('out').textContent = String(firstKey.value) + ':' + firstValue.value.getAttribute('id') + ':' + String(firstEntry.value.index) + ':' + firstEntry.value.value.getAttribute('id') + ':' + out;</script>",
    )?;

    harness.assert_text(
        "#out",
        "0:root:0:root:0:root:5;1:first:5;2:second:5;3:out:5;4:script:5;",
    )?;
    Ok(())
}

#[test]
fn script_get_elements_by_tag_name_ns_is_live_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><svg id='icon'><rect id='rect'></rect><circle id='dot'></circle></svg><math id='formula'><mi id='symbol'>x</mi></math><span id='label'>Label</span></div><div id='out'></div><script>const svgAll = document.getElementsByTagNameNS('http://www.w3.org/2000/svg', '*'); const svgRect = document.getElementById('icon').getElementsByTagNameNS('http://www.w3.org/2000/svg', 'rect'); const htmlSpan = document.getElementsByTagNameNS('http://www.w3.org/1999/xhtml', 'span'); const mathAll = document.getElementsByTagNameNS('http://www.w3.org/1998/Math/MathML', '*'); const beforeSvgAll = svgAll.length; const beforeSvgRect = svgRect.length; const beforeHtmlSpan = htmlSpan.length; const beforeMathAll = mathAll.length; const dot = svgAll.namedItem('dot'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(beforeSvgAll) + ':' + String(svgAll.length) + ':' + String(beforeSvgRect) + ':' + String(svgRect.length) + ':' + String(beforeHtmlSpan) + ':' + String(htmlSpan.length) + ':' + String(beforeMathAll) + ':' + String(mathAll.length) + ':' + String(dot) + ':' + String(svgAll.namedItem('dot'));</script>",
    )?;

    harness.assert_text("#out", "3:0:1:1:1:0:2:0:[object Element]:null")?;
    Ok(())
}

#[test]
fn script_simple_pseudo_classes_work_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main>lead<!-- gap --><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='primary'>Enabled</button><input id='agree' type='checkbox' checked><select id='mode'><option value='a'>A</option><option id='selected' value='b' selected>B</option></select></main><div id='out'></div><script>const first = document.querySelector('#first:first-child'); const second = document.querySelector('button:nth-child(2)'); const third = document.querySelector('button:nth-child(3)'); const oddButtons = document.querySelectorAll('button:nth-child(odd)'); const evenButton = document.querySelector('button:nth-child(even)'); const formula = document.querySelector('button:nth-child(2n+1)'); const limited = document.querySelectorAll('button:nth-child(-n+2)'); const lastFirst = document.querySelector('button:nth-last-child(5)'); const lastSecond = document.querySelector('button:nth-last-child(4)'); const lastOdd = document.querySelectorAll('button:nth-last-child(odd)'); const lastEven = document.querySelector('button:nth-last-child(even)'); const lastFormula = document.querySelector('button:nth-last-child(2n+1)'); const disabled = document.querySelector('button:disabled'); const enabled = document.querySelectorAll('button:enabled'); const checked = document.querySelector('input:checked'); const selected = document.querySelector('option:checked'); document.getElementById('out').textContent = first.textContent + ':' + second.textContent + ':' + third.textContent + ':' + evenButton.textContent + ':' + formula.textContent + ':' + String(oddButtons.length) + ':' + String(limited.length) + ':' + lastFirst.textContent + ':' + lastSecond.textContent + ':' + String(lastOdd.length) + ':' + lastEven.textContent + ':' + lastFormula.textContent + ':' + disabled.textContent + ':' + String(enabled.length) + ':' + checked.checked + ':' + selected.textContent;</script>",
    )?;

    harness.assert_text(
        "#out",
        "First:Disabled:Enabled:Disabled:First:2:2:First:Disabled:2:Disabled:First:Disabled:2:true:B",
    )?;
    Ok(())
}

#[test]
fn script_default_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><form id='form'><input id='submit' type='submit'><input id='agree' type='checkbox' checked><input id='mode-a' type='radio' name='mode'><input id='mode-b' type='radio' name='mode' checked><select id='select'><option id='first' value='a'>A</option><option id='selected' value='b' selected>B</option></select></form></main><div id='out'></div><script>const defaults = document.querySelectorAll(':default'); const submit = document.querySelector('#submit:default'); const agree = document.querySelector('#agree:default'); const radio = document.querySelector('#mode-b:default'); const selected = document.querySelector('#selected:default'); document.getElementById('out').textContent = String(defaults.length) + ':' + submit.getAttribute('id') + ':' + agree.getAttribute('id') + ':' + radio.getAttribute('id') + ':' + selected.getAttribute('id');</script>",
    )?;

    harness.assert_text("#out", "4:submit:agree:mode-b:selected")?;
    Ok(())
}

#[test]
fn script_root_and_empty_pseudo_classes_work_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='empty-comment'><!-- gap --></section><section id='empty'></section><section id='non-empty'>content</section><div id='out'>seed</div></main><script>const root = document.querySelector(':root'); const empties = document.querySelectorAll('#root :empty'); const emptyComment = document.getElementById('empty-comment'); const nonEmpty = document.getElementById('non-empty'); document.getElementById('out').textContent = String(root.matches(':root')) + ':' + String(empties.length) + ':' + empties.item(0).matches(':empty') + ':' + empties.item(1).matches(':empty') + ':' + String(emptyComment.matches(':empty')) + ':' + String(nonEmpty.matches(':empty'));</script>",
    )?;

    harness.assert_text("#out", "true:2:true:true:true:false")?;
    Ok(())
}

#[test]
fn script_only_child_and_only_of_type_pseudo_classes_work_end_to_end()
-> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'>lead<!-- gap --><div id='single-child-parent'>text<!-- marker --><section id='only-child'>child</section><!-- marker --></div><div id='type-parent'><span id='first-span'>one</span><em id='only-of-type'>type</em><span id='second-span'>two</span></div><div id='out'>seed</div><script>const onlyChild = document.querySelector('#only-child:only-child'); const onlyOfType = document.querySelector('#only-of-type:only-of-type'); const onlyChildMatches = document.querySelectorAll('#single-child-parent > :only-child'); const onlyOfTypeMatches = document.querySelectorAll('#type-parent > :only-of-type'); const firstSpan = document.getElementById('first-span'); const firstSpanNotOnlyChild = firstSpan.matches('#first-span:not(:only-child)'); const firstSpanNotOnlyOfType = firstSpan.matches('#first-span:not(:only-of-type)'); const parent = onlyChild.closest('#single-child-parent'); document.getElementById('out').textContent = onlyChild.textContent + ':' + onlyOfType.textContent + ':' + String(onlyChildMatches.length) + ':' + String(onlyOfTypeMatches.length) + ':' + String(firstSpanNotOnlyChild) + ':' + String(firstSpanNotOnlyOfType) + ':' + String(parent.matches('#single-child-parent'));</script></main>",
    )?;

    harness.assert_text("#out", "child:type:1:1:true:true:true")?;
    Ok(())
}

#[test]
fn script_first_last_and_nth_of_type_pseudo_classes_work_end_to_end()
-> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='type-parent'><span id='first-span' class='skip'>one</span><em id='first-em'>first</em><span id='middle-span' class='match'>two</span><em id='last-em'>last</em><span id='last-span' class='match'>three</span></div><div id='out'>seed</div><script>const firstSpan = document.querySelector('#first-span:first-of-type'); const lastSpan = document.querySelector('#last-span:last-of-type'); const middleSpan = document.querySelector('#middle-span:nth-of-type(2)'); const filteredMiddle = document.querySelector('#middle-span:nth-of-type(1 of .match)'); const filteredLast = document.querySelector('#last-span:nth-last-of-type(1 of .match)'); const middleFromEnd = document.querySelector('#middle-span:nth-last-of-type(2)'); const firstEm = document.querySelector('#first-em:first-of-type'); const lastEm = document.querySelector('#last-em:last-of-type'); document.getElementById('out').textContent = String(firstSpan.matches('#first-span:first-of-type')) + ':' + String(lastSpan.matches('#last-span:last-of-type')) + ':' + String(middleSpan.matches('#middle-span:nth-of-type(2)')) + ':' + String(filteredMiddle.matches('#middle-span:nth-of-type(1 of .match)')) + ':' + String(filteredLast.matches('#last-span:nth-last-of-type(1 of .match)')) + ':' + String(middleFromEnd.matches('#middle-span:nth-last-of-type(2)')) + ':' + String(firstEm.matches('#first-em:first-of-type')) + ':' + String(lastEm.matches('#last-em:last-of-type'));</script></main>",
    )?;

    harness.assert_text("#out", "true:true:true:true:true:true:true:true")?;
    Ok(())
}

#[test]
fn script_not_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app' data-kind='app'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button><div id='out'></div><script>const enabled = document.querySelectorAll('button:not(:disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:not([data-kind*=blocked], .blocked)'); const bounded = document.querySelectorAll('button:not(main > .secondary, :disabled)'); document.getElementById('out').textContent = String(enabled.length) + ':' + enabled.item(0).textContent + ':' + enabled.item(1).textContent + ':' + String(second.matches('button:not(.primary)')) + ':' + String(root.matches('main:not([data-kind*=blocked], .blocked)')) + ':' + document.querySelector('button:not(:nth-child(even))').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script></main>",
    )?;

    harness.assert_text("#out", "2:First:Enabled:true:true:First:1:First")?;
    Ok(())
}

#[test]
fn script_is_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app' data-kind='app'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const all = document.querySelectorAll('button:is(.primary, .secondary)'); const filtered = document.querySelectorAll('button:is(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:is(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:is([data-kind^=ap], .blocked)'); document.getElementById('out').textContent = String(all.length) + ':' + String(filtered.length) + ':' + String(second.matches('button:is(.secondary, .blocked)')) + ':' + String(root.matches('main:is([data-kind^=ap], .blocked)')) + ':' + document.querySelector('button:is(.primary, .secondary):not(:disabled)').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script>",
    )?;

    harness.assert_text("#out", "3:2:true:true:First:2:Disabled")?;
    Ok(())
}

#[test]
fn script_has_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='first' class='child'>First</section><section id='child' class='child'><div class='grandchild'>Grand</div></section></main><div id='out'></div><script>const docMatch = document.querySelector('main:has(#child)'); const directMatch = document.querySelector('main:has(> .child)'); const nthMatch = document.querySelector('main:has(:nth-child(2 of .child))'); const root = document.getElementById('root'); const section = document.getElementById('child'); const nested = document.querySelector('main:has(section .grandchild)'); const closest = section.closest('main:has(> .child)'); document.getElementById('out').textContent = docMatch.getAttribute('id') + ':' + directMatch.getAttribute('id') + ':' + nthMatch.getAttribute('id') + ':' + String(root.matches('main:has(> .child)')) + ':' + String(section.matches(':has(.grandchild)')) + ':' + closest.getAttribute('id') + ':' + nested.getAttribute('id');</script>",
    )?;

    harness.assert_text("#out", "root:root:root:true:true:root:root")?;
    Ok(())
}

#[test]
fn script_where_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const all = document.querySelectorAll('button:where(.primary, .secondary)'); const filtered = document.querySelectorAll('button:where(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:where(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:where([lang|=en i], .blocked)'); document.getElementById('out').textContent = String(all.length) + ':' + String(filtered.length) + ':' + String(second.matches('button:where(.secondary, .blocked)')) + ':' + String(root.matches('main:where([lang|=en i], .blocked)')) + ':' + document.querySelector('button:where(.primary, .secondary):not(:disabled)').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script>",
    )?;

    harness.assert_text("#out", "3:2:true:true:First:2:Disabled")?;
    Ok(())
}

#[test]
fn script_general_sibling_selectors_work_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main><button id='first' class='primary'>First</button>text<button id='second' class='primary'>Second</button><div id='out'></div><script>const sibling = document.querySelector('#first ~ .primary'); const second = document.getElementById('second'); document.getElementById('out').textContent = sibling.textContent + ':' + String(second.matches('#first ~ .primary'));</script></main>",
    )?;

    harness.assert_text("#out", "Second:true")?;
    Ok(())
}

#[test]
fn script_element_matches_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'><section><div id='child' class='child'></div></section></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = String(root.matches('.primary')) + ':' + String(root.matches('.child')) + ':' + String(child.matches('.child'));</script>",
    )?;

    harness.assert_text("#out", "true:false:true")?;
    Ok(())
}

#[test]
fn script_element_closest_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>ROOT<section id='section'>SECTION<div id='child' class='child'>CHILD</div></section></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = root.closest('.primary').textContent + ':' + child.closest('.child').textContent + ':' + child.closest('#section').textContent + ':' + String(child.closest('.missing'));</script>",
    )?;

    harness.assert_text("#out", "ROOTSECTIONCHILD:CHILD:SECTIONCHILD:null")?;
    Ok(())
}

#[test]
fn script_scope_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='section'><div id='child'>Child</div></section></main><div id='out'></div><script>const docScope = document.querySelector(':scope'); const root = document.getElementById('root'); const section = root.querySelector(':scope > section'); const missing = root.querySelector(':scope'); const matches = root.matches(':scope'); const closest = document.getElementById('child').closest(':scope'); document.getElementById('out').textContent = docScope.getAttribute('id') + ':' + section.getAttribute('id') + ':' + String(missing) + ':' + String(matches) + ':' + closest.getAttribute('id');</script>",
    )?;

    harness.assert_text("#out", "root:section:null:true:child")?;
    Ok(())
}

#[test]
fn script_lang_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' lang='EN-US'><section id='section'><div id='child'>Child</div></section><p id='french' lang='fr'>French</p></main><div id='out'></div><script>const root = document.querySelector(':lang(en, fr)'); const section = document.querySelector('#section:lang(en)'); const child = document.querySelector('#child:lang(en)'); const french = document.querySelector('#french:lang(fr)'); const closest = document.getElementById('child').closest(':lang(fr, en)'); document.getElementById('out').textContent = root.getAttribute('id') + ':' + section.getAttribute('id') + ':' + child.getAttribute('id') + ':' + french.getAttribute('id') + ':' + closest.getAttribute('id');</script>",
    )?;

    harness.assert_text("#out", "root:section:child:french:child")?;
    Ok(())
}

#[test]
fn script_any_link_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><a id='docs' href='/docs'>Docs</a><a id='plain'>Plain</a><area id='map' href='/map'></main><div id='out'></div><script>const anyLink = document.querySelector(':any-link'); const links = document.querySelectorAll(':link'); const anchor = document.getElementById('docs'); const matched = anchor.matches(':any-link'); const closest = anchor.closest(':link'); document.getElementById('out').textContent = anyLink.getAttribute('id') + ':' + String(links.length) + ':' + links.item(0).getAttribute('id') + ':' + links.item(1).getAttribute('id') + ':' + String(matched) + ':' + closest.getAttribute('id');</script>",
    )?;

    harness.assert_text("#out", "docs:2:docs:map:true:docs")?;
    Ok(())
}

#[test]
fn script_defined_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><x-widget id='widget'></x-widget><svg id='svg'><text id='svg-text'>Hi</text></svg></main><div id='out'></div><script>const defined = document.querySelectorAll(':defined'); const widget = document.getElementById('widget'); const svg = document.getElementById('svg'); document.getElementById('out').textContent = defined.item(0).getAttribute('id') + ':' + defined.item(1).getAttribute('id') + ':' + defined.item(2).getAttribute('id') + ':' + String(widget.matches(':defined')) + ':' + String(svg.matches(':defined'));</script>",
    )?;

    harness.assert_text("#out", "root:svg:svg-text:false:true")?;
    Ok(())
}

#[test]
fn script_dir_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' dir='rtl'><section id='section'><div id='child'>Child</div></section><p id='ltr' dir='ltr'>LTR</p><div id='auto' dir='auto'><span id='auto-child'>Auto</span></div></main><div id='out'></div><script>const dir = document.querySelector(':dir(rtl)'); const dirAll = document.querySelectorAll(':dir(rtl)'); const section = document.querySelector('#section:dir(rtl)'); const child = document.getElementById('child').closest(':dir(rtl)'); const autoChild = document.querySelector('#auto-child:dir(rtl)'); document.getElementById('out').textContent = dir.getAttribute('id') + ':' + String(dirAll.length) + ':' + section.getAttribute('id') + ':' + child.getAttribute('id') + ':' + autoChild.getAttribute('id');</script>",
    )?;

    harness.assert_text("#out", "root:5:section:child:auto-child")?;
    Ok(())
}

#[test]
fn script_placeholder_shown_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='name' placeholder='Name'><input id='filled' placeholder='Filled' value='Ada'><textarea id='bio' placeholder='Bio'></textarea></main><div id='out'></div><script>const before = document.querySelectorAll(':placeholder-shown'); document.getElementById('name').value = 'Alice'; document.getElementById('bio').value = 'Bio text'; const after = document.querySelectorAll(':placeholder-shown'); const name = document.getElementById('name'); document.getElementById('out').textContent = String(before.length) + ':' + before.item(0).getAttribute('id') + ':' + String(after.length) + ':' + String(name.matches(':placeholder-shown')) + ':' + String(document.getElementById('filled').matches(':placeholder-shown'));</script>",
    )?;

    harness.assert_text("#out", "2:name:0:false:false")?;
    Ok(())
}

#[test]
fn script_blank_pseudo_class_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='empty' type='text' value='  '><textarea id='bio'> \n </textarea><input id='check' type='checkbox'></main><div id='out'></div><script>const blank = document.querySelectorAll(':blank'); const empty = document.getElementById('empty'); const bio = document.getElementById('bio'); const check = document.getElementById('check'); document.getElementById('out').textContent = String(blank.length) + ':' + blank.item(0).getAttribute('id') + ':' + blank.item(1).getAttribute('id') + ':' + String(empty.matches(':blank')) + ':' + String(bio.matches(':blank')) + ':' + String(check.matches(':blank'));</script>",
    )?;

    harness.assert_text("#out", "2:empty:bio:true:true:false")?;
    Ok(())
}

#[test]
fn script_read_only_and_read_write_pseudo_classes_work_end_to_end()
-> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='name' value='Ada'><input id='readonly' value='Bee' readonly><textarea id='bio'>Hello</textarea><div id='editable' contenteditable='true'>Edit</div><select id='mode'><option value='a'>A</option></select><button id='button'>Button</button></main><div id='out'></div><script>const readWrite = document.querySelectorAll(':read-write'); const readOnly = document.querySelectorAll(':read-only'); document.getElementById('out').textContent = String(readWrite.length) + ':' + readWrite.item(0).getAttribute('id') + ':' + readWrite.item(1).getAttribute('id') + ':' + readWrite.item(2).getAttribute('id') + ':' + String(readOnly.item(0).matches(':read-only')) + ':' + String(document.getElementById('readonly').matches(':read-only')) + ':' + String(document.getElementById('mode').matches(':read-only')) + ':' + String(document.getElementById('button').matches(':read-only'));</script>",
    )?;

    harness.assert_text("#out", "3:name:bio:editable:true:true:true:true")?;
    Ok(())
}

#[test]
fn script_read_only_and_read_write_pseudo_classes_fail_explicitly() {
    let error = Harness::from_html("<main id='root'><input id='name' value='Ada'></main><div id='out'></div><script>document.querySelector(':read-only()');</script>")
        .expect_err("malformed read-only selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector `:read-only()`"));
}

#[test]
fn script_valid_and_invalid_pseudo_classes_work_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='filled' type='text' required value='Ada'><input id='empty' type='text' required><input id='check' type='checkbox' required><input id='check-ok' type='checkbox' required checked><textarea id='bio' required></textarea><select id='mode' required><option value='a' selected>A</option><option value='b'>B</option></select></main><div id='out'></div><script>const beforeValid = document.querySelectorAll(':valid'); const beforeInvalid = document.querySelectorAll(':invalid'); document.getElementById('empty').value = 'Bee'; document.getElementById('check').setAttribute('checked', ''); const afterValid = document.querySelectorAll(':valid'); const afterInvalid = document.querySelectorAll(':invalid'); const empty = document.getElementById('empty'); const check = document.getElementById('check'); document.getElementById('out').textContent = String(beforeValid.length) + ':' + beforeValid.item(0).getAttribute('id') + ':' + beforeValid.item(1).getAttribute('id') + ':' + beforeValid.item(2).getAttribute('id') + ':' + String(beforeInvalid.length) + ':' + beforeInvalid.item(0).getAttribute('id') + ':' + beforeInvalid.item(1).getAttribute('id') + ':' + beforeInvalid.item(2).getAttribute('id') + ':' + String(afterValid.length) + ':' + String(afterInvalid.length) + ':' + String(empty.matches(':valid')) + ':' + String(check.matches(':valid'));</script>",
    )?;

    harness.assert_text(
        "#out",
        "3:filled:check-ok:mode:3:empty:check:bio:5:1:true:true",
    )?;
    Ok(())
}

#[test]
fn script_valid_and_invalid_pseudo_classes_fail_explicitly() {
    let error = Harness::from_html("<main id='root'><input id='name' type='text' required></main><div id='out'></div><script>document.querySelector(':valid()');</script>")
        .expect_err("malformed valid selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector `:valid()`"));
}

#[test]
fn script_in_range_and_out_of_range_pseudo_classes_work_end_to_end()
-> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><input id='low' type='number' min='2' max='6' value='1'><input id='high' type='number' min='2' max='6' value='7'><input id='in-range' type='number' min='2' max='6' value='4'><div id='out'></div><script>const inRange = document.querySelectorAll(':in-range'); const outOfRange = document.querySelectorAll(':out-of-range'); document.getElementById('out').textContent = String(inRange.length) + ':' + inRange.item(0).getAttribute('id') + ':' + String(outOfRange.length) + ':' + outOfRange.item(0).getAttribute('id') + ':' + outOfRange.item(1).getAttribute('id') + ':' + String(document.getElementById('in-range').matches(':in-range')) + ':' + String(document.getElementById('low').matches(':out-of-range')) + ':' + String(document.getElementById('high').matches(':out-of-range'));</script></main>",
    )?;

    harness.assert_text("#out", "1:in-range:2:low:high:true:true:true")?;
    Ok(())
}

#[test]
fn script_in_range_and_out_of_range_pseudo_classes_fail_explicitly() {
    let error = Harness::from_html("<main id='root'><input id='name' type='number' min='2' max='6' value='4'></main><div id='out'></div><script>document.querySelector(':in-range()');</script>")
        .expect_err("malformed in-range selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector `:in-range()`"));
}

#[test]
fn script_focus_pseudo_classes_work_end_to_end() -> browser_tester::Result<()> {
    let mut harness = Harness::from_html(
        "<main id='root'><section id='section'><input id='field'></section><div id='out'></div><script>document.getElementById('field').addEventListener('focus', () => { const field = document.querySelector(':focus'); const section = document.getElementById('section'); const root = document.getElementById('root'); document.getElementById('out').textContent = field.getAttribute('id') + ':' + String(section.matches(':focus-within')) + ':' + String(root.matches(':focus-within')); });</script></main>",
    )?;

    harness.focus("#field")?;
    harness.assert_text("#out", "field:true:true")?;
    Ok(())
}

#[test]
fn script_target_pseudo_class_tracks_url_fragments_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html_with_url(
        "https://example.test/app#named",
        "<main id='root'><section id='target'>Target</section><a id='fallback' name='fallback'>Fallback</a><span name='named'>Named</span></main><div id='out'></div><script>const target = document.querySelector(':target'); document.getElementById('out').textContent = target.textContent + ':' + String(target.getAttribute('name')) + ':' + String(document.getElementById('root').matches(':target')) + ':' + String(document.getElementById('fallback').matches(':target'));</script>",
    )?;

    harness.assert_text("#out", "Named:named:false:false")?;
    Ok(())
}

#[test]
fn script_node_list_for_each_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span class='item'>First</span><span class='item'>Second</span></main><div id='out'></div><script>const nodes = document.querySelectorAll('.item'); nodes.forEach((item, index, list) => { document.getElementById('out').textContent += String(index) + ':' + item.textContent + ':' + String(list.length) + ';'; }, null);</script>",
    )?;

    harness.assert_text("#out", "0:First:2;1:Second:2;")?;
    Ok(())
}

#[test]
fn script_html_collection_for_each_works_end_to_end() -> browser_tester::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span>child</span><span>more</span></main><div id='out'></div><script>const children = document.getElementById('root').children; children.forEach((child, index, list) => { document.getElementById('out').textContent += String(index) + ':' + child.textContent + ':' + String(list.length) + ';'; }, null);</script>",
    )?;

    harness.assert_text("#out", "0:child:2;1:more:2;")?;
    Ok(())
}

#[test]
fn unsupported_element_get_elements_by_name_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root'><span name='alpha'>First</span></main><script>document.getElementById('root').getElementsByName('alpha');</script>",
    )
    .expect_err("unsupported element method should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported Element method: getElementsByName"));
}

#[test]
fn unsupported_form_elements_on_non_form_elements_fails_explicitly() {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-form'></div></div><script>document.getElementById('wrapper').elements.length;</script>",
    )
    .expect_err("non-form elements should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("node is not a form element"));
}

#[test]
fn unsupported_select_options_on_non_select_elements_fails_explicitly() {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-select'></div></div><script>document.getElementById('not-select').options.length;</script>",
    )
    .expect_err("non-select elements should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("node is not a select element"));
}

#[test]
fn unsupported_document_images_on_elements_fails_explicitly() {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').images.length;</script>",
    )
    .expect_err("non-document images access should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported member access"));
    assert!(message.contains("`images`"));
    assert!(message.contains("element value"));
}

#[test]
fn unsupported_document_all_on_elements_fails_explicitly() {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-doc'></div></div><script>document.getElementById('not-doc').all.length;</script>",
    )
    .expect_err("non-document all access should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported member access"));
    assert!(message.contains("`all`"));
    assert!(message.contains("element value"));
}

#[test]
fn unsupported_get_elements_by_tag_name_ns_arity_fails_explicitly() {
    let error = Harness::from_html(
        "<div id='root'><svg id='icon'><rect id='rect'></rect></svg></div><script>document.getElementsByTagNameNS('http://www.w3.org/2000/svg');</script>",
    )
    .expect_err("arity mismatch should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("getElementsByTagNameNS() expects exactly two arguments"));
}

#[test]
fn unsupported_element_matches_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root' class='primary'></main><script>document.getElementById('root').matches('main:where([data-kind=primary x y])');</script>",
    )
    .expect_err("broader CSS parsing inside :where should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(
        message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr]")
            && message.contains("optional attribute selector flags like `[attr=value i]` and `[attr=value s]`")
            && message.contains("bounded logical pseudo-classes like `:not(.primary)`")
            && message.contains("state pseudo-classes like `:checked`, `:disabled`, `:enabled`, `:indeterminate`, `:default`, `:valid`, `:invalid`, `:in-range`, and `:out-of-range`")
            && message.contains("form-editable state pseudo-classes also include `:read-only` and `:read-write`")
            && message.contains("descendant combinators like `A B`")
            && message.contains("child combinators like `A > B`")
    );
}

#[test]
fn unsupported_element_closest_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root' class='primary'></main><script>document.getElementById('root').closest('main:where([data-kind=primary x y])');</script>",
    )
    .expect_err("broader CSS parsing inside :where should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(
        message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr]")
            && message.contains("optional attribute selector flags like `[attr=value i]` and `[attr=value s]`")
            && message.contains("bounded logical pseudo-classes like `:not(.primary)`")
            && message.contains("state pseudo-classes like `:checked`, `:disabled`, `:enabled`, `:indeterminate`, `:default`, `:valid`, `:invalid`, `:in-range`, and `:out-of-range`")
            && message.contains("form-editable state pseudo-classes also include `:read-only` and `:read-write`")
            && message.contains("descendant combinators like `A B`")
            && message.contains("child combinators like `A > B`")
    );
}

#[test]
fn unsupported_script_scope_has_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root'><section id='child' class='child'></section></main><script>document.querySelector('main:has(:nth-child(2 of))');</script>",
    )
    .expect_err("malformed nth-child of selector syntax should remain unsupported");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector `main:has(:nth-child(2 of))`"));
}

#[test]
fn unsupported_script_hex_escape_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='foo,bar'></main><script>document.querySelector('#foo\\\\110000 bar');</script>",
    )
    .expect_err("out-of-range hex escape should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector `#foo\\110000 bar`"));
}

#[test]
fn unsupported_script_control_character_hex_escape_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='foo'></main><script>document.querySelector('#foo\\\\0 bar');</script>",
    )
    .expect_err("control-character hex escape should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector `#foo\\0 bar`"));
}

#[test]
fn unsupported_script_root_empty_selector_syntax_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root'><section id='empty'></section></main><script>document.querySelector('#empty:empty()');</script>",
    )
    .expect_err("malformed :empty selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector `#empty:empty()`"));
}

#[test]
fn unsupported_script_only_child_selector_syntax_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root'><section id='child'>child</section></main><script>document.querySelector('#child:only-child()');</script>",
    )
    .expect_err("malformed :only-child selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector `#child:only-child()`"));
}

#[test]
fn unsupported_script_first_of_type_selector_syntax_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root'><section id='child'>child</section></main><script>document.querySelector('#child:first-of-type()');</script>",
    )
    .expect_err("malformed :first-of-type selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector `#child:first-of-type()`"));
}

#[test]
fn unsupported_script_nth_of_type_selector_syntax_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root'><section id='child'>child</section></main><script>document.querySelector('#child:nth-of-type(1 of .child, )');</script>",
    )
    .expect_err("malformed :nth-of-type selector should fail explicitly");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported selector"));
    assert!(message.contains(".child,"));
}
