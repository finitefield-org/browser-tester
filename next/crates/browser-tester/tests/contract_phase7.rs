use browser_tester_next::Harness;

#[test]
fn script_dom_query_selectors_work_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>scope<section><div class='primary'>inside</div></section></main><div id='out'></div><script>const docMatch = document.querySelector('.primary'); const scopedMatch = document.getElementById('root').querySelector('.primary'); const missing = document.getElementById('root').querySelector('.missing'); document.getElementById('out').textContent = docMatch.textContent + ':' + scopedMatch.textContent + ':' + String(missing);</script>",
    )?;

    harness.assert_text("#out", "scopeinside:inside:null")?;
    Ok(())
}

#[test]
fn script_dom_query_selector_all_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>root<section><div class='primary'>inside</div></section></main><div id='out'></div><script>const all = document.querySelectorAll('.primary'); const scoped = document.getElementById('root').querySelectorAll('.primary'); document.getElementById('out').textContent = String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent + ':' + String(all.item(2)) + ':' + String(scoped.length) + ':' + scoped.item(0).textContent;</script>",
    )?;

    harness.assert_text("#out", "2:rootinside:inside:null:1:inside")?;
    Ok(())
}

#[test]
fn script_dom_selector_lists_work_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>root</main><div class='primary'>inside</div><div id='out'></div><script>const first = document.querySelector('.primary, main'); const all = document.querySelectorAll('.primary, main'); document.getElementById('out').textContent = first.textContent + ':' + String(all.length) + ':' + all.item(0).textContent + ':' + all.item(1).textContent;</script>",
    )?;

    harness.assert_text("#out", "root:2:root:inside")?;
    Ok(())
}

#[test]
fn script_selector_escapes_and_selector_lists_handle_literal_punctuation_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app'><button id='foo,bar' class='alpha:beta'>First</button><button id='second' class='secondary'>Second</button><div id='out'></div></main><script>const escapedId = document.querySelector('#foo\\\\,bar'); const escapedClass = document.querySelector('.alpha\\\\:beta'); const list = document.querySelectorAll('#foo\\\\,bar, .secondary'); const isMatch = document.getElementById('root').matches('main:is(#foo\\\\)bar, .app)'); const whereMatch = document.getElementById('second').closest('button:where(#foo\\\\,bar, .secondary)'); document.getElementById('out').textContent = escapedId.textContent + ':' + escapedClass.textContent + ':' + String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + String(isMatch) + ':' + whereMatch.textContent;</script>",
    )?;

    harness.assert_text("#out", "First:First:2:First:Second:true:Second")?;
    Ok(())
}

#[test]
fn script_selector_hex_escapes_work_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app'><button id='foo,bar' class='alpha:beta' data-label='foo]bar'>First</button><button id='second' class='secondary'>Second</button><div id='out'></div></main><script>const escapedId = document.querySelector('#foo\\\\2c bar'); const escapedClass = document.querySelector('.alpha\\\\3a beta'); const escapedAttr = document.querySelector('[data-label=foo\\\\5d bar]'); const list = document.querySelectorAll('#foo\\\\2c bar, .secondary'); const whereMatch = document.getElementById('second').closest('button:where(#foo\\\\2c bar, .secondary)'); document.getElementById('out').textContent = escapedId.textContent + ':' + escapedClass.textContent + ':' + escapedAttr.textContent + ':' + String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + whereMatch.textContent;</script>",
    )?;

    harness.assert_text("#out", "First:First:First:2:First:Second:Second")?;
    Ok(())
}

