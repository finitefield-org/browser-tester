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
