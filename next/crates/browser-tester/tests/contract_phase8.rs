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
        "<main id='root'><button id='button' class='base' data-kind='App'>First</button><div id='out'></div><script>const button = document.getElementById('button'); button.className = 'primary secondary'; const before = button.classList.length; const contains = button.classList.contains('primary'); button.classList.add('tertiary'); button.classList.remove('secondary'); const toggled = button.classList.toggle('active'); button.dataset.userId = '42'; document.getElementById('out').textContent = button.className + ':' + String(before) + ':' + String(contains) + ':' + String(toggled) + ':' + button.dataset.kind + ':' + button.dataset.userId + ':' + String(button.classList) + ':' + String(button.dataset);</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "primary tertiary active:2:true:true:App:42:[object DOMTokenList]:[object DOMStringMap]",
    )?;
    harness.assert_exists(".active")?;
    harness.assert_exists("[data-user-id]")?;
    harness.assert_exists("[data-kind=App]")?;
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
fn document_scripts_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><script id='first-script'></script></main><div id='out'></div><script>const out = document.getElementById('out'); const scripts = document.scripts; const before = scripts.length; const first = scripts.namedItem('first-script'); document.getElementById('root').textContent = 'gone'; out.textContent = String(before) + ':' + String(scripts.length) + ':' + String(first) + ':' + String(scripts.namedItem('missing'));</script>",
    )?;

    harness.assert_text("#out", "2:1:[object Element]:null")?;
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