#[test]
fn script_selector_lists_ignore_commas_inside_quoted_attribute_values_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app'><button id='first' data-label='A,B'>First</button><button id='second' class='secondary'>Second</button></main><div id='out'></div><script>const list = document.querySelectorAll(\"button[data-label='A,B'], .secondary\"); const isMatch = document.getElementById('root').matches(\"main:is([data-label='A,B'], .app)\"); const notMatch = document.getElementById('second').matches(\"button:not([data-label='A,B'], .blocked)\"); const whereMatch = document.getElementById('root').closest(\"main:where([data-label='A,B'], .app)\"); document.getElementById('out').textContent = String(list.length) + ':' + list.item(0).textContent + ':' + list.item(1).textContent + ':' + String(isMatch) + ':' + String(notMatch) + ':' + whereMatch.textContent;</script>",
    )?;

    harness.assert_text("#out", "2:First:Second:true:true:FirstSecond")?;
    Ok(())
}

#[test]
fn script_attribute_value_selectors_work_end_to_end() -> browser_tester_next::Result<()> {
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
fn script_html_collection_children_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span id='first'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const children = document.getElementById('root').children; const before = children.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(children.length) + ':' + String(children.item(0));</script>",
    )?;

    harness.assert_text("#out", "2:0:null")?;
    Ok(())
}

#[test]
fn script_html_collection_named_item_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const children = document.getElementById('root').children; const alpha = children.namedItem('alpha'); const second = children.namedItem('second'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = alpha.textContent + ':' + second.textContent + ':' + String(children.namedItem('alpha'));</script>",
    )?;

    harness.assert_text("#out", "First:Second:null")?;
    Ok(())
}

#[test]
fn script_get_elements_by_tag_name_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span name='alpha'>First</span><span id='second'>Second</span></main><div id='out'></div><script>const all = document.getElementsByTagName('span'); const scoped = document.getElementById('root').getElementsByTagName('span'); const alpha = all.namedItem('alpha'); const second = scoped.namedItem('second'); const before = all.length; const beforeScoped = scoped.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(beforeScoped) + ':' + String(scoped.length) + ':' + alpha.textContent + ':' + second.textContent + ':' + String(all.namedItem('alpha'));</script>",
    )?;

    harness.assert_text("#out", "2:0:2:0:First:Second:null")?;
    Ok(())
}

#[test]
fn script_get_elements_by_class_name_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='alpha'><span name='alpha' class='alpha'>First</span><span id='second' class='alpha'>Second</span></main><div id='out'></div><script>const all = document.getElementsByClassName('alpha'); const scoped = document.getElementById('root').getElementsByClassName('alpha'); const named = all.namedItem('alpha'); const root = all.item(0); const before = all.length; const beforeScoped = scoped.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(beforeScoped) + ':' + String(scoped.length) + ':' + named.textContent + ':' + String(scoped.namedItem('alpha')) + ':' + root.textContent;</script>",
    )?;

    harness.assert_text("#out", "3:1:2:0:First:null:gone")?;
    Ok(())
}

#[test]
fn script_get_elements_by_name_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span name='alpha'>First</span><span name='alpha'>Second</span></main><div id='out'></div><script>const nodes = document.getElementsByName('alpha'); const first = nodes.item(0); const before = nodes.length; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(nodes.length) + ':' + first.textContent + ':' + String(nodes.item(1));</script>",
    )?;

    harness.assert_text("#out", "2:0:First:null")?;
    Ok(())
}

#[test]
fn script_document_forms_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><form id='signup' name='signup'>Signup</form><form id='login' name='login'>Login</form></div><div id='out'></div><script>const forms = document.forms; const first = forms.item(0); const named = forms.namedItem('signup'); const before = forms.length; const firstText = first.textContent; const namedText = named.textContent; document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(forms.length) + ':' + firstText + ':' + namedText + ':' + String(forms.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:0:Signup:Signup:null")?;
    Ok(())
}

#[test]
fn script_form_elements_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><form id='signup'><input name='first' value='Ada'><textarea name='bio'>Bio</textarea></form></div><div id='out'></div><script>const elements = document.getElementById('signup').elements; const first = elements.item(0); const named = elements.namedItem('first'); const before = elements.length; const firstValue = first.value; const namedValue = named.value; document.getElementById('signup').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(elements.length) + ':' + firstValue + ':' + namedValue + ':' + String(elements.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:0:Ada:Ada:null")?;
    Ok(())
}

