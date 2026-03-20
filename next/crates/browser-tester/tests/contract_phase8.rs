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
        "<!--pre--><main id='root'>Hello<span>World</span><!--tail--></main><div id='out'></div><script>const docNodes = document.childNodes; const rootNodes = document.getElementById('root').childNodes; const docFirst = docNodes.item(0); const docSecond = docNodes.item(1); const rootValues = rootNodes.values(); const firstRoot = rootValues.next(); const secondRoot = rootValues.next(); const thirdRoot = rootValues.next(); document.getElementById('out').textContent = String(docNodes.length) + ':' + docFirst.nodeName + ':' + String(docFirst.nodeType) + ':' + String(docFirst) + ':' + docSecond.nodeName + ':' + String(docSecond.nodeType) + ':' + firstRoot.value.nodeName + ':' + String(firstRoot.value.nodeType) + ':' + firstRoot.value.textContent + ':' + secondRoot.value.nodeName + ':' + String(secondRoot.value.nodeType) + ':' + secondRoot.value.textContent + ':' + thirdRoot.value.nodeName + ':' + String(thirdRoot.value.nodeType) + ':' + thirdRoot.value.textContent;</script>",
    )?;

    harness.assert_text(
        "#out",
        "4:#comment:8:[object Node]:main:1:#text:3:Hello:span:1:World:#comment:8:",
    )?;
    Ok(())
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
fn document_style_sheets_are_not_available_on_elements_end_to_end() -> browser_tester_next::Result<()>
{
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
fn element_labels_are_live_end_to_end() -> browser_tester_next::Result<()> {
    let harness = Harness::from_html(
        "<main id='root'><label id='explicit-label' for='control'>Explicit</label><input id='control' value='A'><label id='implicit-label'><input id='inner-control' value='B'>Implicit</label><div id='wrapper'></div><div id='out'></div><script>const control = document.getElementById('control'); const labels = control.labels; const inner = document.getElementById('inner-control').labels; const before = labels.length; document.getElementById('wrapper').innerHTML = '<label id=\"second-label\" for=\"control\">Second</label>'; document.getElementById('out').textContent = String(before) + ':' + String(labels.length) + ':' + labels.item(0).getAttribute('id') + ':' + labels.item(1).textContent + ':' + String(inner.length) + ':' + inner.item(0).getAttribute('id');</script></main>",
    )?;

    harness.assert_text(
        "#out",
        "1:2:explicit-label:Second:1:implicit-label",
    )?;
    harness.assert_exists("#second-label")?;
    Ok(())
}

#[test]
fn fieldset_elements_and_datalist_options_are_live_end_to_end() -> browser_tester_next::Result<()>
{
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
fn labels_reject_non_labelable_elements_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-labelable'></div></div><script>document.getElementById('not-labelable').labels.length;</script>",
    )
    .expect_err("non-labelable labels access should fail");

    assert!(error.to_string().contains("node is not a labelable element"));
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
fn map_areas_reject_non_map_elements_end_to_end() -> browser_tester_next::Result<()> {
    let error = Harness::from_html(
        "<div id='wrapper'><div id='not-map'></div></div><script>document.getElementById('not-map').areas.length;</script>",
    )
    .expect_err("non-map areas access should fail");

    assert!(error.to_string().contains("map.areas"));
    assert!(error.to_string().contains("supported map.areas host element"));
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
