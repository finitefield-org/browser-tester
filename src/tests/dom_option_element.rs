use super::*;

#[test]
fn option_role_value_and_state_roundtrip_work() -> Result<()> {
    let html = r#"
        <label for='pet-select'>Choose a pet:</label>
        <select id='pet-select'>
          <option id='placeholder'>--Please choose an option--</option>
          <option id='dog' value='dog'>Dog</option>
          <option id='cat' value='cat' selected>Cat</option>
          <optgroup id='birds' label='Birds' disabled>
            <option id='parrot' value='parrot'>Parrot</option>
          </optgroup>
        </select>

        <datalist id='pet-hints'>
          <option id='hint-a' value='hamster'></option>
          <option id='hint-b'>Goldfish</option>
        </datalist>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const placeholder = document.getElementById('placeholder');
            const dog = document.getElementById('dog');
            const cat = document.getElementById('cat');
            const parrot = document.getElementById('parrot');
            const select = document.getElementById('pet-select');
            const hintA = document.getElementById('hint-a');
            const hintB = document.getElementById('hint-b');

            const initial =
              placeholder.role + ':' +
              placeholder.value + ':' +
              dog.value + ':' +
              cat.getAttribute('selected') + ':' +
              select.value + ':' +
              parrot.disabled + ':' +
              hintA.value + ':' +
              hintB.value;

            dog.setAttribute('label', 'Canine');
            dog.disabled = true;
            cat.removeAttribute('selected');
            parrot.setAttribute('selected', '');
            placeholder.value = '';

            const updated =
              dog.getAttribute('label') + ':' +
              dog.disabled + ':' +
              dog.getAttribute('disabled') + ':' +
              select.value + ':' +
              placeholder.getAttribute('value');

            document.getElementById('result').textContent = initial + '|' + updated;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "option:--Please choose an option--:dog:true:cat:true:hamster:Goldfish|Canine:true:true:parrot:",
    )?;
    Ok(())
}

#[test]
fn option_optional_end_tag_parsing_before_option_and_optgroup_works() -> Result<()> {
    let html = r#"
        <select id='numbers'>
          <option id='one' value='1'>One
          <option id='two'>Two
          <optgroup id='more' label='More'>
            <option id='three'>Three
            <option id='four' value='4'>Four
          </optgroup>
        </select>

        <button id='run' type='button'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const one = document.getElementById('one');
            const two = document.getElementById('two');
            const three = document.getElementById('three');
            const four = document.getElementById('four');

            document.getElementById('result').textContent =
              document.querySelectorAll('#numbers > option').length + ':' +
              document.querySelectorAll('#numbers > optgroup').length + ':' +
              document.querySelectorAll('#more > option').length + ':' +
              one.querySelectorAll('option').length + ':' +
              two.querySelectorAll('optgroup').length + ':' +
              two.value.trim() + ':' +
              three.value.trim() + ':' +
              four.value + ':' +
              three.role;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2:1:2:0:0:Two:Three:4:option")?;
    Ok(())
}
