use super::*;

#[test]
fn select_selection_properties_work() -> Result<()> {
    let html = r#"
        <label for='pet-select'>Choose a pet:</label>
        <select id='pet-select' name='pets'>
          <option value=''>--Please choose an option--</option>
          <option id='dog' value='dog'>Dog</option>
          <option id='cat' value='cat' selected>Cat</option>
          <option id='parrot' value='parrot'>Parrot</option>
        </select>

        <select id='size-list' size='2'>
          <option value='a'>A</option>
          <option value='b' selected>B</option>
        </select>

        <select id='multi-list' multiple size='4'>
          <option value='x'>X</option>
          <option value='y' selected>Y</option>
          <option value='z'>Z</option>
        </select>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const single = document.getElementById('pet-select');
            const sized = document.getElementById('size-list');
            const multi = document.getElementById('multi-list');

            const initial =
              single.value + ':' +
              single.selectedIndex + ':' +
              single.options.length + ':' +
              multi.value + ':' +
              multi.selectedIndex;

            single.value = 'parrot';
            const updated =
              single.value + ':' +
              single.selectedIndex + ':' +
              document.getElementById('cat').hasAttribute('selected') + ':' +
              document.getElementById('parrot').hasAttribute('selected');

            document.getElementById('result').textContent = initial + '|' + updated;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "cat:2:4:y:1|parrot:3:false:true")?;
    Ok(())
}

#[test]
fn select_attributes_roundtrip_work() -> Result<()> {
    let html = r#"
        <form id='search-form'></form>
        <select
          id='cars'
          name='car'
          form='search-form'
          autocomplete='off'
          required
          disabled
          multiple
          size='4'>
          <option value='sedan'>Sedan</option>
          <option value='suv' selected>SUV</option>
          <option value='wagon'>Wagon</option>
        </select>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const cars = document.getElementById('cars');

            const initial =
              cars.tagName + ':' +
              cars.type + ':' +
              cars.getAttribute('name') + ':' +
              cars.form.id + ':' +
              cars.autocomplete + ':' +
              cars.required + ':' +
              cars.disabled + ':' +
              cars.multiple + ':' +
              cars.size;

            cars.disabled = false;
            cars.required = false;
            cars.removeAttribute('multiple');
            cars.setAttribute('size', '1');
            const updated =
              cars.required + ':' +
              cars.disabled + ':' +
              cars.multiple + ':' +
              cars.size + ':' +
              cars.type;

            document.getElementById('result').textContent =
              initial + '|' + updated;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "SELECT:select-multiple:car:search-form:off:true:true:true:4|false:false:false:1:select-one",
    )?;
    Ok(())
}

