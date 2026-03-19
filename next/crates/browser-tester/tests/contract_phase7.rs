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
fn script_simple_pseudo_classes_work_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main><button id='first' class='primary'>First</button><button id='disabled' class='primary' disabled>Disabled</button><button id='enabled' class='primary'>Enabled</button><input id='agree' type='checkbox' checked><select id='mode'><option value='a'>A</option><option id='selected' value='b' selected>B</option></select><div id='out'></div><script>const first = document.querySelector('#first:first-child'); const disabled = document.querySelector('button:disabled'); const enabled = document.querySelectorAll('button:enabled'); const checked = document.querySelector('input:checked'); const selected = document.querySelector('option:checked'); document.getElementById('out').textContent = first.textContent + ':' + disabled.textContent + ':' + String(enabled.length) + ':' + checked.checked + ':' + selected.textContent;</script></main>",
    )?;

    harness.assert_text("#out", "First:Disabled:2:true:B")?;
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
fn unsupported_node_list_methods_fail_explicitly() {
    let error = Harness::from_html(
        "<main id='out'></main><script>document.querySelectorAll('#out').forEach(() => {});</script>",
    )
    .expect_err("unsupported selector methods should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported NodeList method: forEach"));
}

#[test]
fn unsupported_html_collection_methods_fail_explicitly() {
    let error = Harness::from_html(
        "<main id='root'><span>child</span></main><script>document.getElementById('root').children.forEach(() => {});</script>",
    )
    .expect_err("unsupported html collection methods should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("unsupported HTMLCollection method: forEach"));
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
fn unsupported_element_matches_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root' class='primary'></main><script>document.getElementById('root').matches('main:nth-child(2)');</script>",
    )
    .expect_err("unsupported selector syntax should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`"));
}

#[test]
fn unsupported_element_closest_selector_fails_explicitly() {
    let error = Harness::from_html(
        "<main id='root' class='primary'></main><script>document.getElementById('root').closest('main:nth-child(2)');</script>",
    )
    .expect_err("unsupported selector syntax should fail");

    let message = error.to_string();
    assert!(message.contains("Script error"));
    assert!(message.contains("supported forms are #id, .class, tag, tag.class, #id.class, [attr], descendant combinators like `A B`, adjacent sibling combinators like `A + B`, general sibling combinators like `A ~ B`, and child combinators like `A > B`"));
}
