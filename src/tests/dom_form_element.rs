use super::*;

#[test]
fn form_implicit_role_and_role_assignment_roundtrip() -> Result<()> {
    let html = r#"
        <form id='target' name='signup'>
          <label for='email'>Email</label>
          <input id='email' name='email' type='email' required>
        </form>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('target');
            const initial = form.role + ':' + form.tagName + ':' + form.getAttribute('name');
            form.role = 'search';
            const assigned = form.role + ':' + form.getAttribute('role');
            form.removeAttribute('role');
            const restored = form.role + ':' + (form.getAttribute('role') === null);
            document.getElementById('result').textContent =
              initial + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "form:FORM:signup|search:search|form:true")?;
    Ok(())
}

#[test]
fn form_submission_attributes_and_request_submit_work() -> Result<()> {
    let html = r#"
        <form id='target' action='/subscribe' method='get' target='_blank' autocomplete='on' accept-charset='UTF-8' rel='search'>
          <input id='email' name='email' type='email' required value='seed@example.com'>
          <button id='submitter' type='submit'>Subscribe</button>
        </form>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('target');
            const submitter = document.getElementById('submitter');
            let submits = 0;
            form.addEventListener('submit', (event) => {
              submits++;
              event.preventDefault();
            });

            const before =
              form.getAttribute('action') + ':' +
              form.getAttribute('method') + ':' +
              form.getAttribute('target') + ':' +
              form.getAttribute('autocomplete') + ':' +
              form.getAttribute('accept-charset') + ':' +
              form.getAttribute('rel') + ':' +
              form.hasAttribute('novalidate');

            form.setAttribute('method', 'post');
            form.setAttribute('enctype', 'multipart/form-data');
            form.setAttribute('target', '_self');
            form.setAttribute('novalidate', '');
            form.setAttribute('name', 'newsletter');

            const afterAttrs =
              form.getAttribute('method') + ':' +
              form.getAttribute('enctype') + ':' +
              form.getAttribute('target') + ':' +
              form.hasAttribute('novalidate') + ':' +
              form.getAttribute('name');

            form.requestSubmit(submitter);

            const formData = new FormData(form);
            const afterSubmit =
              submits + ':' +
              formData.get('email') + ':' +
              form.elements.length;

            document.getElementById('result').textContent =
              before + '|' + afterAttrs + '|' + afterSubmit;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "/subscribe:get:_blank:on:UTF-8:search:false|post:multipart/form-data:_self:true:newsletter|1:seed@example.com:2",
    )?;
    Ok(())
}

#[test]
fn form_elements_is_live_cached_and_specialized_collection_surface_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='email' name='email' value='seed@example.com'>
          <button id='submitter' name='send' type='submit'>Send</button>
        </form>
        <input id='external' form='target' name='outside' value='extra'>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('target');
            const elements = form.elements;
            const same = form.elements;

            const later = document.createElement('input');
            later.id = 'later';
            later.name = 'later';
            form.appendChild(later);

            document.getElementById('result').textContent = [
              String(same === elements),
              Object.prototype.toString.call(elements),
              elements.constructor.name,
              String(Object.getPrototypeOf(elements) === HTMLFormControlsCollection.prototype),
              elements.namedItem('email').id,
              elements['outside'].id,
              elements.namedItem('later').id,
              String(elements.length),
              String(form.elements === elements)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:[object HTMLFormControlsCollection]:HTMLFormControlsCollection:true:email:external:later:4:true",
    )?;
    Ok(())
}