#[test]
fn setting_select_value_updates_selection_state() -> Result<()> {
    let html = r#"
        <select id='formwork-opening-faces-override'>
          <option value='auto' selected>auto</option>
          <option id='one' value='1'>one</option>
        </select>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const select = document.getElementById('formwork-opening-faces-override');
            select.value = '1';
            document.getElementById('result').textContent = [
              select.value,
              select.selectedIndex,
              select.options.item(0).hasAttribute('selected'),
              select.options.item(1).hasAttribute('selected')
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_value("#formwork-opening-faces-override", "1")?;
    h.assert_text("#result", "1:1:false:true")?;
    Ok(())
}

#[test]
fn harness_set_select_value_updates_select_and_dispatches_events() -> Result<()> {
    let html = r#"
        <select id='json-key-sort-indent'>
          <option value='0' selected>auto</option>
          <option value='2'>2</option>
          <option value='4'>4</option>
        </select>
        <p id='result'></p>
        <script>
          const select = document.getElementById('json-key-sort-indent');
          const logs = [];
          select.addEventListener('input', () => logs.push('input:' + select.value));
          select.addEventListener('change', () => logs.push('change:' + select.value));
          select.addEventListener('change', () => {
            document.getElementById('result').textContent = logs.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.set_select_value("#json-key-sort-indent", "4")?;
    h.assert_value("#json-key-sort-indent", "4")?;
    h.assert_text("#result", "input:4|change:4")?;
    Ok(())
}

#[test]
fn type_text_accepts_select_and_updates_value() -> Result<()> {
    let html = r#"
        <select id='json-key-sort-indent'>
          <option value='0' selected>auto</option>
          <option value='2'>2</option>
          <option value='4'>4</option>
        </select>
        <p id='result'></p>
        <script>
          const select = document.getElementById('json-key-sort-indent');
          const logs = [];
          select.addEventListener('input', () => logs.push('input:' + select.value));
          select.addEventListener('change', () => logs.push('change:' + select.value));
          select.addEventListener('change', () => {
            document.getElementById('result').textContent = logs.join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#json-key-sort-indent", "2")?;
    h.assert_value("#json-key-sort-indent", "2")?;
    h.assert_text("#result", "input:2|change:2")?;
    Ok(())
}

#[test]
fn select_interface_properties_reflect_form_labels_and_selection_state() -> Result<()> {
    let html = r#"
        <form id='pet-form'></form>
        <label id='pet-label' for='pets'>Pets</label>
        <select id='pets' name='pets' form='pet-form' autocomplete='list' required>
          <option value=''>Choose</option>
          <option id='dog' value='dog' selected>Dog</option>
          <option id='cat' value='cat'>Cat</option>
        </select>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const select = document.getElementById('pets');
            const labels = select.labels;
            const selected = select.selectedOptions;
            document.getElementById('result').textContent = [
              select.autocomplete,
              select.form.id,
              labels.length,
              labels.item(0).id,
              select.length,
              select.multiple,
              select.name,
              select.options.length,
              select.required,
              select.selectedIndex,
              selected.length,
              selected.item(0).id,
              select.size,
              select.type,
              select.value,
              select.willValidate,
              select.validity.valueMissing,
              select.validationMessage === ''
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "list:pet-form:1:pet-label:3:false:pets:3:true:1:1:dog:1:select-one:dog:true:false:true",
    )?;
    Ok(())
}

#[test]
fn select_item_named_item_add_and_remove_index_work() -> Result<()> {
    let html = r#"
        <select id='list'>
          <option id='o1' name='alpha' value='a'>A</option>
          <option id='o2' value='b' selected>B</option>
        </select>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('list');

            const o3 = document.createElement('option');
            o3.id = 'o3';
            o3.setAttribute('name', 'gamma');
            o3.value = 'c';
            o3.textContent = 'C';
            list.add(o3);

            const o4 = document.createElement('option');
            o4.id = 'o4';
            o4.value = 'd';
            o4.textContent = 'D';
            list.add(o4, 1);

            const item1 = list.item(1).id;
            const named = list.namedItem('gamma').id;

            list.remove(2);

            document.getElementById('result').textContent = [
              list.length,
              item1,
              named,
              list.options.item(0).id,
              list.options.item(1).id,
              list.options.item(2).id,
              list.value,
              list.selectedIndex
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "3:o4:o3:o1:o4:o3:a:0")?;
    Ok(())
}

#[test]
fn select_selected_index_length_multiple_and_size_setters_work() -> Result<()> {
    let html = r#"
        <select id='list'>
          <option value='a'>A</option>
          <option value='b' selected>B</option>
          <option value='c'>C</option>
        </select>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('list');

            list.selectedIndex = 2;
            const step1 = [
              list.value,
              list.selectedIndex,
              list.selectedOptions.length,
              list.selectedOptions[0].value
            ].join(',');

            list.selectedIndex = -1;
            const step2 = [
              list.value === '',
              list.selectedIndex,
              list.selectedOptions.length
            ].join(',');

            list.length = 2;
            const step3 = [
              list.options.length,
              list.value,
              list.selectedIndex
            ].join(',');

            list.length = 4;
            const step4 = [
              list.options.length,
              list.options[2].value === '',
              list.options[3].textContent === ''
            ].join(',');

            list.multiple = true;
            list.size = 6;
            list.type = 'select-one';
            const step5 = [
              list.multiple,
              list.size,
              list.type,
              list.getAttribute('size')
            ].join(',');

            document.getElementById('result').textContent =
              step1 + '|' + step2 + '|' + step3 + '|' + step4 + '|' + step5;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "c,2,1,c|true,-1,0|2,a,0|4,true,true|true,6,select-multiple,6",
    )?;
    Ok(())
}

#[test]
fn select_validity_will_validate_and_custom_validity_work() -> Result<()> {
    let html = r#"
        <form id='f'></form>
        <select id='pet' form='f' required>
          <option value='' selected>Choose</option>
          <option value='dog'>Dog</option>
        </select>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const pet = document.getElementById('pet');
            const first = [
              pet.willValidate,
              pet.checkValidity(),
              pet.validity.valueMissing,
              pet.validationMessage === ''
            ].join(',');

            pet.setCustomValidity('Pick one');
            const second = [
              pet.reportValidity(),
              pet.validity.customError,
              pet.validationMessage
            ].join(',');

            pet.setCustomValidity('');
            pet.value = 'dog';
            const third = [
              pet.checkValidity(),
              pet.validity.valueMissing,
              pet.validity.customError,
              pet.value
            ].join(',');

            pet.disabled = true;
            const fourth = [
              pet.willValidate,
              pet.checkValidity(),
              pet.validity.valid
            ].join(',');

            document.getElementById('result').textContent =
              first + '|' + second + '|' + third + '|' + fourth;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true,false,true,true|false,true,Pick one|true,false,false,dog|false,true,true",
    )?;
    Ok(())
}

#[test]
fn select_item_requires_an_index_argument() -> Result<()> {
    let html = r#"
        <select id='list'><option value='a'>A</option></select>
        <button id='run' type='button'>run</button>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('list').item();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    match h.click("#run") {
        Err(Error::ScriptRuntime(message)) => {
            assert!(
                message.contains("item on HTMLSelectElement requires exactly one index argument"),
                "unexpected runtime error message: {message}"
            );
        }
        other => panic!("expected runtime error, got: {other:?}"),
    }
    Ok(())
}

#[test]
fn select_options_and_selected_options_are_live_cached_specialized_collections_work() -> Result<()>
{
    let html = r#"
        <select id='list' multiple>
          <option id='o1' name='alpha' value='a' selected>A</option>
          <option id='o2' value='b'>B</option>
        </select>
        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const list = document.getElementById('list');
            const options = list.options;
            const sameOptions = list.options;
            const selected = list.selectedOptions;
            const sameSelected = list.selectedOptions;

            const o3 = document.createElement('option');
            o3.id = 'o3';
            o3.setAttribute('name', 'gamma');
            o3.value = 'c';
            o3.textContent = 'C';
            o3.setAttribute('selected', '');
            list.appendChild(o3);

            document.getElementById('result').textContent = [
              String(sameOptions === options),
              String(sameSelected === selected),
              Object.prototype.toString.call(options),
              Object.prototype.toString.call(selected),
              options.constructor.name,
              selected.constructor.name,
              String(Object.getPrototypeOf(options) === HTMLOptionsCollection.prototype),
              String(Object.getPrototypeOf(selected) === HTMLCollection.prototype),
              options.namedItem('alpha').id,
              options.namedItem('gamma').id,
              String(options.length),
              String(selected.length),
              selected.item(1).id,
              String(list.options === options),
              String(list.selectedOptions === selected)
            ].join(':');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "true:true:[object HTMLOptionsCollection]:[object HTMLCollection]:HTMLOptionsCollection:HTMLCollection:true:true:o1:o3:3:2:o3:true:true",
    )?;
    Ok(())
}

#[test]
fn select_options_define_property_delete_and_shadow_parity_work() -> Result<()> {
    let html = r#"
        <select id='list'>
          <option id='o1' name='alpha' value='a'>A</option>
          <option id='o2' value='b'>B</option>
        </select>
        <p id='result'></p>
        <script>
          const options = document.getElementById('list').options;

          const returnedZero = Object.defineProperty(options, '0', { value: 'shadow-zero' });
          const returnedLength = Object.defineProperty(options, 'length', { value: 41 });
          const returnedAlpha = Object.defineProperty(options, 'alpha', { value: 'shadow-alpha' });

          const zeroDesc = Object.getOwnPropertyDescriptor(options, '0');
          const lengthDesc = Object.getOwnPropertyDescriptor(options, 'length');
          const alphaDesc = Object.getOwnPropertyDescriptor(options, 'alpha');

          const before = [
            String(returnedZero === options),
            String(returnedLength === options),
            String(returnedAlpha === options),
            options[0],
            options.length,
            options.alpha,
            String(zeroDesc.enumerable),
            String(zeroDesc.configurable),
            String(zeroDesc.writable),
            String(lengthDesc.enumerable),
            String(lengthDesc.configurable),
            String(lengthDesc.writable),
            String(alphaDesc.enumerable),
            String(alphaDesc.configurable),
            String(alphaDesc.writable)
          ].join(':');

          const deleted = [
            String(delete options[0]),
            String(delete options.length),
            String(delete options.alpha)
          ].join(':');

          const after = [
            options[0].id,
            String(options.length),
            options.alpha.id,
            options.namedItem('alpha').id
          ].join(':');

          document.getElementById('result').textContent = [before, deleted, after].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true:true:true:shadow-zero:41:shadow-alpha:true:true:false:false:true:false:true:true:false|true:true:true|o1:2:o1:o1",
    )?;
    Ok(())
}

#[test]
fn selected_options_and_datalist_options_keep_html_collection_collision_rules_work() -> Result<()> {
    let html = r#"
        <select id='list' multiple>
          <option id='sel-item' name='namedItem' value='a' selected>A</option>
          <option id='sel-hidden' name='item' value='b'>B</option>
          <option id='sel-values' name='values' value='c' selected>C</option>
          <option id='sel-ctor' name='constructor' value='d' selected>D</option>
        </select>
        <datalist id='choices'>
          <option id='dl-named' name='namedItem' value='x'></option>
          <option id='dl-length' name='length' value='y'></option>
          <option id='dl-values' name='values' value='z'></option>
        </datalist>
        <p id='result'></p>
        <script>
          const list = document.getElementById('list');
          const datalist = document.getElementById('choices');
          const selected = list.selectedOptions;
          const sameSelected = list.selectedOptions;
          const options = datalist.options;
          const sameOptions = datalist.options;
          const selectedKeys = Reflect.ownKeys(selected);
          const datalistKeys = Reflect.ownKeys(options);

          const later = document.createElement('option');
          later.id = 'dl-entries';
          later.setAttribute('name', 'entries');
          later.value = 'later';
          datalist.appendChild(later);

          document.getElementById('result').textContent = [
            String(sameSelected === selected),
            Object.prototype.toString.call(selected),
            selected.constructor.name,
            String(Object.getPrototypeOf(selected) === HTMLCollection.prototype),
            typeof selected.item,
            typeof selected.namedItem,
            selected.namedItem('namedItem').id,
            selected.namedItem('values').id,
            selected.namedItem('constructor').id,
            String(selectedKeys.includes('item')),
            String(selectedKeys.includes('namedItem')),
            String(selectedKeys.includes('values')),
            String(selectedKeys.includes('constructor')),
            String(sameOptions === options),
            Object.prototype.toString.call(options),
            options.constructor.name,
            String(Object.getPrototypeOf(options) === HTMLCollection.prototype),
            options.namedItem('namedItem').id,
            options.namedItem('length').id,
            options.namedItem('values').id,
            options.namedItem('entries').id,
            String(datalistKeys.includes('namedItem')),
            String(datalistKeys.includes('length')),
            String(datalistKeys.includes('values')),
            String(datalistKeys.includes('constructor')),
            String(datalist.options === options)
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true:[object HTMLCollection]:HTMLCollection:true:function:function:sel-item:sel-values:sel-ctor:false:false:false:false:true:[object HTMLCollection]:HTMLCollection:true:dl-named:dl-length:dl-values:dl-entries:false:true:false:false:true",
    )?;
    Ok(())
}

#[test]
fn selected_options_and_options_duplicate_names_do_not_switch_to_radio_node_lists_work()
-> Result<()> {
    let html = r#"
        <select id='list' multiple>
          <option id='o1' name='dup' value='a' selected>A</option>
          <option id='o2' name='dup' value='b' selected>B</option>
          <option id='o3' name='dup' value='c'>C</option>
        </select>
        <p id='result'></p>
        <script>
          const list = document.getElementById('list');
          const selected = list.selectedOptions;
          const selectedNamed = selected.namedItem('dup');
          const selectedDirect = selected['dup'];
          const optionsNamed = list.options.namedItem('dup');
          const optionsDirect = list.options['dup'];

          document.getElementById('result').textContent = [
            Object.prototype.toString.call(selected),
            selected.constructor.name,
            selectedNamed.id,
            String(selectedNamed === document.getElementById('o1')),
            String(selectedDirect === selectedNamed),
            typeof selectedDirect.namedItem,
            String(optionsNamed === document.getElementById('o1')),
            String(optionsDirect === optionsNamed),
            typeof optionsDirect.namedItem,
            String(list.selectedOptions === selected)
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "[object HTMLCollection]:HTMLCollection:o1:true:true:undefined:true:true:undefined:true",
    )?;
    Ok(())
}
