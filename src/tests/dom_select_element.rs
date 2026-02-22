use super::*;

#[test]
fn select_implicit_role_and_selection_properties_work() -> Result<()> {
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
              single.role + ':' +
              single.value + ':' +
              single.selectedIndex + ':' +
              single.options.length + ':' +
              sized.role + ':' +
              multi.role + ':' +
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
    h.assert_text(
        "#result",
        "combobox:cat:2:4:listbox:listbox:y:1|parrot:3:false:true",
    )?;
    Ok(())
}

#[test]
fn select_attributes_and_role_override_roundtrip_work() -> Result<()> {
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
              cars.role + ':' +
              cars.tagName + ':' +
              cars.getAttribute('name') + ':' +
              cars.getAttribute('form') + ':' +
              cars.getAttribute('autocomplete') + ':' +
              cars.required + ':' +
              cars.disabled + ':' +
              cars.hasAttribute('multiple') + ':' +
              cars.getAttribute('size');

            cars.disabled = false;
            cars.required = false;
            cars.removeAttribute('multiple');
            cars.setAttribute('size', '1');
            const updated =
              cars.role + ':' +
              cars.required + ':' +
              cars.disabled + ':' +
              cars.hasAttribute('multiple') + ':' +
              cars.getAttribute('size');

            cars.role = 'menu';
            const assigned = cars.role + ':' + cars.getAttribute('role');
            cars.removeAttribute('role');
            const restored = cars.role + ':' + (cars.getAttribute('role') === null);

            document.getElementById('result').textContent =
              initial + '|' + updated + '|' + assigned + '|' + restored;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "listbox:SELECT:car:search-form:off:true:true:true:4|combobox:false:false:false:1|menu:menu|combobox:true",
    )?;
    Ok(())
}