#[test]
fn form_elements_define_property_delete_and_shadow_parity_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='email' name='email' value='seed@example.com'>
          <button id='submitter' name='send' type='submit'>Send</button>
        </form>
        <p id='result'></p>
        <script>
          const elements = document.getElementById('target').elements;

          const returnedZero = Object.defineProperty(elements, '0', { value: 'shadow-zero' });
          const returnedLength = Object.defineProperty(elements, 'length', { value: 99 });
          const returnedEmail = Object.defineProperty(elements, 'email', { value: 'shadow-email' });

          const zeroDesc = Object.getOwnPropertyDescriptor(elements, '0');
          const lengthDesc = Object.getOwnPropertyDescriptor(elements, 'length');
          const emailDesc = Object.getOwnPropertyDescriptor(elements, 'email');

          const before = [
            String(returnedZero === elements),
            String(returnedLength === elements),
            String(returnedEmail === elements),
            elements[0],
            elements.length,
            elements.email,
            String(zeroDesc.enumerable),
            String(zeroDesc.configurable),
            String(zeroDesc.writable),
            String(lengthDesc.enumerable),
            String(lengthDesc.configurable),
            String(lengthDesc.writable),
            String(emailDesc.enumerable),
            String(emailDesc.configurable),
            String(emailDesc.writable)
          ].join(':');

          const deleted = [
            String(delete elements[0]),
            String(delete elements.length),
            String(delete elements.email)
          ].join(':');

          const after = [
            elements[0].id,
            String(elements.length),
            elements.email.id,
            elements.namedItem('email').id
          ].join(':');

          document.getElementById('result').textContent = [before, deleted, after].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true:true:true:shadow-zero:99:shadow-email:true:true:false:false:true:false:true:true:false|true:true:true|email:2:email:email",
    )?;
    Ok(())
}

#[test]
fn form_elements_named_property_collisions_keep_builtin_surface_visible_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='named-item-id' name='namedItem' value='a'>
          <input id='item-id' name='item' value='b'>
          <input id='length-id' name='length' value='c'>
          <input id='ctor-id' name='constructor' value='d'>
          <input id='values-id' name='values' value='e'>
        </form>
        <p id='result'></p>
        <script>
          const elements = document.getElementById('target').elements;
          const keys = Reflect.ownKeys(elements);
          document.getElementById('result').textContent = [
            typeof elements.item,
            typeof elements.namedItem,
            elements.namedItem('namedItem').id,
            elements.namedItem('item').id,
            elements.namedItem('length').id,
            elements.namedItem('constructor').id,
            elements.namedItem('values').id,
            String(keys.includes('item')),
            String(keys.includes('namedItem')),
            String(keys.includes('length')),
            String(keys.includes('constructor')),
            String(keys.includes('values')),
            elements.constructor.name
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "function:function:named-item-id:item-id:length-id:ctor-id:values-id:false:false:true:false:false:HTMLFormControlsCollection",
    )?;
    Ok(())
}

#[test]
fn form_elements_multi_match_named_lookup_returns_live_radio_node_lists_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='r1' type='radio' name='pick' value='a' checked>
          <input id='r2' type='radio' name='pick' value='b'>
          <input id='t1' name='dup' value='x'>
          <input id='t2' name='dup' value='y'>
        </form>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('target');
            const radioGroup = form.elements['pick'];
            const sameRadioGroup = form.elements.namedItem('pick');
            const textGroup = form.elements['dup'];
            const sameTextGroup = form.elements.namedItem('dup');
            const lengthKey = 'length';
            const valueKey = 'value';

            let illegal = false;
            try {
              RadioNodeList();
            } catch (error) {
              illegal = String(error).includes('Illegal constructor');
            }

            const initialRadio = [
              typeof RadioNodeList,
              String(window.RadioNodeList === RadioNodeList),
              String(radioGroup === sameRadioGroup),
              Object.prototype.toString.call(radioGroup),
              String(radioGroup !== null),
              String(Object.getPrototypeOf(radioGroup) === RadioNodeList.prototype),
              String(RadioNodeList.prototype.constructor === RadioNodeList),
              String(Object.getPrototypeOf(RadioNodeList.prototype) === NodeList.prototype),
              String(radioGroup[lengthKey]),
              radioGroup[valueKey],
              String(illegal)
            ].join(':');

            radioGroup[valueKey] = 'b';
            const afterSet = [
              radioGroup[valueKey],
              String(document.getElementById('r1').checked),
              String(document.getElementById('r2').checked)
            ].join(':');

            const initialText = [
              String(textGroup === sameTextGroup),
              Object.prototype.toString.call(textGroup),
              String(textGroup !== null),
              String(textGroup[lengthKey]),
            ].join(':');

            const later = document.createElement('input');
            later.id = 't3';
            later.name = 'dup';
            later.value = 'z';
            form.appendChild(later);

            const afterAppend = [
              String(form.elements['dup'] === textGroup),
              String(form.elements.namedItem('dup') === textGroup),
              String(textGroup[lengthKey])
            ].join(':');

            document.getElementById('result').textContent =
              [initialRadio, afterSet, initialText, afterAppend].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "function:true:true:[object RadioNodeList]:true:true:true:true:2:a:true|b:false:true|true:[object RadioNodeList]:true:2|true:true:3",
    )?;
    Ok(())
}

