use browser_tester::Harness;

#[test]
fn regex_charclass_quote_does_not_break_balancer() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      "<a&b>".replace(/[&<>"']/g, (ch) => ch);
      document.getElementById("result").textContent = "ok";
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn unary_not_regex_test_condition_parses() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const value = "abc";
      if (!/^[0-9]+$/.test(value)) {
        document.getElementById("result").textContent = "non-numeric";
      } else {
        document.getElementById("result").textContent = "numeric";
      }
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "non-numeric")?;
    Ok(())
}

#[test]
fn window_unknown_property_falls_back_to_undefined() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const value = window.lucide;
      document.getElementById("result").textContent = String(value === undefined);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn document_element_lang_is_readable() -> browser_tester::Result<()> {
    let html = r#"
    <html lang="ja">
      <body>
        <div id="result"></div>
        <script>
          document.getElementById("result").textContent = document.documentElement.lang;
        </script>
      </body>
    </html>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ja")?;
    Ok(())
}

#[test]
fn object_literal_shorthand_and_spread_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const seed = { count: 1 };
      const name = "lot";
      const merged = { ...seed, name };
      document.getElementById("result").textContent = merged.count + ":" + merged.name;
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "1:lot")?;
    Ok(())
}

#[test]
fn object_property_compound_assignment_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const obj = { count: 1 };
      obj.count += 1;
      document.getElementById("result").textContent = String(obj.count);
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "2")?;
    Ok(())
}

#[test]
fn array_destructured_callback_params_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id="result"></div>
    <script>
      const obj = { a: "x", b: "y" };
      const result = document.getElementById("result");
      Object.entries(obj).forEach(([k, v]) => {
        result.textContent = result.textContent + k + v;
      });
    </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "axby")?;
    Ok(())
}

#[test]
fn real_world_picking_breakdown_smoke() -> browser_tester::Result<()> {
    let html = include_str!("fixtures/picking-breakdown-minimal.html");
    let harness = Harness::from_html(html)?;
    harness.assert_text("#result", "ja|true|true|2")?;
    Ok(())
}

#[test]
fn array_push_trailing_comma_is_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      const lines = [];
      lines.push("x",);
      document.getElementById("r").textContent = lines.join(",");
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "x")?;
    Ok(())
}

#[test]
fn nested_call_with_trailing_commas_is_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      function interpolate(v, obj) { return v + ":" + obj.n; }
      const lines = [];
      lines.push(interpolate("total", { n: 1, },),);
      document.getElementById("r").textContent = lines[0];
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "total:1")?;
    Ok(())
}

#[test]
fn empty_call_argument_is_still_rejected() {
    let err = Harness::from_html("<script>const a=[]; a.push(1,,2);</script>").unwrap_err();
    match err {
        browser_tester::Error::ScriptParse(msg) => assert!(msg.contains("empty")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn lone_comma_argument_is_rejected() {
    let err = Harness::from_html("<script>function f(x){} f(,);</script>").unwrap_err();
    match err {
        browser_tester::Error::ScriptParse(msg) => assert!(msg.contains("empty")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn function_call_spread_arguments_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      function sum(x, y, z) { return x + y + z; }
      const numbers = [1, 2, 3];
      document.getElementById("r").textContent = String(sum(...numbers));
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "6")?;
    Ok(())
}

#[test]
fn mixed_call_spread_arguments_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      function myFunction(v, w, x, y, z) { return v + "," + w + "," + x + "," + y + "," + z; }
      const args = [0, 1];
      document.getElementById("r").textContent = myFunction(-1, ...args, 2, ...[3]);
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "-1,0,1,2,3")?;
    Ok(())
}

#[test]
fn array_push_spread_arguments_supported() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      const lines = ["a"];
      const next = ["b", "c"];
      lines.push(...next,);
      document.getElementById("r").textContent = lines.join(",");
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "a,b,c")?;
    Ok(())
}

#[test]
fn object_spread_primitives_follow_js_behavior() -> browser_tester::Result<()> {
    let html = r#"
    <div id='r'></div>
    <script>
      const obj = { ...true, ..."test", ...10 };
      document.getElementById("r").textContent =
        obj["0"] + obj["1"] + obj["2"] + obj["3"] + "|" + String(obj["4"] === undefined);
    </script>
    "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#r", "test|true")?;
    Ok(())
}

#[test]
fn non_iterable_spread_in_call_is_rejected() {
    let err = Harness::from_html("<script>function f(a){} const obj={a:1}; f(...obj);</script>")
        .unwrap_err();
    match err {
        browser_tester::Error::ScriptRuntime(msg) => assert!(msg.contains("iterable")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn non_iterable_spread_in_array_literal_is_rejected() {
    let err =
        Harness::from_html("<script>const obj={a:1}; const arr=[...obj];</script>").unwrap_err();
    match err {
        browser_tester::Error::ScriptRuntime(msg) => assert!(msg.contains("iterable")),
        other => panic!("unexpected error: {other:?}"),
    }
}