#[test]
fn script_select_options_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><select id='mode'><option name='alpha' value='a'>A</option><option id='second' value='b'>B</option></select></div><div id='out'></div><script>const options = document.getElementById('mode').options; const first = options.item(0); const named = options.namedItem('second'); const before = options.length; const firstText = first.textContent; const namedText = named.textContent; document.getElementById('mode').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(options.length) + ':' + firstText + ':' + namedText + ':' + String(options.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:0:A:B:null")?;
    Ok(())
}

#[test]
fn script_document_images_and_links_are_live_end_to_end() -> browser_tester_next::Result<()> {
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
fn script_document_all_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><span id='first'>First</span><span id='second'>Second</span></div><div id='out'></div><script>const all = document.all; const before = all.length; const named = all.namedItem('second'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(before) + ':' + String(all.length) + ':' + String(named) + ':' + String(all.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "5:3:[object Element]:null")?;
    Ok(())
}

#[test]
fn script_get_elements_by_tag_name_ns_is_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<div id='root'><svg id='icon'><rect id='rect'></rect><circle id='dot'></circle></svg><math id='formula'><mi id='symbol'>x</mi></math><span id='label'>Label</span></div><div id='out'></div><script>const svgAll = document.getElementsByTagNameNS('http://www.w3.org/2000/svg', '*'); const svgRect = document.getElementById('icon').getElementsByTagNameNS('http://www.w3.org/2000/svg', 'rect'); const htmlSpan = document.getElementsByTagNameNS('http://www.w3.org/1999/xhtml', 'span'); const mathAll = document.getElementsByTagNameNS('http://www.w3.org/1998/Math/MathML', '*'); const beforeSvgAll = svgAll.length; const beforeSvgRect = svgRect.length; const beforeHtmlSpan = htmlSpan.length; const beforeMathAll = mathAll.length; const dot = svgAll.namedItem('dot'); document.getElementById('root').textContent = 'gone'; document.getElementById('out').textContent = String(beforeSvgAll) + ':' + String(svgAll.length) + ':' + String(beforeSvgRect) + ':' + String(svgRect.length) + ':' + String(beforeHtmlSpan) + ':' + String(htmlSpan.length) + ':' + String(beforeMathAll) + ':' + String(mathAll.length) + ':' + String(dot) + ':' + String(svgAll.namedItem('dot'));</script>",
    )?;

    harness.assert_text("#out", "3:0:1:1:1:0:2:0:[object Element]:null")?;
    Ok(())
}

#[test]
fn script_simple_pseudo_classes_work_end_to_end() -> browser_tester_next::Result<()> {
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
fn script_root_and_empty_pseudo_classes_work_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><section id='empty-comment'><!-- gap --></section><section id='empty'></section><section id='non-empty'>content</section><div id='out'>seed</div></main><script>const root = document.querySelector(':root'); const empties = document.querySelectorAll('#root :empty'); const emptyComment = document.getElementById('empty-comment'); const nonEmpty = document.getElementById('non-empty'); document.getElementById('out').textContent = String(root.matches(':root')) + ':' + String(empties.length) + ':' + empties.item(0).matches(':empty') + ':' + empties.item(1).matches(':empty') + ':' + String(emptyComment.matches(':empty')) + ':' + String(nonEmpty.matches(':empty'));</script>",
    )?;

    harness.assert_text("#out", "true:2:true:true:true:false")?;
    Ok(())
}