#[test]
fn radio_node_list_reflective_surface_and_object_copy_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='r1' type='radio' name='pick' value='a' checked>
          <input id='r2' type='radio' name='pick' value='b'>
        </form>
        <p id='result'></p>
        <script>
          const group = document.getElementById('target').elements['pick'];
          const valueKey = 'value';
          const protoDesc = Object.getOwnPropertyDescriptor(RadioNodeList.prototype, valueKey);
          const assigned = Object.assign({}, group);
          const spread = { ...group };
          const getter = protoDesc.get;
          const setter = protoDesc.set;

          document.getElementById('result').textContent = [
            Object.keys(group).join(','),
            Object.getOwnPropertyNames(group).join(','),
            Reflect.ownKeys(group).join(','),
            String(Object.getOwnPropertyDescriptor(group, valueKey) === undefined),
            String(Object.getOwnPropertyNames(RadioNodeList.prototype).includes(valueKey)),
            String(typeof getter === 'function'),
            String(typeof setter === 'function'),
            String(protoDesc.enumerable === false),
            String(protoDesc.configurable === true),
            String(Function.prototype.call.call(getter, group) === 'a'),
            String(Function.prototype.call.call(setter, group, 'b') === undefined),
            group[valueKey],
            String(document.getElementById('r1').checked) + ':' + String(document.getElementById('r2').checked),
            group.item(1).id,
            assigned[0].id,
            assigned[1].id,
            Object.keys(assigned).join(','),
            spread[0].id,
            spread[1].id,
            Object.keys(spread).join(',')
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "0,1|0,1,length|0,1,length|true|true|true|true|true|true|true|true|b|false:true|r2|r1|r2|0,1|r1|r2|0,1",
    )?;
    Ok(())
}

#[test]
fn radio_node_list_shadow_delete_and_explicit_prototype_override_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='r1' type='radio' name='pick' value='a' checked>
          <input id='r2' type='radio' name='pick' value='b'>
        </form>
        <p id='result'></p>
        <script>
          const group = document.getElementById('target').elements['pick'];
          const valueKey = 'value';

          Object.defineProperty(group, valueKey, {
            value: 'shadow',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(group, '0', {
            value: 'shadow-zero',
            enumerable: true,
            configurable: true
          });

          const shadow = [
            group[valueKey],
            group[0],
            Object.keys(group).join(','),
            String(document.getElementById('r1').checked) + ':' + String(document.getElementById('r2').checked)
          ].join('|');

          delete group[valueKey];
          delete group[0];

          const afterDelete = [
            group[valueKey],
            group[0].id,
            String(Object.getOwnPropertyDescriptor(group, valueKey) === undefined),
            String(Object.getOwnPropertyDescriptor(group, '0').value.id === 'r1')
          ].join('|');

          const customProto = { value: 'proto', marker: 'm' };
          Object.setPrototypeOf(group, customProto);

          const afterProto = [
            group[valueKey],
            String(valueKey in group),
            group.marker
          ].join('|');

          group[valueKey] = 'own';

          const afterSet = [
            group[valueKey],
            String(document.getElementById('r1').checked) + ':' + String(document.getElementById('r2').checked),
            Object.keys(group).join(',')
          ].join('|');

          delete group[valueKey];

          const afterOwnDelete = [
            group[valueKey],
            String(Object.getPrototypeOf(group) === customProto),
            String(valueKey in group)
          ].join('|');

          document.getElementById('result').textContent =
            [shadow, afterDelete, afterProto, afterSet, afterOwnDelete].join('||');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "shadow|shadow-zero|0,1,value|true:false||a|r1|true|true||proto|true|m||own|true:false|0,1,value||proto|true|true",
    )?;
    Ok(())
}

