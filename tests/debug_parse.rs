use browser_tester::Harness;

#[test]
fn debug_single() {
    let html = "<script>const a = document.getElementById('a'); document.getElementById('btn').addEventListener('click', () => {});</script>";
    let err = Harness::from_html(html).err();
    println!("single err = {:?}", err);
}

#[test]
fn debug_focus_active_element_ternary() {
    let html = r#"
    <input id='a'>
    <input id='b'>
    <button id='btn'>run</button>
    <p id='result'></p>
    <script>
      const a = document.getElementById('a');
      const b = document.getElementById('b');
      let order = '';

      a.addEventListener('focus', () => {
        order += 'aF';
      });
      a.addEventListener('blur', () => {
        order += 'aB';
      });
      b.addEventListener('focus', () => {
        order += 'bF';
      });
      b.addEventListener('blur', () => {
        order += 'bB';
      });

      document.getElementById('btn').addEventListener('click', () => {
        a.focus();
        b.focus();
        b.blur();
        document.getElementById('result').textContent =
          order + ':' + (document.activeElement === null ? 'none' : 'active');
      });
    </script>
    "#;

    let mut h = Harness::from_html(html).unwrap();
    h.click("#btn").unwrap();
    println!("result dom: {}", h.dump_dom("#result").unwrap());
}

#[test]
fn debug_active_element_ternary_direct() {
    let html = r#"
    <button id='btn'>run</button>
    <p id='result'></p>
    <script>
      document.getElementById('btn').addEventListener('click', () => {
        document.getElementById('result').textContent =
          document.activeElement === null ? 'none' : 'active';
      });
    </script>
    "#;

    let mut h = Harness::from_html(html).unwrap();
    h.click("#btn").unwrap();
    println!("result dom direct: {}", h.dump_dom("#result").unwrap());
    h.assert_text("#result", "none").unwrap();
}

#[test]
fn debug_concat_and_ternary() {
    let html = r#"
    <button id='btn'>run</button>
    <p id='result'></p>
    <p id='concat2'></p>
    <p id='concat3'></p>
    <script>
      document.getElementById('btn').addEventListener('click', () => {
        const order = 'aFaBbFbB';
        document.getElementById('result').textContent =
          order + ':' + (document.activeElement === null ? 'none' : 'active');
        document.getElementById('concat2').textContent =
          order + (document.activeElement === null ? 'none' : 'active');
        document.getElementById('concat3').textContent =
          (document.activeElement === null ? 'none' : 'active');
      });
    </script>
    "#;
    let mut h = Harness::from_html(html).unwrap();
    h.click("#btn").unwrap();
    println!("concat direct: {}", h.dump_dom("#result").unwrap());
    println!("concat2: {}", h.dump_dom("#concat2").unwrap());
    println!("concat3: {}", h.dump_dom("#concat3").unwrap());
}

#[test]
fn debug_ternary_variable() {
    let html = r#"
    <button id='btn'>run</button>
    <p id='result'></p>
    <script>
      document.getElementById('btn').addEventListener('click', () => {
        const suffix = document.activeElement === null ? 'none' : 'active';
        document.getElementById('result').textContent = 'start:' + suffix;
      });
    </script>
    "#;
    let mut h = Harness::from_html(html).unwrap();
    h.click("#btn").unwrap();
    println!("ternary var: {}", h.dump_dom("#result").unwrap());
    h.assert_text("#result", "start:none").unwrap();
}