#[test]
fn script_only_child_and_only_of_type_pseudo_classes_work_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'>lead<!-- gap --><div id='single-child-parent'>text<!-- marker --><section id='only-child'>child</section><!-- marker --></div><div id='type-parent'><span id='first-span'>one</span><em id='only-of-type'>type</em><span id='second-span'>two</span></div><div id='out'>seed</div><script>const onlyChild = document.querySelector('#only-child:only-child'); const onlyOfType = document.querySelector('#only-of-type:only-of-type'); const onlyChildMatches = document.querySelectorAll('#single-child-parent > :only-child'); const onlyOfTypeMatches = document.querySelectorAll('#type-parent > :only-of-type'); const firstSpan = document.getElementById('first-span'); const firstSpanNotOnlyChild = firstSpan.matches('#first-span:not(:only-child)'); const firstSpanNotOnlyOfType = firstSpan.matches('#first-span:not(:only-of-type)'); const parent = onlyChild.closest('#single-child-parent'); document.getElementById('out').textContent = onlyChild.textContent + ':' + onlyOfType.textContent + ':' + String(onlyChildMatches.length) + ':' + String(onlyOfTypeMatches.length) + ':' + String(firstSpanNotOnlyChild) + ':' + String(firstSpanNotOnlyOfType) + ':' + String(parent.matches('#single-child-parent'));</script></main>",
    )?;

    harness.assert_text("#out", "child:type:1:1:true:true:true")?;
    Ok(())
}

#[test]
fn script_first_last_and_nth_of_type_pseudo_classes_work_end_to_end()
-> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><div id='type-parent'><span id='first-span'>one</span><em id='first-em'>first</em><span id='middle-span'>two</span><em id='last-em'>last</em><span id='last-span'>three</span></div><div id='out'>seed</div><script>const firstSpan = document.querySelector('#first-span:first-of-type'); const lastSpan = document.querySelector('#last-span:last-of-type'); const middleSpan = document.querySelector('#middle-span:nth-of-type(2)'); const middleFromEnd = document.querySelector('#middle-span:nth-last-of-type(2)'); const firstEm = document.querySelector('#first-em:first-of-type'); const lastEm = document.querySelector('#last-em:last-of-type'); document.getElementById('out').textContent = String(firstSpan.matches('#first-span:first-of-type')) + ':' + String(lastSpan.matches('#last-span:last-of-type')) + ':' + String(middleSpan.matches('#middle-span:nth-of-type(2)')) + ':' + String(middleFromEnd.matches('#middle-span:nth-last-of-type(2)')) + ':' + String(firstEm.matches('#first-em:first-of-type')) + ':' + String(lastEm.matches('#last-em:last-of-type'));</script></main>",
    )?;

    harness.assert_text("#out", "true:true:true:true:true:true")?;
    Ok(())
}

#[test]
fn script_not_pseudo_class_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app' data-kind='app'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button><div id='out'></div><script>const enabled = document.querySelectorAll('button:not(:disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:not([data-kind*=blocked], .blocked)'); const bounded = document.querySelectorAll('button:not(main > .secondary, :disabled)'); document.getElementById('out').textContent = String(enabled.length) + ':' + enabled.item(0).textContent + ':' + enabled.item(1).textContent + ':' + String(second.matches('button:not(.primary)')) + ':' + String(root.matches('main:not([data-kind*=blocked], .blocked)')) + ':' + document.querySelector('button:not(:nth-child(even))').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script></main>",
    )?;

    harness.assert_text("#out", "2:First:Enabled:true:true:First:1:First")?;
    Ok(())
}

#[test]
fn script_is_pseudo_class_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app' data-kind='app'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const all = document.querySelectorAll('button:is(.primary, .secondary)'); const filtered = document.querySelectorAll('button:is(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:is(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:is([data-kind^=ap], .blocked)'); document.getElementById('out').textContent = String(all.length) + ':' + String(filtered.length) + ':' + String(second.matches('button:is(.secondary, .blocked)')) + ':' + String(root.matches('main:is([data-kind^=ap], .blocked)')) + ':' + document.querySelector('button:is(.primary, .secondary):not(:disabled)').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script>",
    )?;

    harness.assert_text("#out", "3:2:true:true:First:2:Disabled")?;
    Ok(())
}