#[test]
fn grouped_form_control_named_collisions_keep_builtin_surface_visible_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='item-a' type='radio' name='item' value='a' checked>
          <input id='item-b' type='radio' name='item' value='b'>
          <input id='named-a' name='namedItem' value='x'>
          <input id='named-b' name='namedItem' value='y'>
          <input id='ctor-a' name='constructor' value='m'>
          <input id='ctor-b' name='constructor' value='n'>
          <input id='values-a' name='values' value='p'>
          <input id='values-b' name='values' value='q'>
        </form>
        <p id='result'></p>
        <script>
          const elements = document.getElementById('target').elements;
          const itemGroup = elements.namedItem('item');
          const namedItemGroup = elements.namedItem('namedItem');
          const ctorGroup = elements.namedItem('constructor');
          const valuesGroup = elements.namedItem('values');
          const keys = Reflect.ownKeys(elements);

          document.getElementById('result').textContent = [
            typeof elements['item'],
            typeof elements['namedItem'],
            typeof elements['values'],
            String(elements['constructor'] === HTMLFormControlsCollection),
            Object.prototype.toString.call(itemGroup),
            itemGroup['value'],
            itemGroup['item'](1).id,
            Object.prototype.toString.call(namedItemGroup),
            namedItemGroup['item'](1).id,
            Object.prototype.toString.call(ctorGroup),
            ctorGroup['item'](1).id,
            Object.prototype.toString.call(valuesGroup),
            valuesGroup['item'](1).id,
            String(itemGroup['constructor'] === RadioNodeList),
            String(valuesGroup['constructor'] === RadioNodeList),
            String(keys.includes('item')),
            String(keys.includes('namedItem')),
            String(keys.includes('values')),
            String(keys.includes('constructor'))
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "function:function:function:true:[object RadioNodeList]:a:item-b:[object RadioNodeList]:named-b:[object RadioNodeList]:ctor-b:[object RadioNodeList]:values-b:true:true:false:false:false:false",
    )?;
    Ok(())
}

#[test]
fn radio_node_list_item_iterator_and_mixed_match_order_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='pick' name='other' value='id-only'>
          <input id='r1' type='radio' name='pick' value='a'>
          <input id='r2' type='radio' name='pick' value='b' checked>
          <input id='t1' name='pick' value='text'>
        </form>
        <p id='result'></p>
        <script>
          const group = document.getElementById('target').elements.namedItem('pick');
          const item = group['item'];
          const values = group['values'];
          const iter = values.call(group);
          const valueKey = 'value';
          const before = [
            String(group['length']),
            item.call(group, 0).id,
            item.call(group, 1).id,
            item.call(group, 2).id,
            item.call(group, 3).id,
            iter.next().value.id,
            iter.next().value.id,
            iter.next().value.id,
            iter.next().value.id,
            group[valueKey],
            RadioNodeList.prototype.item.call(group, 2).id,
            String(RadioNodeList.prototype.namedItem === undefined),
            Object.prototype.toString.call(group)
          ].join(':');

          group[valueKey] = 'a';

          const after = [
            group[valueKey],
            String(document.getElementById('r1').checked),
            String(document.getElementById('r2').checked)
          ].join(':');

          document.getElementById('result').textContent = [before, after].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "4:pick:r1:r2:t1:pick:r1:r2:t1:b:r2:true:[object RadioNodeList]|a:true:false",
    )?;
    Ok(())
}

#[test]
fn form_direct_named_property_multi_match_and_live_updates_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='r1' type='radio' name='pick' value='a' checked>
          <input id='r2' type='radio' name='pick' value='b'>
          <input id='t1' name='dup' value='x'>
          <input id='t2' name='dup' value='y'>
          <input id='solo' name='solo' value='z'>
        </form>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const form = document.getElementById('target');
            const radioDot = form.pick;
            const radioBracket = form['pick'];
            const radioNamed = form.elements.namedItem('pick');
            const textDot = form.dup;
            const textBracket = form['dup'];
            const textNamed = form.elements.namedItem('dup');
            const soloDot = form.solo;
            const soloBracket = form['solo'];
            const soloNamed = form.elements.namedItem('solo');
            const valueKey = 'value';
            const lengthKey = 'length';

            const initial = [
              Object.prototype.toString.call(radioDot),
              String(radioDot === radioBracket),
              String(radioDot === radioNamed),
              String(radioDot[lengthKey]),
              radioDot[valueKey],
              Object.prototype.toString.call(textDot),
              String(textDot === textBracket),
              String(textDot === textNamed),
              String(textDot[lengthKey]),
              soloDot.id,
              String(soloDot === soloBracket),
              String(soloDot === soloNamed)
            ].join(':');

            radioDot[valueKey] = 'b';

            const afterSet = [
              radioDot[valueKey],
              String(document.getElementById('r1').checked),
              String(document.getElementById('r2').checked)
            ].join(':');

            const later = document.createElement('input');
            later.id = 't3';
            later.name = 'dup';
            later.value = 'later';
            form.appendChild(later);

            const afterAppend = [
              String(form.dup === textDot),
              String(form['dup'] === textDot),
              String(textDot[lengthKey])
            ].join(':');

            document.getElementById('result').textContent =
              [initial, afterSet, afterAppend].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "[object RadioNodeList]:true:true:2:a:[object RadioNodeList]:true:true:2:solo:true:true|b:false:true|true:true:3",
    )?;
    Ok(())
}

