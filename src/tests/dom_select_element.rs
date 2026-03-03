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