#[test]
fn script_where_pseudo_class_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='app' data-kind='APP READY' lang='EN-US'><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='secondary'>Enabled</button></main><div id='out'></div><script>const all = document.querySelectorAll('button:where(.primary, .secondary)'); const filtered = document.querySelectorAll('button:where(:disabled, .secondary)'); const bounded = document.querySelectorAll('button:where(main > .secondary, :disabled)'); const second = document.getElementById('enabled'); const root = second.closest('main:where([lang|=en i], .blocked)'); document.getElementById('out').textContent = String(all.length) + ':' + String(filtered.length) + ':' + String(second.matches('button:where(.secondary, .blocked)')) + ':' + String(root.matches('main:where([lang|=en i], .blocked)')) + ':' + document.querySelector('button:where(.primary, .secondary):not(:disabled)').textContent + ':' + String(bounded.length) + ':' + bounded.item(0).textContent;</script>",
    )?;

    harness.assert_text("#out", "3:2:true:true:First:2:Disabled")?;
    Ok(())
}

#[test]
fn script_general_sibling_selectors_work_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main><button id='first' class='primary'>First</button>text<button id='second' class='primary'>Second</button><div id='out'></div><script>const sibling = document.querySelector('#first ~ .primary'); const second = document.getElementById('second'); document.getElementById('out').textContent = sibling.textContent + ':' + String(second.matches('#first ~ .primary'));</script></main>",
    )?;

    harness.assert_text("#out", "Second:true")?;
    Ok(())
}

#[test]
fn script_element_matches_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'><section><div id='child' class='child'></div></section></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = String(root.matches('.primary')) + ':' + String(root.matches('.child')) + ':' + String(child.matches('.child'));</script>",
    )?;

    harness.assert_text("#out", "true:false:true")?;
    Ok(())
}

#[test]
fn script_element_closest_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root' class='primary'>ROOT<section id='section'>SECTION<div id='child' class='child'>CHILD</div></section></main><div id='out'></div><script>const root = document.getElementById('root'); const child = document.getElementById('child'); document.getElementById('out').textContent = root.closest('.primary').textContent + ':' + child.closest('.child').textContent + ':' + child.closest('#section').textContent + ':' + String(child.closest('.missing'));</script>",
    )?;

    harness.assert_text("#out", "ROOTSECTIONCHILD:CHILD:SECTIONCHILD:null")?;
    Ok(())
}

#[test]
fn script_node_list_for_each_works_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><span class='item'>First</span><span class='item'>Second</span></main><div id='out'></div><script>const nodes = document.querySelectorAll('.item'); nodes.forEach((item, index, list) => { document.getElementById('out').textContent += String(index) + ':' + item.textContent + ':' + String(list.length) + ';'; }, null);</script>",
    )?;

    harness.assert_text("#out", "0:First:2;1:Second:2;")?;
    Ok(())
}

#[test]
fn script_html_collection_for_each_works_end_to_end() -> browser_tester_next::Result<()> {
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
        "<main id='root' class='primary'></main><script>document.getElementById('root').matches('main:where([data-kind=primary x])');</script>",
    )
    .expect_err("broader CSS parsing inside :where should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`"));
}

#[test]
fn unsupported_element_closest_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root' class='primary'></main><script>document.getElementById('root').closest('main:where([data-kind=primary x])');</script>",
    )
    .expect_err("broader CSS parsing inside :where should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], [attr=value], [attr^=value], [attr$=value], [attr*=value], [attr~=value], [attr|=value], optional attribute selector flags like `[attr=value i]` and `[attr=value s]`, bounded logical pseudo-classes like `:not(.primary)`, `:is(.primary, .secondary)`, and `:where(.primary, .secondary)`, structural pseudo-classes like `:first-child`, `:last-child`, `:nth-child(2)`, `:nth-child(odd)`, `:nth-child(2n+1)`, and `:nth-last-child(2)`, state pseudo-classes like `:checked`, `:disabled`, and `:enabled`, descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`"));
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