#[test]
fn form_direct_named_property_shadow_delete_and_builtin_collision_work() -> Result<()> {
    let html = r#"
        <form id='target' name='signup'>
          <input id='named-elements' name='elements' value='ignore-elements'>
          <input id='named-length' name='length' value='ignore-length'>
          <input id='named-name' name='name' value='ignore-name'>
          <input id='named-pick-a' type='radio' name='pick' value='a' checked>
          <input id='named-pick-b' type='radio' name='pick' value='b'>
        </form>
        <p id='result'></p>
        <script>
          const form = document.getElementById('target');
          const valueKey = 'value';
          const before = [
            Object.prototype.toString.call(form.elements),
            String(form.length),
            form.name,
            Object.prototype.toString.call(form.pick),
            form['pick'][valueKey]
          ].join(':');

          form.pick = 'shadow';

          const shadow = [
            form.pick,
            String(delete form.pick),
            Object.prototype.toString.call(form.pick),
            form['pick'][valueKey]
          ].join(':');

          document.getElementById('result').textContent = [before, shadow].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "[object HTMLFormControlsCollection]:5:signup:[object RadioNodeList]:a|shadow:true:[object RadioNodeList]:a",
    )?;
    Ok(())
}

#[test]
fn form_reflective_own_property_surface_tracks_named_properties_and_shadow_delete_work()
-> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='r1' type='radio' name='pick' value='a' checked>
          <input id='r2' type='radio' name='pick' value='b'>
        </form>
        <p id='result'></p>
        <script>
          const form = document.getElementById('target');
          const beforeDesc = Object.getOwnPropertyDescriptor(form, 'pick');
          const before = [
            String(Object.hasOwn(form, 'pick')),
            Object.prototype.toString.call(beforeDesc.value),
            String(beforeDesc.writable === false),
            String(beforeDesc.enumerable === false),
            String(beforeDesc.configurable === true),
            String(Object.getOwnPropertyNames(form).includes('pick')),
            String(Reflect.ownKeys(form).includes('pick')),
            String(Object.keys(form).includes('pick')),
            String(Object.hasOwn(form, 'length')),
            String(typeof Object.getOwnPropertyDescriptor(form, 'submit').value),
            String(Object.getOwnPropertyNames(form).includes('submit')),
            String(Reflect.ownKeys(form).includes('submit'))
          ].join(':');

          form['pick'] = 'shadow';
          const shadowDesc = Object.getOwnPropertyDescriptor(form, 'pick');
          const shadow = [
            form['pick'],
            String(shadowDesc.writable === true),
            String(shadowDesc.enumerable === true),
            String(shadowDesc.configurable === true),
            String(delete form['pick'])
          ].join(':');

          const afterDesc = Object.getOwnPropertyDescriptor(form, 'pick');
          const after = [
            Object.prototype.toString.call(form['pick']),
            form['pick'].value,
            String(afterDesc.writable === false),
            String(afterDesc.enumerable === false),
            String(afterDesc.configurable === true)
          ].join(':');

          document.getElementById('result').textContent = [before, shadow, after].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true:[object RadioNodeList]:true:true:true:true:true:false:true:function:true:true|shadow:true:true:true:true|[object RadioNodeList]:a:true:true:true",
    )?;
    Ok(())
}

#[test]
fn form_method_collisions_keep_builtin_call_surface_visible_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='named-submit' name='submit' value='collision-submit'>
          <input id='named-request' name='requestSubmit' value='collision-request'>
          <input id='named-reset' name='reset' value='collision-reset'>
          <button id='submitter' type='submit'>Go</button>
        </form>
        <p id='result'></p>
        <script>
          const form = document.getElementById('target');
          form.dataset.submitCount = '0';
          form.dataset.resetCount = '0';
          form.addEventListener('submit', (event) => {
            form.dataset.submitCount = String(Number(form.dataset.submitCount) + 1);
            event.preventDefault();
          });
          form.addEventListener('reset', (event) => {
            form.dataset.resetCount = String(Number(form.dataset.resetCount) + 1);
            event.preventDefault();
          });

          const submitFn = Object.getOwnPropertyDescriptor(form, 'submit').value;
          const requestSubmitFn = Object.getOwnPropertyDescriptor(form, 'requestSubmit').value;
          const resetFn = Object.getOwnPropertyDescriptor(form, 'reset').value;

          const before = [
            typeof form.submit,
            typeof form['requestSubmit'],
            typeof form.reset,
            form.elements.namedItem('submit').id,
            form.elements.namedItem('requestSubmit').id,
            form.elements.namedItem('reset').id,
            String(form.submit !== form.elements.namedItem('submit')),
            String(Object.getOwnPropertyDescriptor(form, 'submit').enumerable === false),
            String(Object.getOwnPropertyNames(form).includes('requestSubmit')),
            String(Reflect.ownKeys(form).includes('reset'))
          ].join(':');

          submitFn.call(form);
          requestSubmitFn.call(form, document.getElementById('submitter'));
          resetFn.call(form);

          const after = [
            form.dataset.submitCount,
            form.dataset.resetCount,
            typeof form.submit,
            typeof form.requestSubmit,
            typeof form.reset
          ].join(':');

          document.getElementById('result').textContent = [before, after].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "function:function:function:named-submit:named-request:named-reset:true:true:true:true|1:1:function:function:function",
    )?;
    Ok(())
}

#[test]
fn form_attribute_backed_properties_stay_off_reflective_own_surface_work() -> Result<()> {
    let html = r#"
        <form
          id='target'
          name='signup'
          action='/submit'
          method='post'
          enctype='text/plain'
          target='_blank'
          accept-charset='UTF-8'
          novalidate
        >
          <input id='named-method' name='method' value='collision-method'>
          <input id='named-action' name='action' value='collision-action'>
        </form>
        <p id='result'></p>
        <script>
          const form = document.getElementById('target');
          const ownNames = Object.getOwnPropertyNames(form);
          const ownKeys = Reflect.ownKeys(form);
          document.getElementById('result').textContent = [
            form.name,
            form.method,
            form.enctype,
            form.target,
            form.acceptCharset,
            String(form.noValidate),
            String(form.action.includes('/submit')),
            String(Object.hasOwn(form, 'name')),
            String(Object.hasOwn(form, 'method')),
            String(Object.hasOwn(form, 'action')),
            String(Object.getOwnPropertyDescriptor(form, 'target') === undefined),
            String(ownNames.includes('acceptCharset')),
            String(ownKeys.includes('noValidate')),
            form.elements.namedItem('method').id,
            form.elements.namedItem('action').id
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "signup:post:text/plain:_blank:UTF-8:true:true:false:false:false:true:false:false:named-method:named-action",
    )?;
    Ok(())
}

#[test]
fn form_object_assign_and_spread_copy_only_expando_surface_work() -> Result<()> {
    let html = r#"
        <form id='target'>
          <input id='pick-a' type='radio' name='pick' value='a' checked>
          <input id='pick-b' type='radio' name='pick' value='b'>
          <input id='named-submit' name='submit' value='collision-submit'>
        </form>
        <p id='result'></p>
        <script>
          const form = document.getElementById('target');
          form.extra = 'expando';
          form.count = 3;
          const assigned = Object.assign({}, form);
          const spread = { ...form };

          document.getElementById('result').textContent = [
            String(Object.keys(form).includes('count')),
            String(Object.keys(form).includes('extra')),
            String(Object.keys(form).includes('pick')),
            String(Object.keys(form).includes('submit')),
            String(Reflect.ownKeys(form).includes('pick')),
            String(Reflect.ownKeys(form).includes('submit')),
            assigned.count,
            assigned.extra,
            String('pick' in assigned),
            String('submit' in assigned),
            spread.count,
            spread.extra,
            String('pick' in spread),
            String('submit' in spread),
            Object.keys(assigned).join(','),
            Object.keys(spread).join(',')
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true:true:false:false:true:true:3:expando:false:false:3:expando:false:false:count,extra:count,extra",
    )?;
    Ok(())
}

#[test]
fn form_attribute_backed_define_property_delete_and_reflect_set_work() -> Result<()> {
    let html = r#"
        <form
          id='target'
          name='signup'
          action='/submit'
          method='post'
          enctype='text/plain'
          target='_blank'
          accept-charset='UTF-8'
          novalidate
        ></form>
        <p id='result'></p>
        <script>
          const form = document.getElementById('target');

          Object.defineProperty(form, 'method', {
            value: 'shadow-method',
            writable: false,
            enumerable: false,
            configurable: true
          });
          Object.defineProperty(form, 'target', {
            get() { return this.getAttribute('target') + ':getter'; },
            set(value) { this.setAttribute('target', value + '-set'); },
            configurable: true
          });
          Object.defineProperty(form, 'encoding', {
            value: 'shadow-encoding',
            writable: true,
            enumerable: true,
            configurable: false
          });

          const shadow = [
            form.method,
            form.getAttribute('method'),
            String(Object.getOwnPropertyDescriptor(form, 'method').writable === false),
            String(Reflect.set(form, 'method', 'ignored') === false),
            form.method,
            form.getAttribute('method'),
            form.target,
            String(Reflect.set(form, 'target', 'self')),
            form.target,
            form.getAttribute('target'),
            form.encoding,
            form.enctype,
            String(Object.keys(form).includes('encoding'))
          ].join(':');

          const deleted = [
            String(delete form.method),
            form.method,
            form.getAttribute('method'),
            String(delete form.target),
            form.target,
            form.getAttribute('target'),
            String(delete form.encoding),
            form.encoding,
            form.enctype
          ].join(':');

          const reflected = [
            String(Reflect.set(form, 'method', 'put')),
            form.method,
            form.getAttribute('method'),
            String(Reflect.set(form, 'acceptCharset', 'Shift_JIS')),
            form.acceptCharset,
            form.getAttribute('accept-charset'),
            String(Reflect.set(form, 'noValidate', false)),
            String(form.noValidate),
            String(form.hasAttribute('novalidate'))
          ].join(':');

          document.getElementById('result').textContent = [shadow, deleted, reflected].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "shadow-method:post:true:true:shadow-method:post:_blank:getter:true:self-set:getter:self-set:shadow-encoding:text/plain:true|true:post:post:true:self-set:self-set:false:shadow-encoding:text/plain|true:put:put:true:Shift_JIS:Shift_JIS:true:false:false",
    )?;
    Ok(())
}

#[test]
fn form_direct_property_lookup_prefers_expando_over_reflected_builtin_and_named_surface_work()
-> Result<()> {
    let html = r#"
        <form id='target' name='signup'>
          <input id='pick-a' type='radio' name='pick' value='a' checked>
          <input id='pick-b' type='radio' name='pick' value='b'>
          <input id='named-submit' name='submit' value='collision-submit'>
        </form>
        <p id='result'></p>
        <script>
          const form = document.getElementById('target');
          const valueKey = 'value';

          Object.defineProperty(form, 'name', {
            value: 'own-name',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(form, 'submit', {
            value: 'own-submit',
            enumerable: true,
            configurable: true
          });
          Object.defineProperty(form, 'pick', {
            value: 'own-pick',
            enumerable: true,
            configurable: true
          });

          const shadow = [
            form.name,
            form.submit,
            form.pick,
            String(Object.keys(form).includes('name')),
            String(Object.keys(form).includes('submit')),
            String(Object.keys(form).includes('pick')),
            String(Object.getOwnPropertyDescriptor(form, 'submit').value === 'own-submit')
          ].join(':');

          delete form.name;
          delete form.submit;
          delete form.pick;
          const restoredPick = form['pick'];

          const restored = [
            form.name,
            typeof form.submit,
            Object.prototype.toString.call(restoredPick),
            restoredPick[valueKey],
            String(Object.keys(form).includes('name')),
            String(Object.keys(form).includes('submit')),
            String(Object.keys(form).includes('pick')),
            form.elements.namedItem('submit').id
          ].join(':');

          document.getElementById('result').textContent = [shadow, restored].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "own-name:own-submit:own-pick:true:true:true:true|signup:function:[object RadioNodeList]:a:false:false:false:named-submit",
    )?;
    Ok(())
}
