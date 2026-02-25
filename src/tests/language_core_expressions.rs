use super::*;

#[test]
fn foreach_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
          <li class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            document.querySelectorAll('.item').forEach((item, idx) => {
              if (idx === 0) {
                continue;
              }
              if (idx === 2) {
                break;
              }
              out = out + idx;
            });
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn for_in_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
          <li class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (let index in document.querySelectorAll('.item')) {
              if (index === '1') {
                continue;
              }
              if (index === '3') {
                break;
              }
              out = out + index;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "02")?;
    Ok(())
}

#[test]
fn for_in_loop_iterates_object_string_keys_and_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const object = { a: 1, b: 2, c: 3 };
            let out = '';
            for (const property in object) {
              out = out + property + ':' + object[property] + '|';
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a:1|b:2|c:3|")?;
    Ok(())
}

#[test]
fn for_in_loop_includes_inherited_properties_and_skips_shadowed_keys() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function Base() {}
          Base.prototype.a = 1;
          Base.prototype.b = 2;

          document.getElementById('btn').addEventListener('click', () => {
            const obj = new Base();
            obj.b = 20;
            obj.c = 3;
            let out = '';
            for (const key in obj) {
              out = out + key + ',';
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "b,c,a,")?;
    Ok(())
}

#[test]
fn for_in_loop_orders_integer_keys_before_other_strings() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = {};
            obj.foo = 'x';
            obj[2] = 'b';
            obj[1] = 'a';
            obj.bar = 'y';
            let out = '';
            for (const key in obj) {
              out = out + key + ',';
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1,2,foo,bar,")?;
    Ok(())
}

#[test]
fn for_in_loop_ignores_symbol_keys() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const token = Symbol.for('token');
            const obj = { a: 1 };
            obj[token] = 2;
            let out = '';
            for (const key in obj) {
              out = out + key;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "a")?;
    Ok(())
}

#[test]
fn for_of_loop_supports_break_and_continue() -> Result<()> {
    let html = r#"
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
          <li class='item'>D</li>
        </ul>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (const item of document.querySelectorAll('.item')) {
              if (item.textContent === 'B') {
                continue;
              }
              if (item.textContent === 'D') {
                break;
              }
              out = out + item.textContent;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AC")?;
    Ok(())
}

#[test]
fn for_of_loop_supports_string_and_typed_array_iterables() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let text = '';
            for (const ch of 'abc') {
              text = text + ch;
            }

            let bytes = '';
            for (const value of new Uint8Array([7, 8])) {
              bytes = bytes + String(value) + ',';
            }

            document.getElementById('result').textContent = text + ':' + bytes;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "abc:7,8,")?;
    Ok(())
}

#[test]
fn for_of_loop_uses_symbol_iterator_factory_and_closes_on_break() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let closed = 'no';
            const iterable = {};
            iterable[Symbol.iterator] = () => {
              let i = 0;
              const iterator = {};
              iterator.next = () => {
                const result = {};
                if (i < 3) {
                  result.value = i;
                  i += 1;
                  result.done = false;
                  return result;
                }
                result.done = true;
                return result;
              };
              iterator['return'] = () => {
                closed = 'yes';
                const result = {};
                result.done = true;
                return result;
              };
              return iterator;
            };

            let out = '';
            for (const value of iterable) {
              out = out + String(value);
              if (value === 1) {
                break;
              }
            }
            document.getElementById('result').textContent = out + ':' + closed;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "01:yes")?;
    Ok(())
}

#[test]
fn for_of_loop_rejects_non_iterable_objects() {
    let err = Harness::from_html("<script>for (const x of { a: 1 }) { }</script>")
        .expect_err("for...of on plain object should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("for...of iterable must be")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn for_of_loop_rejects_async_as_loop_variable_name() {
    let err = Harness::from_html("<script>for (async of [1, 2, 3]) { }</script>")
        .expect_err("for...of with async binding should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("for-of loop may not be 'async'") || msg.contains("'async'"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn for_await_of_consumes_async_generator_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          async function* values() {
            yield Promise.resolve('a');
            yield Promise.resolve('b');
            yield 'c';
          }

          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for await (const item of values()) {
              out = out + item;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "abc")?;
    Ok(())
}

#[test]
fn for_await_of_consumes_sync_iterable_and_awaits_promises() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const source = [Promise.resolve('x'), 'y', Promise.resolve('z')];
            let out = '';
            for await (const item of source) {
              out = out + item;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "xyz")?;
    Ok(())
}

#[test]
fn for_await_of_uses_symbol_async_iterator_from_iterable_object() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const stream = new Blob(['AB']).stream();
            let out = '';
            for await (const chunk of stream) {
              out = out + chunk.join(',');
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "65,66")?;
    Ok(())
}

#[test]
fn for_await_of_rejects_for_in_form() {
    let err = Harness::from_html("<script>for await (const key in { a: 1 }) { }</script>")
        .expect_err("for await...in should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("only supports of") || msg.contains("of-clause"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn while_loop_break_exits_loop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            while (i < 6) {
              if (i === 3) {
                break;
              }
              i += 1;
            }
            document.getElementById('result').textContent = String(i);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn while_loop_basic_accumulation_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 0;
            let x = 0;
            while (n < 3) {
              n += 1;
              x += n;
            }
            document.getElementById('result').textContent = String(n) + ':' + String(x);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3:6")?;
    Ok(())
}

#[test]
fn while_loop_continue_rechecks_condition() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            let n = 0;
            while (i < 5) {
              i += 1;
              if (i === 3) {
                continue;
              }
              n += i;
            }
            document.getElementById('result').textContent = String(n);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12")?;
    Ok(())
}

#[test]
fn while_loop_supports_single_statement_body_without_block() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let n = 0;
            while (n < 3) n += 1;
            document.getElementById('result').textContent = String(n);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn while_loop_supports_empty_statement_body() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            function next() {
              i += 1;
              return i < 4;
            }
            while (next());
            document.getElementById('result').textContent = String(i);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "4")?;
    Ok(())
}

#[test]
fn while_statement_requires_a_body() {
    match Harness::from_html("<script>while (true)</script>") {
        Err(Error::ScriptParse(msg)) => assert!(msg.contains("while statement has no body")),
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("while without body should fail"),
    }
}

#[test]
fn while_single_statement_lexical_declaration_is_rejected() {
    match Harness::from_html("<script>while (false) let x = 1;</script>") {
        Err(Error::ScriptParse(msg)) => {
            assert!(msg.contains("lexical declaration cannot appear in a single-statement context"))
        }
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("while with lexical declaration single body should fail"),
    }
}

#[test]
fn for_loop_allows_empty_statement_body() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            for (; i < 5; i++);
            document.getElementById('result').textContent = String(i);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5")?;
    Ok(())
}

#[test]
fn for_loop_with_in_operator_in_condition_is_not_misparsed_as_for_in() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = { a: 1 };
            let count = 0;
            for (let key = 'a'; key in obj; key = 'missing') {
              count += 1;
            }
            document.getElementById('result').textContent = String(count);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn for_loop_supports_multiple_counters_in_init_and_post() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const arr = [1, 2, 3, 4, 5, 6];
            let out = '';
            for (let l = 0, r = arr.length - 1; l < r; l++, r--) {
              out = out + String(arr[l]) + String(arr[r]);
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "162534")?;
    Ok(())
}

#[test]
fn for_loop_with_omitted_condition_uses_break_to_terminate() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            let out = '';
            for (; ; i++) {
              if (i > 3) {
                break;
              }
              out = out + String(i);
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0123")?;
    Ok(())
}

#[test]
fn for_loop_with_all_clauses_omitted_runs_until_break() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            let out = '';
            for (;;) {
              if (i > 2) {
                break;
              }
              out = out + String(i);
              i += 1;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "012")?;
    Ok(())
}

#[test]
fn for_loop_let_initializer_is_scoped_to_the_loop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = 'ok';
            for (let i = 0; i < 1; i++) {
              out = out + '!';
            }
            out = out + ':' + typeof i;
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok!:undefined")?;
    Ok(())
}

#[test]
fn for_loop_var_initializer_is_not_scoped_to_the_loop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            for (var i = 0; i < 1; i++) {}
            document.getElementById('result').textContent = typeof i + ':' + String(i);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "number:1")?;
    Ok(())
}

#[test]
fn if_allows_empty_statement_body() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            if (true);
            out += 'a';
            if (false);
            out += 'b';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ab")?;
    Ok(())
}

#[test]
fn labeled_empty_statement_is_accepted() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = 'ok';
            marker: ;
            out += '!';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok!")?;
    Ok(())
}

#[test]
fn labeled_continue_with_nested_for_loops_targets_outer_loop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            loop1: for (let i = 0; i < 3; i++) {
              loop2: for (let j = 0; j < 3; j++) {
                if (i === 1 && j === 1) {
                  continue loop1;
                }
                out = out + String(i) + String(j) + ',';
              }
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "00,01,02,10,20,21,22,")?;
    Ok(())
}

#[test]
fn labeled_break_with_nested_for_loops_exits_outer_loop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            loop1: for (let i = 0; i < 3; i++) {
              loop2: for (let j = 0; j < 3; j++) {
                if (i === 1 && j === 1) {
                  break loop1;
                }
                out = out + String(i) + String(j) + ',';
              }
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "00,01,02,10,")?;
    Ok(())
}

#[test]
fn multiple_labels_on_same_loop_are_equivalent() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            let i = 0;
            top: inner: while (i < 4) {
              i += 1;
              if (i === 2) {
                continue top;
              }
              if (i === 3) {
                break inner;
              }
              out = out + String(i);
            }
            out = out + 'x';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1x")?;
    Ok(())
}

#[test]
fn reserved_word_cannot_be_used_as_label() {
    let err = Harness::from_html("<script>default: ;</script>")
        .expect_err("reserved word label should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("reserved word")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn lexical_declaration_cannot_be_labeled() {
    let err = Harness::from_html("<script>label: const x = 1;</script>")
        .expect_err("labeled lexical declaration should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("lexical declaration cannot be labeled")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn labeled_async_or_generator_function_declaration_is_rejected() {
    let err = Harness::from_html("<script>label: async function f() {}</script>")
        .expect_err("labeled async function declaration should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("non-async")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>label: function* g() {}</script>")
        .expect_err("labeled generator function declaration should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("non-generator") || msg.contains("non-async"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn do_while_executes_body_at_least_once() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 10;
            do {
              i += 1;
            } while (i < 5);
            document.getElementById('result').textContent = String(i);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "11")?;
    Ok(())
}

#[test]
fn do_while_supports_single_statement_body() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let result = '';
            let i = 0;
            do i += 1; while (i < 5);
            result += String(i);
            document.getElementById('result').textContent = result;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "5")?;
    Ok(())
}

#[test]
fn do_while_continue_rechecks_condition() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            let i = 0;
            do {
              i += 1;
              if (i === 3) {
                continue;
              }
              out = out + String(i);
            } while (i < 5);
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1245")?;
    Ok(())
}

#[test]
fn switch_matches_case_and_supports_grouped_labels() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const expr = 'Papayas';
            let out = '';
            switch (expr) {
              case 'Oranges':
                out = 'oranges';
                break;
              case 'Mangoes':
              case 'Papayas':
                out = 'tropical';
                break;
              default:
                out = 'default';
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "tropical")?;
    Ok(())
}

#[test]
fn switch_fallthrough_and_default_in_middle_follow_js_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let first = '';
            switch (0) {
              case -1:
                first += 'n';
                break;
              case 0:
                first += '0';
              case 1:
                first += '1';
                break;
              default:
                first += 'd';
            }

            let second = '';
            switch (5) {
              case 2:
                second += '2';
                break;
              default:
                second += 'd';
              case 1:
                second += '1';
            }

            document.getElementById('result').textContent = first + ':' + second;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "01:d1")?;
    Ok(())
}

#[test]
fn switch_evaluates_case_expressions_lazily_after_match() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            result.setAttribute('data-log', '');

            switch (undefined) {
              case result.setAttribute('data-log', result.getAttribute('data-log') + '1'):
                result.setAttribute('data-log', result.getAttribute('data-log') + 'm');
                break;
              case result.setAttribute('data-log', result.getAttribute('data-log') + '2'):
                result.setAttribute('data-log', result.getAttribute('data-log') + 'n');
                break;
              default:
                result.setAttribute('data-log', result.getAttribute('data-log') + 'd');
            }

            result.textContent = result.getAttribute('data-log');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1m")?;
    Ok(())
}

#[test]
fn switch_uses_strict_equality_for_case_matching() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            switch (1) {
              case '1':
                out = 'string';
                break;
              case 1:
                out = 'number';
                break;
              default:
                out = 'default';
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "number")?;
    Ok(())
}

#[test]
fn switch_break_and_continue_interact_with_enclosing_loop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            for (let i = 0; i < 3; i++) {
              switch (i) {
                case 1:
                  continue;
                case 2:
                  break;
                default:
                  out += String(i);
              }
              out += 'x';
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0xx")?;
    Ok(())
}

#[test]
fn switch_labeled_break_exits_labeled_statement() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            outer: switch (1) {
              case 1:
                out += 'a';
                break outer;
              default:
                out += 'b';
            }
            out += 'c';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ac")?;
    Ok(())
}

#[test]
fn switch_rejects_multiple_default_clauses() {
    let err = Harness::from_html(
        "<script>switch (1) { default: break; case 1: break; default: break; }</script>",
    )
    .expect_err("switch with duplicate default should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("multiple default")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn switch_case_lexical_declarations_share_one_scope() {
    let err = Harness::from_html(
        "<script>switch (1) { case 1: let message = 'hello'; break; case 2: let message = 'hi'; break; }</script>",
    )
    .expect_err("switch case lexical declarations should share one scope");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("already been declared")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn labeled_break_exits_target_block() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            outerBlock: {
              innerBlock: {
                out = out + '1';
                break outerBlock;
                out = out + 'X';
              }
              out = out + '2';
            }
            out = out + '3';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "13")?;
    Ok(())
}

#[test]
fn break_with_unknown_label_reports_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            break missingLabel;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("break with unknown label should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("label not found: missingLabel")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn break_inside_nested_function_reports_outside_loop_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            while (i < 4) {
              if (i === 1) {
                (() => {
                  break;
                })();
              }
              i += 1;
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("break inside nested function should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("break statement outside of loop")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn labeled_continue_targets_outer_loop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            let i = 0;
            outer: while (i < 3) {
              let j = 0;
              while (j < 3) {
                j += 1;
                if (j === 2) {
                  i += 1;
                  continue outer;
                }
                out = out + String(i) + String(j);
              }
              i += 1;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "011121")?;
    Ok(())
}

#[test]
fn continue_with_unknown_label_reports_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            while (true) {
              continue missingLabel;
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("continue with unknown label should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("label not found: missingLabel")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn continue_with_non_iteration_label_reports_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            label: {
              for (let i = 0; i < 1; i++) {
                continue label;
              }
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("continue to non-loop label should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("does not denote an iteration statement"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn continue_inside_nested_function_reports_outside_loop_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let i = 0;
            while (i < 4) {
              if (i === 1) {
                (() => {
                  continue;
                })();
              }
              i += 1;
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("continue inside nested function should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("continue statement outside of loop")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn debugger_statement_is_noop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            debugger;
            out += 'a';
            debugger;
            out += 'b';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ab")?;
    Ok(())
}

#[test]
fn debugger_statement_rejects_trailing_tokens() {
    let err = Harness::from_html("<script>debugger extra;</script>")
        .expect_err("debugger statement with trailing tokens should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("unsupported debugger statement")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn expression_statement_runs_side_effects_and_discards_value() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let value = 1;
            value + 100;
            value = value + 1;
            (function () {
              value = value + 2;
            })();
            document.getElementById('result').textContent = String(value);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "4")?;
    Ok(())
}

#[test]
fn expression_statement_allows_parenthesized_object_literal() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = 'A';
            ({ foo: 1, bar: 2 });
            out = out + 'B';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn grouping_operator_controls_precedence_without_changing_operand_order() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          window.trace = '';
          function a() {
            window.trace += 'a';
            return 2;
          }
          function b() {
            window.trace += 'b';
            return 3;
          }
          function c() {
            window.trace += 'c';
            return 4;
          }

          const values = [
            1 + 2 * 3,
            1 + (2 * 3),
            (1 + 2) * 3,
            1 * 3 + 2 * 3,
            a() * (b() + c()),
            window.trace
          ];
          document.getElementById('result').textContent = values.join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "7:7:9:9:14:abc")?;
    Ok(())
}

#[test]
fn function_declaration_invocation_syntax_is_rejected() {
    let err = Harness::from_html("<script>function foo() { console.log('foo'); }();</script>")
        .expect_err("function declaration invocation form should fail");
    match err {
        Error::ScriptParse(msg) => assert!(!msg.is_empty()),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn grouped_iife_and_arrow_object_literal_expression_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const iifeValue = (function () {
            return 'ok';
          })();
          const f = () => ({ a: 1 });
          document.getElementById('result').textContent =
            iifeValue + ':' + String(f().a);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok:1")?;
    Ok(())
}

#[test]
fn grouping_operator_allows_integer_literal_property_access() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          document.getElementById('result').textContent = (1).toString();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn grouping_operator_can_avoid_return_asi_pitfall() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function broken(a, b) {
            return
              a + b;
          }
          function fixed(a, b) {
            return (
              a + b
            );
          }

          document.getElementById('result').textContent =
            String(fixed(1, 2)) + ':' + typeof broken(1, 2);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "3:undefined")?;
    Ok(())
}

#[test]
fn operator_precedence_examples_and_associativity_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const n = 1;
          const out = [
            3 + 4 * 5,
            4 * 3 ** 2,
            (window.chainA = window.chainB = 5),
            window.chainA,
            window.chainB,
            4 ** 3 ** 2,
            12 / 3 / 2,
            typeof n + 2,
          ];
          document.getElementById('result').textContent = out.join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "23:36:5:5:5:262144:2:number2")?;
    Ok(())
}

#[test]
fn operator_precedence_keeps_left_to_right_operand_evaluation_with_short_circuiting() -> Result<()>
{
    let html = r#"
        <p id='result'></p>
        <script>
          let evalOrder = '';
          function echo(name, num) {
            evalOrder += name;
            return num;
          }

          const expValue = echo('L', 4) ** echo('M', 3) ** echo('R', 2);
          const divValue = echo('l', 4) / echo('m', 3) ** echo('r', 2);

          let calls1 = '';
          function A1() { calls1 += 'A'; return false; }
          function B1() { calls1 += 'B'; return false; }
          function C1() { calls1 += 'C'; return true; }
          const short1 = C1() || B1() && A1();

          let calls2 = '';
          function A2() { calls2 += 'A'; return false; }
          function B2() { calls2 += 'B'; return false; }
          function C2() { calls2 += 'C'; return true; }
          const short2 = A2() && B2() || C2();

          document.getElementById('result').textContent =
            evalOrder + ':' + String(expValue) + ':' + String(divValue) + ':' +
            calls1 + ':' + String(short1) + ':' + calls2 + ':' + String(short2);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "LMRlmr:262144:0.4444444444444444:C:true:AC:true")?;
    Ok(())
}

#[test]
fn additive_expression_does_not_allow_unparenthesized_assignment_rhs() {
    let err = Harness::from_html("<script>let a = 1; 1 + a = 2;</script>")
        .expect_err("assignment with additive lhs should fail");
    match err {
        Error::ScriptParse(msg) => assert!(!msg.is_empty()),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn leading_parenthesis_without_semicolon_triggers_asi_hazard() {
    let err = Harness::from_html(
        r#"
        <script>
          const a = 1
          (1).toString()
        </script>
        "#,
    )
    .expect_err("line-leading parenthesis without semicolon should be hazardous");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("unsupported expression"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn leading_parenthesis_with_semicolon_avoids_asi_hazard() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const a = 1;
          ;(1).toString();
          document.getElementById('result').textContent = String(a);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn function_declaration_is_hoisted_and_callable_before_declaration() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const value = calcRectArea(5, 6);
            function calcRectArea(width, height) {
              return width * height;
            }
            document.getElementById('result').textContent = String(value);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "30")?;
    Ok(())
}

#[test]
fn function_expression_can_be_assigned_and_called() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const getRectArea = function (width, height) {
            return width * height;
          };

          document.getElementById('result').textContent = String(getRectArea(3, 4));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "12")?;
    Ok(())
}

#[test]
fn function_expression_is_not_hoisted_like_declaration() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const beforeType = typeof notHoisted;
            let callStatus = 'ok';
            try {
              notHoisted();
              callStatus = 'no-throw';
            } catch (error) {
              callStatus = 'threw';
            }

            var notHoisted = function () {
              return 'ready';
            };

            document.getElementById('result').textContent =
              beforeType + ':' + callStatus + ':' + notHoisted();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:threw:ready")?;
    Ok(())
}

#[test]
fn named_function_expression_supports_recursion_and_keeps_name_local() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const math = {
            factorial: function factorial(n) {
              if (n <= 1) {
                return 1;
              }
              return n * factorial(n - 1);
            },
          };

          document.getElementById('result').textContent =
            String(math.factorial(4)) + ':' + typeof factorial;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "24:undefined")?;
    Ok(())
}

#[test]
fn named_function_expression_name_binding_is_read_only() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const fn = function foo() {
            let status = 'no-throw';
            try {
              foo = 1;
            } catch (error) {
              status = 'threw';
            }
            return typeof foo + ':' + status;
          };

          document.getElementById('result').textContent = fn();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "function:threw")?;
    Ok(())
}

#[test]
fn standard_iife_function_expression_runs_immediately() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const value = (function (a, b) {
            return a + b;
          })(1, 2);

          document.getElementById('result').textContent = String(value);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn function_declaration_returns_undefined_without_return_statement() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            function noReturn() {}
            document.getElementById('result').textContent = typeof noReturn();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined")?;
    Ok(())
}

#[test]
fn return_statement_with_line_terminator_uses_asi() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            function viaAsi() {
              return
              123;
            }
            document.getElementById('result').textContent =
              typeof viaAsi() + ':' + String(viaAsi() === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:true")?;
    Ok(())
}

#[test]
fn return_statement_at_script_top_level_is_rejected() {
    match Harness::from_html("<script>return 1;</script>") {
        Err(Error::ScriptParse(msg)) => assert!(msg.contains("Illegal return statement")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("top-level return should be rejected"),
    }
}

#[test]
fn return_statement_nested_in_top_level_block_is_rejected() {
    match Harness::from_html("<script>if (true) { return 1; }</script>") {
        Err(Error::ScriptParse(msg)) => assert!(msg.contains("Illegal return statement")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("nested top-level return should be rejected"),
    }
}

#[test]
fn function_and_var_redeclaration_uses_var_initializer_value() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            var a = 1;
            function a() { return 2; }

            function b() { return 3; }
            var b = 4;

            document.getElementById('result').textContent =
              String(a) + ':' + String(b);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:4")?;
    Ok(())
}

#[test]
fn function_declaration_cannot_share_name_with_let_in_same_scope() {
    let err = Harness::from_html("<script>let value = 1; function value() { return 2; }</script>")
        .expect_err("function and let redeclaration should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("already been declared"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn block_function_declaration_is_scoped_to_the_block() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            {
              function inside() {
                return 'ok';
              }
              inside();
            }
            document.getElementById('result').textContent = typeof inside;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined")?;
    Ok(())
}

#[test]
fn class_expression_statement_without_name_is_rejected() {
    let err = Harness::from_html("<script>class {};</script>")
        .expect_err("unnamed class statement should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("class declaration requires a class name"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn let_bracket_statement_is_rejected() {
    let err = Harness::from_html("<script>var let = [1, 2, 3]; let[0] = 4;</script>")
        .expect_err("let[ should be rejected as a declaration parse");
    match err {
        Error::ScriptParse(msg) => assert!(!msg.is_empty()),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn module_export_declarations_and_lists_are_accepted() -> Result<()> {
    let html = r#"
        <script type='module'>
          export const base = 2;
          export function double(value) {
            return value * base;
          }
          export class Box {
            constructor(value) {
              this.value = value;
            }
          }
          export { base as "base-name", double as doubled };
          export {};

          function markDefault() {
            window.defaultEval = 'ok';
            return 1;
          }
          export default markDefault();

          window.moduleExportResult =
            String(double(3)) + ':' +
            String(new Box(7).value) + ':' +
            window.defaultEval;
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.moduleExportResult;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "6:7:ok")?;
    Ok(())
}

#[test]
fn module_export_list_can_reference_later_declarations() -> Result<()> {
    let html = r#"
        <script type='module'>
          export { laterValue };
          const laterValue = 9;
          window.laterExportValue = String(laterValue);
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.laterExportValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "9")?;
    Ok(())
}

#[test]
fn module_export_default_anonymous_function_is_accepted() -> Result<()> {
    let html = r#"
        <script type='module'>
          export default function () { return 42; };
          window.defaultAnonAccepted = 'yes';
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.defaultAnonAccepted;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "yes")?;
    Ok(())
}

#[test]
fn export_is_rejected_in_classic_script() {
    let err = Harness::from_html("<script>export const value = 1;</script>")
        .expect_err("export in classic script should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("module scripts")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn export_from_is_not_supported_yet() {
    let err =
        Harness::from_html("<script type='module'>export { value } from './mod.js';</script>")
            .expect_err("export-from should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("not supported") || msg.contains("unsupported"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn module_import_forms_and_hoisting_work() -> Result<()> {
    let html = r#"
        <script type='module'>
          window.hoistedValue = String(triple(3));
          import defaultValue, { named as renamed, default as alsoDefault } from "data:text/javascript,export%20const%20named%20%3D%207%3B%20export%20default%205%3B";
          import * as ns from "data:text/javascript,export%20const%20value%20%3D%2011%3B%20export%20default%20%22ns%22%3B";
          import { triple } from "data:text/javascript,export%20function%20triple(value)%20%7B%20return%20value%20*%203%3B%20%7D";
          window.moduleImportResult =
            window.hoistedValue + ':' +
            String(defaultValue) + ':' +
            String(renamed) + ':' +
            String(alsoDefault) + ':' +
            String(ns.value) + ':' +
            String(ns.default);
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.moduleImportResult;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "9:5:7:5:11:ns")?;
    Ok(())
}

#[test]
fn module_side_effect_import_runs() -> Result<()> {
    let html = r#"
        <script type='module'>
          import "data:text/javascript,window.sideEffectRan%20%3D%20%22yes%22%3B";
          window.sideEffectImportResult = String(window.sideEffectRan);
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.sideEffectImportResult;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "yes")?;
    Ok(())
}

#[test]
fn module_import_json_with_attribute_works() -> Result<()> {
    let html = r#"
        <script type='module'>
          import data from "data:application/json,%7B%22answer%22%3A42%7D" with { type: "json" };
          window.importedJsonValue = String(data.answer);
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.importedJsonValue;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "42")?;
    Ok(())
}

#[test]
fn import_meta_url_exposes_module_url_and_query_params() -> Result<()> {
    let html = r#"
        <script type='module'>
          const currentUrl = import.meta.url;
          const queryValue = new URL(import.meta.url).searchParams.get("someURLInfo");
          window.importMetaInfo = currentUrl + ':' + String(queryValue);
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.importMetaInfo;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url(
        "https://app.local/modules/index.html?someURLInfo=5#hash",
        html,
    )?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "https://app.local/modules/index.html?someURLInfo=5#hash:5",
    )?;
    Ok(())
}

#[test]
fn import_meta_resolve_resolves_against_current_module() -> Result<()> {
    let html = r#"
        <script type='module'>
          window.importMetaResolved = import.meta.resolve("./utils/helper.js");
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.importMetaResolved;
          });
        </script>
        "#;

    let mut h = Harness::from_html_with_url("https://app.local/app/main/page.html", html)?;
    h.click("#run")?;
    h.assert_text("#result", "https://app.local/app/main/utils/helper.js")?;
    Ok(())
}

#[test]
fn import_meta_url_uses_imported_module_specifier() -> Result<()> {
    let html = r#"
        <script type='module'>
          import importedModuleUrl from "data:text/javascript,export%20default%20import.meta.url%3B";
          window.importedModuleMetaUrl = importedModuleUrl;
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.importedModuleMetaUrl;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#result",
        "data:text/javascript,export%20default%20import.meta.url%3B",
    )?;
    Ok(())
}

#[test]
fn property_access_named_import_meta_is_not_import_meta_syntax() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          const holder = { import: { meta: 7 } };
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = String(holder.import.meta);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "7")?;
    Ok(())
}

#[test]
fn import_meta_is_rejected_in_classic_script() {
    let err = Harness::from_html("<script>window.importMeta = import.meta.url;</script>")
        .expect_err("import.meta in classic script should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("module scripts")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn dynamic_import_works_in_classic_script_with_computed_specifier() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', async () => {
            const specifier =
              "data:text/javascript,export%20default%204%3B%20export%20const%20named%20%3D%207%3B";
            const ns = await import(specifier);
            document.getElementById('result').textContent =
              String(ns.default) + ':' + String(ns.named);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "4:7")?;
    Ok(())
}

#[test]
fn dynamic_import_supports_side_effect_only_usage() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', async () => {
            await import("data:text/javascript,window.dynamicImportSideEffect%20%3D%20%22yes%22%3B");
            document.getElementById('result').textContent = String(window.dynamicImportSideEffect);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "yes")?;
    Ok(())
}

#[test]
fn dynamic_import_supports_json_import_attributes() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', async () => {
            const ns = await import("data:application/json,%7B%22answer%22%3A42%7D", {
              with: { type: "json" },
            });
            document.getElementById('result').textContent = String(ns.default.answer);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "42")?;
    Ok(())
}

#[test]
fn dynamic_import_rejection_is_async_not_sync_throw() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let syncState = 'not-thrown';
            try {
              import('/missing-dynamic-module.js')
                .then(() => {
                  document.getElementById('result').textContent = syncState + ':resolved';
                })
                .catch((err) => {
                  const isRejected = String(err).includes('module source mock not found');
                  document.getElementById('result').textContent =
                    syncState + ':' + (isRejected ? 'rejected' : 'other');
                });
            } catch (err) {
              syncState = 'thrown';
              document.getElementById('result').textContent = syncState;
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "not-thrown:rejected")?;
    Ok(())
}

#[test]
fn dynamic_import_reuses_cached_module_namespace_object() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', async () => {
            const specifier = "data:text/javascript,export%20const%20value%20%3D%201%3B";
            const first = await import(specifier);
            const second = await import(specifier);
            document.getElementById('result').textContent =
              String(first === second) + ':' + Object.keys(first).join(',');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "true:value")?;
    Ok(())
}

#[test]
fn dynamic_import_expression_statement_is_accepted_in_module_script() -> Result<()> {
    let html = r#"
        <script type='module'>
          import("data:text/javascript,window.dynamicModuleImportRan%20%3D%20%22ok%22%3B")
            .then(() => {
              window.dynamicModuleImportResult = String(window.dynamicModuleImportRan);
            });
        </script>
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            document.getElementById('result').textContent = window.dynamicModuleImportResult;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn import_is_rejected_in_classic_script() {
    let err = Harness::from_html("<script>import value from './dep.js';</script>")
        .expect_err("import in classic script should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("top level") || msg.contains("module")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn import_inside_block_is_rejected_in_module_script() {
    let err = Harness::from_html(
        "<script type='module'>{ import value from \"data:text/javascript,export%20default%201%3B\"; }</script>",
    )
    .expect_err("nested import should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("top level")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn imported_binding_cannot_be_reassigned() {
    let err = Harness::from_html(
        "<script type='module'>import value from \"data:text/javascript,export%20default%201%3B\"; value = 2;</script>",
    )
    .expect_err("reassigning import should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("constant variable")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn import_named_binding_requires_existing_export() {
    let err = Harness::from_html(
        "<script type='module'>import { missing } from \"data:text/javascript,export%20const%20present%20%3D%201%3B\";</script>",
    )
    .expect_err("missing named export should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("does not provide an export named")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn unsupported_import_attribute_is_rejected() {
    let err = Harness::from_html(
        "<script type='module'>import data from \"data:application/json,%7B%7D\" with { mode: \"json\" };</script>",
    )
    .expect_err("unsupported import attribute should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("unsupported import attribute")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn foreach_supports_nested_if_else_and_event_variable() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <ul>
          <li class='item'>A</li>
          <li class='item'>B</li>
          <li class='item'>C</li>
        </ul>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', (event) => {
            document.querySelectorAll('.item').forEach((item, idx) => {
              if (idx === 1) {
                if (event.target.id === 'btn') {
                  item.classList.add('mid');
                } else {
                  item.classList.add('other');
                }
              } else {
                item.classList.add('edge');
              }
            });
            document.getElementById('result').textContent =
              document.querySelectorAll('.edge').length + ':' +
              document.querySelectorAll('.mid').length + ':' +
              event.currentTarget.id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:1:btn")?;
    Ok(())
}

#[test]
fn if_without_braces_with_else_on_next_statement_works() -> Result<()> {
    let html = r#"
        <input id='agree' type='checkbox'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            if (document.getElementById('agree').checked) document.getElementById('result').textContent = 'yes';
            else document.getElementById('result').textContent = 'no';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "no")?;
    h.set_checked("#agree", true)?;
    h.click("#btn")?;
    h.assert_text("#result", "yes")?;
    Ok(())
}

#[test]
fn if_else_if_chain_runs_first_matching_branch() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const x = 20;
            let out = '';
            if (x > 50) {
              out = 'large';
            } else if (x > 5) {
              out = 'middle';
            } else {
              out = 'small';
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "middle")?;
    Ok(())
}

#[test]
fn dangling_else_binds_to_the_closest_if_statement() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            let out = 'none';
            if (1 === 1)
              if (3 === 2)
                out = 'a is 1 and b is 2';
              else
                out = 'a is not 1';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "a is not 1")?;
    Ok(())
}

#[test]
fn if_block_and_following_statement_without_semicolon_are_split() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let text = '';
            if (true) {
              text = 'A';
            }
            text += 'B';
            document.getElementById('result').textContent = text;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn while_block_and_following_statement_without_semicolon_are_split() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let count = 0;
            let n = 0;
            while (n < 2) {
              count = count + 1;
              n = n + 1;
            }
            count = count + 10;
            document.getElementById('result').textContent = count;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12")?;
    Ok(())
}

#[test]
fn for_block_and_following_statement_without_semicolon_are_split() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let sum = 0;
            for (let i = 0; i < 3; i = i + 1) {
              sum = sum + i;
            } sum = sum + 10;
            document.getElementById('result').textContent = sum;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "13")?;
    Ok(())
}

#[test]
fn if_block_and_following_statement_without_space_are_split() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let text = '';
            if (true) {
              text = 'A';
            } if (true) {
              text = text + 'B';
            }
            document.getElementById('result').textContent = text;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn standalone_block_statement_groups_multiple_statements() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = 'A';
            {
              out = out + 'B';
              out = out + 'C';
            }
            out = out + 'D';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ABCD")?;
    Ok(())
}

#[test]
fn empty_block_statement_is_a_noop() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = 'A';
            {}
            out = out + 'B';
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn var_declared_inside_block_updates_containing_scope() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            var x = 1;
            {
              var x = 2;
            }
            document.getElementById('result').textContent = String(x);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2")?;
    Ok(())
}

#[test]
fn var_is_hoisted_and_reads_as_undefined_before_initializer() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            function readBeforeInit() {
              const beforeType = typeof bar;
              const beforeIsUndefined = String(bar === undefined);
              var bar = 111;
              return beforeType + ':' + beforeIsUndefined + ':' + String(bar);
            }
            document.getElementById('result').textContent = readBeforeInit();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:true:111")?;
    Ok(())
}

#[test]
fn var_redeclaration_without_initializer_preserves_existing_value() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            var a = 2;
            var a;
            document.getElementById('result').textContent = String(a);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2")?;
    Ok(())
}

#[test]
fn var_in_unexecuted_branch_is_still_hoisted() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            if (false) {
              var hidden = 1;
            }
            document.getElementById('result').textContent =
              typeof hidden + ':' + String(hidden === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:true")?;
    Ok(())
}

#[test]
fn var_initializer_can_read_later_var_as_undefined() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            var x = y, y = 'A';
            document.getElementById('result').textContent = String(x) + ':' + y;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:A")?;
    Ok(())
}

#[test]
fn let_declared_inside_block_does_not_override_outer_binding() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            var x = 1;
            let y = 1;
            if (true) {
              var x = 2;
              let y = 2;
            }
            document.getElementById('result').textContent = x + ':' + y;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:1")?;
    Ok(())
}

#[test]
fn const_declared_inside_block_does_not_override_outer_binding() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const c = 1;
            {
              const c = 2;
            }
            document.getElementById('result').textContent = String(c);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn for_loop_post_increment_with_function_callback_works() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', function() {
            let sum = 0;
            for (let i = 0; i < 3; i++) {
              sum = sum + i;
            }
            document.getElementById('result').textContent = sum;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn try_catch_catches_runtime_error_and_binds_exception() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = 'init';
            try {
              nonExistentFunction();
              out = 'not-caught';
            } catch (error) {
              out = typeof error + ':' + (error ? 'y' : 'n');
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "string:y")?;
    Ok(())
}

#[test]
fn try_catch_finally_and_rethrow_behavior_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = '';
            try {
              try {
                throw 'oops';
              } catch (ex) {
                out = out + 'inner:' + ex;
                throw ex;
              } finally {
                out = out + ':finally';
              }
            } catch (ex) {
              out = out + ':outer:' + ex;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "inner:oops:finally:outer:oops")?;
    Ok(())
}

#[test]
fn try_finally_runs_without_catch_and_finally_return_masks_try_return() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function doIt() {
            try {
              return 1;
            } finally {
              return 2;
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            let out = 'start';
            try {
              try {
                throw 'boom';
              } finally {
                out = out + ':inner-finally';
              }
            } catch (e) {
              out = out + ':outer-catch:' + e;
            }
            document.getElementById('result').textContent = doIt() + ':' + out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2:start:inner-finally:outer-catch:boom")?;
    Ok(())
}

#[test]
fn catch_without_binding_and_pattern_binding_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function isValidJSON(text) {
            try {
              JSON.parse(text);
              return true;
            } catch {
              return false;
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            let out = isValidJSON('{"a":1}') + ':' + isValidJSON('{bad}');
            try {
              throw { name: 'TypeError', message: 'oops' };
            } catch ({ name, message }) {
              out = out + ':' + name + ':' + message;
            }
            try {
              throw ['A', 'B'];
            } catch ([first, second]) {
              out = out + ':' + first + second;
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:TypeError:oops:AB")?;
    Ok(())
}

#[test]
fn catch_binding_is_writable_and_does_not_leak_outside_catch_scope() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let error = 'outer';
            let out = '';
            try {
              throw 'inner';
            } catch (error) {
              error = error + '!';
              out = error;
            }
            document.getElementById('result').textContent = out + ':' + error;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "inner!:outer")?;
    Ok(())
}

#[test]
fn catch_binding_name_conflicts_with_var_and_const_declarations() {
    match Harness::from_html(
        "<script>try { throw {name:'x'}; } catch ({ name }) { var name = 'y'; }</script>",
    ) {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("already been declared")),
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("catch binding conflict with var should fail"),
    }

    match Harness::from_html(
        "<script>try { throw {name:'x'}; } catch ({ name }) { const name = 'y'; }</script>",
    ) {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("already been declared")),
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("catch binding conflict with const should fail"),
    }
}

#[test]
fn var_can_share_name_with_simple_catch_identifier_binding() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            try {
              throw 'boom';
            } catch (e) {
              var e = 2;
            }
            document.getElementById('result').textContent =
              typeof e + ':' + String(e === undefined);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:true")?;
    Ok(())
}

#[test]
fn try_requires_block_body_and_catch_or_finally() {
    match Harness::from_html("<script>try doSomething(); catch (e) {}</script>") {
        Err(Error::ScriptParse(msg)) => assert!(msg.contains("expected '{'")),
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("try without block body should fail"),
    }

    match Harness::from_html("<script>try {}</script>") {
        Err(Error::ScriptParse(msg)) => assert!(msg.contains("requires catch or finally")),
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("try without catch/finally should fail"),
    }
}

#[test]
fn finally_return_masks_throw_from_catch() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function doIt() {
            try {
              throw 'boom';
            } catch (e) {
              throw e;
            } finally {
              return 'masked';
            }
          }
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = doIt();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "masked")?;
    Ok(())
}

#[test]
fn throw_new_error_is_supported() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            let out = 'init';
            try {
              throw new Error('boom');
            } catch (e) {
              out = String(e);
            }
            document.getElementById('result').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "boom")?;
    Ok(())
}

#[test]
fn throw_statement_with_line_terminator_is_rejected() {
    let html = r#"
        <script>
          function fail() {
            throw
            new Error('boom');
          }
          fail();
        </script>
        "#;

    match Harness::from_html(html) {
        Err(Error::ScriptParse(msg)) => {
            assert!(msg.contains("throw statement requires an operand"))
        }
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("throw with newline should be rejected"),
    }
}

#[test]
fn throw_statement_with_line_comment_then_line_terminator_is_rejected() {
    let html = r#"
        <script>
          function fail() {
            throw // force ASI
            new Error('boom');
          }
          fail();
        </script>
        "#;

    match Harness::from_html(html) {
        Err(Error::ScriptParse(msg)) => {
            assert!(msg.contains("throw statement requires an operand"))
        }
        Err(other) => panic!("unexpected error: {other:?}"),
        Ok(_) => panic!("throw with line-comment terminator should be rejected"),
    }
}

#[test]
fn throw_statement_allows_parenthesized_expression() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          try {
            throw (
              new Error('boom')
            );
          } catch (e) {
            document.getElementById('result').textContent = String(e);
          }
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "boom")?;
    Ok(())
}

#[test]
fn promise_then_function_callback_runs_as_microtask() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', function() {
            const result = document.getElementById('result');
            result.textContent = 'A';
            Promise.resolve().then(function() {
              result.textContent = result.textContent + 'P';
            });
            setTimeout(function() {
              result.textContent = result.textContent + 'T';
            }, 0);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AP")?;
    h.flush()?;
    h.assert_text("#result", "APT")?;
    Ok(())
}

#[test]
fn promise_direct_then_chain_parses_and_runs() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.resolve('A')
              .then((value) => value + 'B')
              .then((value) => {
                result.textContent = value;
              })
              .catch((reason) => {
                result.textContent = 'ERR:' + reason;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AB")?;
    Ok(())
}

#[test]
fn promise_constructor_resolves_via_timer() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            new Promise((resolve) => {
              setTimeout(() => {
                resolve('done');
              }, 0);
            }).then((value) => {
              result.textContent = value;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.flush()?;
    h.assert_text("#result", "done")?;
    Ok(())
}

#[test]
fn promise_catch_and_finally_chain_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.reject('E')
              .catch((reason) => {
                result.textContent = reason;
                return 'recovered';
              })
              .finally(() => {
                result.textContent = result.textContent + 'F';
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "EF")?;
    Ok(())
}

#[test]
fn promise_finally_waits_for_returned_promise() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.resolve('A')
              .finally(() => {
                return new Promise((resolve) => {
                  setTimeout(() => resolve('x'), 0);
                });
              })
              .then((value) => {
                result.textContent = value;
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "")?;
    h.flush()?;
    h.assert_text("#result", "A")?;
    Ok(())
}

#[test]
fn promise_with_resolvers_can_be_used_externally() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const bag = Promise.withResolvers();
            const resolveBag = bag.resolve;
            const rejectBag = bag.reject;

            bag.promise
              .then((value) => {
                result.textContent = 'ok:' + value;
              })
              .catch((reason) => {
                result.textContent = 'ng:' + reason;
              });

            resolveBag('A');
            rejectBag('B');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok:A")?;
    Ok(())
}

#[test]
fn promise_all_resolves_values_in_input_order() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.all([Promise.resolve('A'), 2]).then((values) => {
              result.textContent = values.join(',');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A,2")?;
    Ok(())
}

#[test]
fn promise_all_settled_returns_outcome_objects() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.allSettled([Promise.resolve('A'), Promise.reject('B')]).then((values) => {
              result.textContent = JSON.stringify(values);
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        r#"[{"status":"fulfilled","value":"A"},{"status":"rejected","reason":"B"}]"#,
    )?;
    Ok(())
}

#[test]
fn promise_any_rejects_with_aggregate_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.any([Promise.reject('E1'), Promise.reject('E2')]).catch((reason) => {
              const errors = reason.errors;
              result.textContent = reason.name + ':' + errors.join(',');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "AggregateError:E1,E2")?;
    Ok(())
}

#[test]
fn promise_race_settles_with_first_settled_value() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.race([
              Promise.resolve('fast'),
              new Promise((resolve) => {
                setTimeout(() => resolve('slow'), 0);
              })
            ]).then((value) => {
              result.textContent = value;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "fast")?;
    Ok(())
}

#[test]
fn promise_try_wraps_sync_return_and_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            Promise.try(() => 'ok')
              .then((value) => {
                result.textContent = value;
                return Promise.try(() => missingVar);
              })
              .catch((reason) => {
                if (reason.includes('unknown variable')) {
                  result.textContent = result.textContent + ':caught';
                } else {
                  result.textContent = 'unexpected:' + reason;
                }
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "ok:caught")?;
    Ok(())
}

#[test]
fn promise_constructor_requires_new_keyword() {
    let err = Harness::from_html("<script>Promise(() => {});</script>")
        .expect_err("Promise without new should throw");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("Promise constructor must be called with new"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn arrow_function_value_can_be_called() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const result = document.getElementById('result');
            const fn = (value) => {
              result.textContent = value;
            };
            fn('A');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "A")?;
    Ok(())
}

#[test]
fn iife_arrow_function_expression_can_be_called() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          (() => {
            document.getElementById('result').textContent = 'ok';
          })();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn default_function_parameters_apply_for_missing_or_undefined() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function multiply(a, b = 1) {
            return a * b;
          }

          function test(num = 1) {
            return typeof num;
          }

          document.getElementById('result').textContent =
            multiply(5, 2) + ':' +
            multiply(5) + ':' +
            multiply(5, undefined) + ':' +
            test() + ':' +
            test(undefined) + ':' +
            test('') + ':' +
            test(null);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "10:5:5:number:number:string:object")?;
    Ok(())
}

#[test]
fn default_function_parameters_are_evaluated_left_to_right_at_call_time() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function greet(name, greeting, message = `${greeting} ${name}`) {
            return message;
          }

          function append(value, array = []) {
            array.push(value);
            return array.length;
          }

          const exprFn = function(a = 2, b = a + 1) {
            return a + b;
          };

          const arrowFn = (a, b = a + 5) => {
            return a + ':' + b;
          };

          document.getElementById('result').textContent =
            greet('David', 'Hi') + ':' +
            greet('David', 'Hi', 'Happy Birthday!') + ':' +
            append(1) + ':' +
            append(2) + ':' +
            exprFn(undefined, undefined) + ':' +
            exprFn(5) + ':' +
            arrowFn(7);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "Hi David:Happy Birthday!:1:1:5:11:7:12")?;
    Ok(())
}

#[test]
fn default_parameter_initializers_can_read_this_and_arguments() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function withDefaults(a, b = this.value, c = arguments[0], d = this.value + ':' + arguments.length) {
            return [String(a), String(b), String(c), d].join('|');
          }

          const holder = {
            value: 'ctx',
            withDefaults,
          };

          document.getElementById('result').textContent =
            holder.withDefaults('x') + ';' +
            holder.withDefaults(undefined, 'y');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "x|ctx|x|ctx:1;undefined|y|undefined|ctx:2")?;
    Ok(())
}

#[test]
fn default_parameter_scope_is_separate_from_function_body_var_scope() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function f(a, b = () => String(a)) {
            var a = 1;
            return b();
          }

          document.getElementById('result').textContent = f() + ':' + f(5);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "undefined:5")?;
    Ok(())
}

#[test]
fn default_parameter_initializer_cannot_access_function_body_var_bindings() {
    let err = Harness::from_html(
        "<script>function f(a = seed) { var seed = 3; return a; } f();</script>",
    )
    .expect_err("default parameter initializer should not see function body var bindings");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("unknown variable: seed")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn object_literal_getter_reads_latest_value_and_assignment_does_not_override_it() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const obj = {
            log: ['a', 'b', 'c'],
            get latest() {
              return this.log[this.log.length - 1];
            },
          };

          const before = obj.latest;
          obj.latest = 'ignored';
          obj.log.push('d');
          document.getElementById('result').textContent = before + ':' + obj.latest;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "c:d")?;
    Ok(())
}

#[test]
fn object_literal_computed_getter_is_used_by_values_and_entries() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let calls = 0;
          const key = 'foo';
          const obj = {
            get [key]() {
              calls += 1;
              return 'bar';
            },
          };

          const direct = obj.foo;
          const keys = Object.keys(obj).join(',');
          const values = Object.values(obj).join(',');
          const entries = Object.entries(obj).map((entry) => entry[0] + '=' + entry[1]).join(',');
          document.getElementById('result').textContent =
            direct + '|' + keys + '|' + values + '|' + entries + '|' + String(calls);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "bar|foo|bar|foo=bar|3")?;
    Ok(())
}

#[test]
fn class_getter_reads_instance_state_and_assignment_is_ignored_without_setter() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Bucket {
            constructor() {
              this.items = ['x', 'y'];
            }
            get latest() {
              return this.items[this.items.length - 1];
            }
          }

          const bucket = new Bucket();
          const before = bucket.latest;
          bucket.latest = 'ignored';
          bucket.items.push('z');
          document.getElementById('result').textContent = before + ':' + bucket.latest;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "y:z")?;
    Ok(())
}

#[test]
fn getter_syntax_rejects_parameters() {
    let err = Harness::from_html("<script>const obj = { get value(x) { return x; } };</script>")
        .expect_err("object literal getter with parameters should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("getter") && msg.contains("parameters")),
        other => panic!("unexpected error: {other:?}"),
    }

    let err = Harness::from_html("<script>class C { get value(x) { return x; } }</script>")
        .expect_err("class getter with parameters should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("getter") && msg.contains("parameters")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn object_literal_setter_updates_log_and_property_read_is_undefined() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const language = {
            set current(name) {
              this.log.push(name);
            },
            log: [],
          };

          language.current = 'EN';
          language.current = 'FA';

          document.getElementById('result').textContent =
            String(language.current) + '|' + language.log.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "undefined|EN,FA")?;
    Ok(())
}

#[test]
fn object_literal_computed_setter_updates_target_value() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const expr = 'foo';
          const obj = {
            baz: 'bar',
            set [expr](v) {
              this.baz = v;
            },
          };

          obj.foo = 'baz';
          document.getElementById('result').textContent =
            obj.baz + ':' + String(obj.foo);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "baz:undefined")?;
    Ok(())
}

#[test]
fn class_setter_and_getter_update_instance_state() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class ClassWithGetSet {
            constructor() {
              this._msg = 'hello world';
            }
            get msg() {
              return this._msg;
            }
            set msg(x) {
              this._msg = `hello ${x}`;
            }
          }

          const instance = new ClassWithGetSet();
          const before = instance.msg;
          instance.msg = 'cake';
          const after = instance.msg;
          document.getElementById('result').textContent = before + ':' + after;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "hello world:hello cake")?;
    Ok(())
}

#[test]
fn setter_syntax_rejects_invalid_parameter_counts() {
    for source in [
        "<script>const obj = { set value() {} };</script>",
        "<script>const obj = { set value(a, b) {} };</script>",
        "<script>const obj = { set value(...rest) {} };</script>",
        "<script>class C { set value() {} }</script>",
        "<script>class C { set value(a, b) {} }</script>",
        "<script>class C { set value(...rest) {} }</script>",
    ] {
        let err =
            Harness::from_html(source).expect_err("invalid setter parameter arity should fail");
        match err {
            Error::ScriptParse(msg) => assert!(msg.contains("setter")),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn object_method_definition_supports_this_and_computed_names() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const methodName = 'FOO';
          const obj = {
            a: 'bar',
            b() {
              return this.a;
            },
            [methodName]() {
              return 2;
            },
          };
          document.getElementById('result').textContent = obj.b() + ':' + String(obj.FOO());
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "bar:2")?;
    Ok(())
}

#[test]
fn object_initializer_supports_numeric_literal_keys() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const obj = {
            1: 'one',
            1.5: 'float',
            0x10: 'hex',
            1e2: 'exp',
          };
          document.getElementById('result').textContent =
            obj[1] + ':' + obj['1.5'] + ':' + obj[16] + ':' + obj[100] + ':' + String(obj['1e2']);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "one:float:hex:exp:undefined")?;
    Ok(())
}

#[test]
fn object_initializer_proto_setter_only_applies_to_colon_form() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const proto = { marker: 'proto' };
          const __proto__ = 'shorthand';
          const obj = {
            __proto__: proto,
            __proto__,
            ['__proto__']: 'computed',
            __proto__() {
              return 'method';
            },
          };

          document.getElementById('result').textContent =
            String(Object.hasOwn(obj, '__proto__')) + ':' + String(obj.marker) + ':' + String(obj.__proto__());
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true:proto:method")?;
    Ok(())
}

#[test]
fn object_initializer_rejects_duplicate_proto_setters() {
    let err =
        Harness::from_html("<script>const obj = { __proto__: {}, \"__proto__\": null };</script>")
            .expect_err("duplicate __proto__ setters should fail");

    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("duplicate"));
            assert!(msg.contains("__proto__"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn object_initializer_proto_setter_ignores_non_object_values() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const obj = { __proto__: 1 };
          document.getElementById('result').textContent = String(Object.hasOwn(obj, '__proto__'));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "false")?;
    Ok(())
}

#[test]
fn object_method_definition_variants_generator_async_and_async_generator_work() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const obj = {
              *g() {
                yield 1;
                yield 2;
              },
              async f() {
                return 3;
              },
              async *ag() {
                yield 4;
                yield 5;
              },
            };

            const it = obj.g();
            const ait = obj.ag();
            let out = String(it.next().value) + ',' + String(it.next().value);
            obj.f()
              .then((v) => {
                out = out + '|' + String(v);
                return ait.next();
              })
              .then((step) => {
                out = out + '|' + String(step.value);
                return ait.next();
              })
              .then((step) => {
                document.getElementById('result').textContent = out + ',' + String(step.value);
              });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "1,2|3|4,5")?;
    Ok(())
}

#[test]
fn method_definitions_are_not_constructable_and_do_not_expose_prototype() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const obj = {
            method() {},
            regular: function () {},
          };
          const methodRef = obj.method;
          const regularRef = obj.regular;
          let blocked = false;
          try {
            new methodRef();
          } catch (error) {
            blocked = String(error).includes('not a constructor');
          }
          document.getElementById('result').textContent =
            String(blocked) + ':' + typeof methodRef.prototype + ':' + typeof regularRef.prototype;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true:undefined:object")?;
    Ok(())
}

#[test]
fn class_method_definitions_are_not_constructable_and_do_not_expose_prototype() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class C {
            method() {
              return 'ok';
            }
          }

          const c = new C();
          const methodRef = c.method;
          let blocked = false;
          try {
            new methodRef();
          } catch (error) {
            blocked = String(error).includes('not a constructor');
          }
          document.getElementById('result').textContent =
            c.method() + ':' + String(blocked) + ':' + typeof methodRef.prototype;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok:true:undefined")?;
    Ok(())
}

#[test]
fn default_parameters_support_destructured_binding_defaults() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function preFilledArray([x = 1, y = 2] = []) {
            return x + y;
          }

          function preFilledObject({ z = 3 } = {}) {
            return z;
          }

          document.getElementById('result').textContent = [
            preFilledArray(),
            preFilledArray([]),
            preFilledArray([2]),
            preFilledArray([2, 3]),
            preFilledObject(),
            preFilledObject({}),
            preFilledObject({ z: 2 }),
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "3:3:4:5:3:3:2")?;
    Ok(())
}

#[test]
fn arrow_function_parameters_support_rest_and_destructuring() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const sum = (a, b, ...rest) => a + b + rest[0] + rest[1];
          const pick = ({ x, y: z }) => x + ':' + z;
          const pair = ([m, n] = [9, 8]) => m + ':' + n;
          document.getElementById('result').textContent =
            sum(1, 2, 3, 4) + '|' +
            pick({ x: 'A', y: 'B' }) + '|' +
            pair() + '|' +
            pair([5, 6]);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "10|A:B|9:8|5:6")?;
    Ok(())
}

#[test]
fn rest_parameters_collect_extra_arguments_and_keep_arguments_full() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function myFun(a, b, ...manyMoreArgs) {
            return a + ':' + b + ':' + manyMoreArgs.join(',');
          }

          function inspect(a, b, ...rest) {
            a = 'changed';
            return [
              arguments.length,
              rest.length,
              rest[0],
              rest[1],
              arguments[0],
              arguments[2],
            ].join(',');
          }

          document.getElementById('result').textContent =
            myFun('one', 'two', 'three', 'four', 'five', 'six') + '|' +
            myFun('one', 'two', 'three') + '|' +
            myFun('one', 'two') + '|' +
            myFun('one') + '|' +
            inspect('A', 'B', 'C', 'D');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "one:two:three,four,five,six|one:two:three|one:two:|one:undefined:|4,2,C,D,A,C",
    )?;
    Ok(())
}

#[test]
fn rest_parameters_support_destructuring_patterns() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function ignoreFirst(...[, b, c]) {
            return b + c;
          }

          function restObjectLength(...{ length }) {
            return length;
          }

          const firstTwo = (...[first, second]) => first + ':' + second;

          document.getElementById('result').textContent =
            String(ignoreFirst(1, 2, 3, 4)) + '|' +
            String(restObjectLength('x', 'y', 'z')) + '|' +
            firstTwo('a', 'b', 'c');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "5|3|a:b")?;
    Ok(())
}

#[test]
fn rest_parameter_length_property_ignores_rest_and_default_tail() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function withRest(a, b, ...rest) {}
          function withDefault(a, b = 1, c) {}
          function onlyRest(...args) {}
          const arrow = (x, ...tail) => x;

          document.getElementById('result').textContent =
            [withRest.length, withDefault.length, onlyRest.length, arrow.length].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "2:1:0:1")?;
    Ok(())
}

#[test]
fn rest_parameter_syntax_restrictions_are_enforced() {
    for source in [
        "<script>function wrong1(...one, ...wrong) {}</script>",
        "<script>function wrong2(...wrong, arg2, arg3) {}</script>",
        "<script>function wrong3(...wrong,) {}</script>",
        "<script>function wrong4(...wrong = []) {}</script>",
    ] {
        let err = Harness::from_html(source).expect_err("invalid rest syntax should fail");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains("unsupported function parameters"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn arguments_object_exposes_values_length_and_type() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function func1(a, b, c) {
            return [
              arguments[0],
              arguments[1],
              arguments[2],
              arguments.length,
              typeof arguments,
            ].join(':');
          }

          document.getElementById('result').textContent = func1(1, 2, 3);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:2:3:3:object")?;
    Ok(())
}

#[test]
fn arguments_object_syncs_with_simple_parameters() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function sync(a) {
            const args = arguments;
            args[0] = 88;
            a = 77;
            return [a, args[0], arguments[0]].join(':');
          }

          document.getElementById('result').textContent = sync(10);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "77:77:77")?;
    Ok(())
}

#[test]
fn arguments_object_does_not_sync_for_complex_parameters() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function withDefault(a = 55) {
            arguments[0] = 99;
            const first = a;
            a = 12;
            return first + ':' + String(arguments[0]);
          }

          function withRest(a, ...rest) {
            a = 42;
            return String(arguments[0]) + ':' + String(rest.length);
          }

          function withDestructure([x]) {
            x = 8;
            return String(arguments[0][0]);
          }

          document.getElementById('result').textContent =
            withDefault(10) + '|' +
            withRest(5, 6, 7) + '|' +
            withDestructure([3]);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "10:99|5:2|3")?;
    Ok(())
}

#[test]
fn arguments_object_is_iterable_and_arrow_uses_outer_arguments_binding() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function sum() {
            let total = 0;
            for (const arg of arguments) {
              total += arg;
            }
            return total;
          }

          function outer(n) {
            const f = () => arguments[0] + n;
            return f();
          }

          document.getElementById('result').textContent =
            String(sum(1, 2, 3, 4)) + '|' + String(outer(3));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "10|6")?;
    Ok(())
}

#[test]
fn arguments_object_callee_references_current_function() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function factorial(n) {
            if (n <= 1) {
              return 1;
            }
            return n * arguments.callee(n - 1);
          }

          document.getElementById('result').textContent = String(factorial(4));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "24")?;
    Ok(())
}

#[test]
fn arguments_object_does_not_expose_array_methods() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function check() {
            try {
              arguments.sort();
              return 'no-error';
            } catch (e) {
              return 'threw';
            }
          }

          document.getElementById('result').textContent = check(3, 1, 2);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "threw")?;
    Ok(())
}

#[test]
fn async_arrow_function_expressions_are_supported() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const add = async (a, b) => a + b;
            const inc = async value => {
              return value + 1;
            };
            Promise.all([add(1, 2), inc(4)]).then((values) => {
              document.getElementById('result').textContent = values.join(':');
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.flush()?;
    h.assert_text("#result", "3:5")?;
    Ok(())
}

#[test]
fn async_identifier_arrow_form_still_parses_as_normal_arrow() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const fn = async => async + 1;
          document.getElementById('result').textContent = String(fn(3));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "4")?;
    Ok(())
}

#[test]
fn arrow_function_concise_assignment_expression_returns_assigned_value() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const fn = (value) => ((value) = value + 1);
          document.getElementById('result').textContent = String(fn(4));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "5")?;
    Ok(())
}

#[test]
fn arrow_function_with_parenthesized_parameter_and_optional_chain_body_parses() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const selectedLabel = (select) => select?.options?.[select.selectedIndex]?.textContent?.trim() || "";
          const sample = {
            selectedIndex: 0,
            options: [{ textContent: "  ok  " }],
          };
          document.getElementById('result').textContent = selectedLabel(sample);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn optional_chain_with_optional_index_and_optional_call_parses() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const sample = {
            selectedIndex: 0,
            options: [{ textContent: "  ok  " }],
          };
          const value = sample?.options?.[sample.selectedIndex]?.textContent?.trim() || "";
          document.getElementById('result').textContent = value;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn optional_chain_without_fallback_parses() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const sample = {
            selectedIndex: 0,
            options: [{ textContent: "  ok  " }],
          };
          const value = sample?.options?.[sample.selectedIndex]?.textContent?.trim();
          document.getElementById('result').textContent = value || "";
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok")?;
    Ok(())
}

#[test]
fn optional_chain_with_dom_select_options_dynamic_index_and_trim_works() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <select id='delay-count'>
          <option value='0'>  低  </option>
          <option value='1'>  中  </option>
        </select>
        <script>
          const selectedLabel = (select) => select?.options?.[select.selectedIndex]?.textContent?.trim() || "";
          const select = document.getElementById('delay-count');
          document.getElementById('result').textContent = selectedLabel(select);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "低")?;
    Ok(())
}

#[test]
fn optional_chaining_basic_property_access_and_missing_method_call_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const adventurer = {
            name: 'Alice',
            cat: {
              name: 'Dinah',
            },
          };

          const dogName = adventurer.dog?.name;
          const missingMethod = adventurer.someNonExistentMethod?.();
          document.getElementById('result').textContent =
            String(dogName) + ':' + String(missingMethod);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "undefined:undefined")?;
    Ok(())
}

#[test]
fn optional_chaining_function_call_behaviors_work_and_short_circuit_rhs_evaluation() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const iface = {
            value: 41,
            method() {
              return this.value + 1;
            },
            nonFn: 123,
          };

          let side = 0;
          const fromMissing = iface.missing?.(side++);
          const fromMethod = iface.method?.();
          const fnValue = undefined;
          const fromFn = fnValue?.(side++);

          let nonFunctionThrows = 'no';
          try {
            iface.nonFn?.();
          } catch (e) {
            nonFunctionThrows = 'yes';
          }

          let nullBaseThrows = 'no';
          try {
            const root = null;
            root.customMethod?.();
          } catch (e) {
            nullBaseThrows = 'yes';
          }

          document.getElementById('result').textContent =
            String(fromMissing) + ':' +
            String(fromMethod) + ':' +
            String(fromFn) + ':' +
            String(side) + ':' +
            nonFunctionThrows + ':' +
            nullBaseThrows;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "undefined:42:undefined:0:yes:yes")?;
    Ok(())
}

#[test]
fn optional_chaining_short_circuits_computed_operands_and_continuous_chains() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const potentiallyNullObj = null;
          let x = 0;

          const indexValue = potentiallyNullObj?.[x++];
          const chainedValue = potentiallyNullObj?.a.b;

          let groupedThrows = 'no';
          try {
            const grouped = (potentiallyNullObj?.a).b;
          } catch (e) {
            groupedThrows = 'yes';
          }

          document.getElementById('result').textContent =
            String(indexValue) + ':' + String(x) + ':' + String(chainedValue) + ':' + groupedThrows;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "undefined:0:undefined:yes")?;
    Ok(())
}

#[test]
fn optional_chaining_on_undeclared_root_still_throws_reference_error() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let threw = 'no';
          try {
            undeclaredVar?.prop;
          } catch (e) {
            threw = 'yes';
          }
          document.getElementById('result').textContent = threw;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "yes")?;
    Ok(())
}

#[test]
fn optional_chaining_invalid_syntax_cases_are_rejected() {
    for source in [
        "<script>const object = {}; object?.property = 1;</script>",
        "<script>String?.raw`Hello, world!`;</script>",
        "<script>String.raw?.`Hello, world!`;</script>",
        "<script>new Intl?.DateTimeFormat();</script>",
        "<script>new Map?.();</script>",
    ] {
        let err = Harness::from_html(source).expect_err("invalid optional chaining syntax");
        match err {
            Error::ScriptParse(msg) => assert!(!msg.is_empty()),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn for_each_arrow_concise_assignment_expression_updates_outer_binding() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const values = [1, 2, 3];
          let sum = 0;
          values.forEach((value) => (sum = sum + value));
          document.getElementById('result').textContent = String(sum);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "6")?;
    Ok(())
}

#[test]
fn arrow_function_line_break_before_arrow_is_rejected() {
    let err = Harness::from_html(
        "<script>const fn = (a, b)\n=> a + b; document.body.textContent = String(fn(1, 2));</script>",
    )
    .expect_err("line break between parameter list and arrow should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("=>") || msg.contains("unsupported")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn arrow_function_uses_lexical_this_even_when_called_with_other_receiver() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Box {
            constructor(value) {
              this.value = value;
            }
            makeReader() {
              return () => this.value;
            }
          }

          const box = new Box(7);
          const reader = box.makeReader();
          const wrapper = { value: 99, reader: reader };
          document.getElementById('result').textContent = String(wrapper.reader());
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "7")?;
    Ok(())
}

#[test]
fn normal_function_expression_keeps_dynamic_this_binding() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Box {
            constructor(value) {
              this.value = value;
            }
            makeReader() {
              return function() {
                return this.value;
              };
            }
          }

          const box = new Box(7);
          const reader = box.makeReader();
          const wrapper = { value: 99, reader: reader };
          document.getElementById('result').textContent = String(wrapper.reader());
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "99")?;
    Ok(())
}

#[test]
fn function_prototype_call_apply_and_bind_set_this_and_arguments() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function add(c, d) {
            return this.a + this.b + c + d;
          }

          const target = { a: 1, b: 3 };
          const fromCall = add.call(target, 5, 7);
          const fromApplyArray = add.apply(target, [10, 20]);
          const fromApplyArrayLike = add.apply(target, { 0: 4, 1: 5, length: 2 });
          const bound = add.bind({ a: 2, b: 4 }, 1);
          const fromBind = bound(3);

          document.getElementById('result').textContent =
            String(fromCall) + ':' +
            String(fromApplyArray) + ':' +
            String(fromApplyArrayLike) + ':' +
            String(fromBind);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "16:34:13:10")?;
    Ok(())
}

#[test]
fn function_prototype_call_and_bind_do_not_rebind_arrow_this() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const source = {
            value: 7,
            make() {
              return () => this.value;
            },
          };

          const arrow = source.make();
          const viaCall = arrow.call({ value: 99 });
          const rebound = arrow.bind({ value: 42 });
          const viaBind = rebound();
          document.getElementById('result').textContent = String(viaCall) + ':' + String(viaBind);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "7:7")?;
    Ok(())
}

#[test]
fn top_level_this_is_window_and_free_function_this_is_undefined() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function readThisKind() {
            return this === undefined ? 'undefined' : 'other';
          }

          const topLevel = this === window ? 'window' : 'other';
          document.getElementById('result').textContent = topLevel + ':' + readThisKind();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "window:undefined")?;
    Ok(())
}

#[test]
fn arrow_function_cannot_be_used_as_constructor() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const Arrow = () => {};
          let out = '';
          try {
            new Arrow();
            out = 'constructed';
          } catch (error) {
            out = String(error).includes('not a constructor') ? 'not-constructable' : String(error);
          }
          document.getElementById('result').textContent = out;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "not-constructable")?;
    Ok(())
}

#[test]
fn arrow_function_has_no_prototype_property() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const Arrow = () => {};
          const Fn = function() {};
          document.getElementById('result').textContent =
            typeof Fn.prototype + ':' + typeof Arrow.prototype;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "object:undefined")?;
    Ok(())
}

#[test]
fn default_parameter_initializer_cannot_access_function_body_bindings() {
    let err = Harness::from_html(
        "<script>function f(a = go()) { function go() { return ':P'; } } f();</script>",
    )
    .expect_err("default parameter initializer should not see function body bindings");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("unknown variable: go")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn class_list_toggle_force_argument_works() -> Result<()> {
    let html = r#"
        <input id='force' type='checkbox'>
        <div id='box' class='base'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').classList.toggle('active', document.getElementById('force').checked);
            if (document.getElementById('box').classList.contains('active'))
              document.getElementById('result').textContent = 'active';
            else
              document.getElementById('result').textContent = 'inactive';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "inactive")?;
    h.set_checked("#force", true)?;
    h.click("#btn")?;
    h.assert_text("#result", "active")?;
    h.set_checked("#force", false)?;
    h.click("#btn")?;
    h.assert_text("#result", "inactive")?;
    Ok(())
}

#[test]
fn logical_and_relational_and_strict_operators_work() -> Result<()> {
    let html = r#"
        <input id='age' value='25'>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const age = document.getElementById('age').value;
            const okRange = age >= 20 && age < 30;
            if ((okRange === true && age !== '40') || age === '18')
              document.getElementById('result').textContent = 'pass';
            else
              document.getElementById('result').textContent = 'fail';
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "pass")?;
    h.type_text("#age", "40")?;
    h.click("#btn")?;
    h.assert_text("#result", "fail")?;
    h.type_text("#age", "18")?;
    h.click("#btn")?;
    h.assert_text("#result", "pass")?;
    Ok(())
}

#[test]
fn logical_and_operator_returns_first_falsy_or_last_truthy_and_short_circuits() -> Result<()> {
    let html = r#"
        <p id='log'></p>
        <p id='result'></p>
        <script>
          function sideEffect(value) {
            const log = document.getElementById('log');
            log.textContent = log.textContent + value;
            return value;
          }
          function format(value) {
            if (typeof value === 'number' && Number.isNaN(value)) return 'NaN';
            if (value === '') return '<empty>';
            return String(value);
          }

          const out = [
            format(true && true),
            format(true && false),
            format(false && true),
            format(false && (3 === 4)),
            format("Cat" && "Dog"),
            format(false && "Cat"),
            format("Cat" && false),
            format("" && false),
            format(false && ""),
            format(NaN && "x"),
            format(0 && sideEffect('rhs')),
            format(1 && sideEffect('rhs')),
            document.getElementById('log').textContent,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true,false,false,false,Dog,false,false,<empty>,false,NaN,0,rhs,rhs",
    )?;
    Ok(())
}

#[test]
fn logical_or_operator_returns_first_truthy_or_last_falsy_and_short_circuits() -> Result<()> {
    let html = r#"
        <p id='log'></p>
        <p id='result'></p>
        <script>
          function sideEffect(value) {
            const log = document.getElementById('log');
            log.textContent = log.textContent + value;
            return value;
          }
          function format(value) {
            if (typeof value === 'number' && Number.isNaN(value)) return 'NaN';
            if (value === '') return '<empty>';
            if (value && typeof value === 'object') return 'object';
            return String(value);
          }

          const obj = { ok: 1 };
          const out = [
            format(true || true),
            format(false || true),
            format(true || false),
            format(false || (3 === 4)),
            format("Cat" || "Dog"),
            format(false || "Cat"),
            format("Cat" || false),
            format("" || false),
            format(false || ""),
            format(false || obj),
            format(NaN || "x"),
            format(0 || sideEffect('rhs')),
            format(1 || sideEffect('skip')),
            document.getElementById('log').textContent,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true,true,true,false,Cat,Cat,Cat,false,<empty>,object,x,rhs,1,rhs",
    )?;
    Ok(())
}

#[test]
fn logical_or_operator_precedence_and_grouping_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            true || false && false,
            (true || false) && false,
            false || true && false,
            (false || true) && false,
          ];
          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true,false,false,false")?;
    Ok(())
}

#[test]
fn logical_and_operator_has_higher_precedence_than_or() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            true || false && false,
            true && (false || false),
            (2 === 3) || (4 < 0) && (1 === 1),
          ];
          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true,false,false")?;
    Ok(())
}

#[test]
fn logical_not_operator_negates_truthiness_and_supports_double_not_coercion() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const a = 3;
          const b = -2;
          const out = [
            !(a > 0 || b > 0),
            !true,
            !false,
            !'',
            !'Cat',
            !null,
            !NaN,
            !0,
            !undefined,
            !!true,
            !!{},
            !!false,
            !!'',
            !!Boolean(false),
          ];
          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "false,false,true,true,false,true,true,true,true,true,true,false,false,false",
    )?;
    Ok(())
}

#[test]
fn null_keyword_core_behaviors_match_javascript_rules() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            typeof null,
            typeof undefined,
            null === undefined,
            null == undefined,
            null === null,
            null == null,
            !null,
            Number.isNaN(1 + null),
            Number.isNaN(1 + undefined),
            JSON.stringify(null),
          ];
          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "object,undefined,false,true,true,true,true,false,true,null",
    )?;
    Ok(())
}

#[test]
fn property_accessors_dot_and_bracket_basic_usage_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const person1 = {};
          person1["firstName"] = "Mario";
          person1["lastName"] = "Rossi";

          const person2 = {
            firstName: "John",
            lastName: "Doe",
          };

          const object = {};
          object.$1 = "foo";
          const reserved = { default: 7 };

          document.getElementById('result').textContent =
            person1.firstName + ':' +
            person2["lastName"] + ':' +
            object.$1 + ':' +
            String(reserved.default);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "Mario:Doe:foo:7")?;
    Ok(())
}

#[test]
fn property_accessors_bracket_expression_and_key_coercion_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const key = "name";
          const getKey = () => "name";
          const obj = { name: "Michel" };

          const dict = {};
          dict["1"] = "value";

          const foo = { uniqueProp: 1 };
          const bar = { uniqueProp: 2 };
          const refMap = {};
          refMap[foo] = "same-key";

          document.getElementById('result').textContent =
            obj["name"] + ':' +
            obj[key] + ':' +
            obj[getKey()] + ':' +
            dict[1] + ':' +
            refMap[bar];
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "Michel:Michel:Michel:value:same-key")?;
    Ok(())
}

#[test]
fn property_accessors_method_this_depends_on_call_site() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const source = {
            x: 1,
            getX() {
              return this.x;
            },
          };
          const target = { x: 7, getX: source.getX };

          document.getElementById('result').textContent =
            String(source.getX()) + ':' + String(target.getX());
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:7")?;
    Ok(())
}

#[test]
fn property_accessors_support_numeric_literal_method_forms() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const a = 77 .toExponential();
          const b = (77).toExponential();
          const c = 77..toExponential();
          document.getElementById('result').textContent = a + ':' + b + ':' + c;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "7.7e+1:7.7e+1:7.7e+1")?;
    Ok(())
}

#[test]
fn property_accessors_dot_notation_rejects_numeric_identifier() {
    let err = Harness::from_html("<script>const object = {}; object.1 = 'bar';</script>")
        .expect_err("object.1 should be invalid syntax");
    match err {
        Error::ScriptParse(msg) => assert!(!msg.is_empty()),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn property_access_on_null_throws_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const value = null;
            value.anyProp;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("property access on null should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("not an object")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn remainder_operator_handles_number_bigint_and_special_values() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            13 % 5,
            -13 % 5,
            4 % 2,
            1 / (-4 % 2),
            1 % -2,
            1 % 2,
            2 % 3,
            5.5 % 2,
            String(3n % 2n),
            String(-3n % 2n),
            Number.isNaN(NaN % 2),
            Number.isNaN(Infinity % 2),
            Number.isNaN(Infinity % 0),
            Number.isNaN(Infinity % Infinity),
            2 % Infinity,
            0 % Infinity,
          ];
          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "3,-3,0,-Infinity,1,1,2,1.5,1,-1,true,true,true,true,2,0",
    )?;
    Ok(())
}

#[test]
fn remainder_operator_rejects_mixed_bigint_and_number() -> Result<()> {
    let html = r#"
        <button id='mix1'>mix1</button>
        <button id='mix2'>mix2</button>
        <script>
          document.getElementById('mix1').addEventListener('click', () => {
            const v = 2n % 2;
          });
          document.getElementById('mix2').addEventListener('click', () => {
            const v = 2 % 2n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let mix1 = h
        .click("#mix1")
        .expect_err("BigInt and Number remainder should fail");
    match mix1 {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("cannot mix BigInt and other types in arithmetic operations"))
        }
        other => panic!("unexpected mixed-type remainder error: {other:?}"),
    }

    let mix2 = h
        .click("#mix2")
        .expect_err("Number and BigInt remainder should fail");
    match mix2 {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("cannot mix BigInt and other types in arithmetic operations"))
        }
        other => panic!("unexpected mixed-type remainder error: {other:?}"),
    }

    Ok(())
}

#[test]
fn remainder_operator_rejects_bigint_division_by_zero() -> Result<()> {
    let html = r#"
        <button id='zero'>zero</button>
        <script>
          document.getElementById('zero').addEventListener('click', () => {
            const v = 2n % 0n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let zero = h
        .click("#zero")
        .expect_err("BigInt remainder by zero should fail");
    match zero {
        Error::ScriptRuntime(msg) => assert!(msg.contains("division by zero")),
        other => panic!("unexpected BigInt remainder-by-zero error: {other:?}"),
    }
    Ok(())
}

#[test]
fn remainder_assignment_operator_handles_number_nan_and_bigint_cases() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let a = 3;
          a %= 2;
          const first = a;
          a %= 0;
          const second = String(a);
          a %= "hello";
          const third = String(a);

          let b = 5;
          b %= 2;
          const fourth = b;

          let c = 3n;
          c %= 2n;
          const fifth = String(c);

          document.getElementById('result').textContent =
            String(first) + ':' + second + ':' + third + ':' + String(fourth) + ':' + fifth;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:NaN:NaN:1:1")?;
    Ok(())
}

#[test]
fn remainder_assignment_operator_expression_returns_assigned_value() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let a = 3;
          const first = (a %= 2);
          const second = (a %= 0);
          const third = (a %= "hello");
          document.getElementById('result').textContent =
            String(first) + ':' + String(second) + ':' + String(third) + ':' + String(a);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:NaN:NaN:NaN")?;
    Ok(())
}

#[test]
fn remainder_assignment_operator_rejects_mixed_bigint_and_number() -> Result<()> {
    let html = r#"
        <button id='mix1'>mix1</button>
        <button id='mix2'>mix2</button>
        <script>
          document.getElementById('mix1').addEventListener('click', () => {
            let x = 3n;
            x %= 2;
          });
          document.getElementById('mix2').addEventListener('click', () => {
            let y = 3;
            y %= 2n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    let mix1 = h
        .click("#mix1")
        .expect_err("BigInt and Number remainder assignment should fail");
    match mix1 {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("cannot mix BigInt and other types in arithmetic operations"))
        }
        other => panic!("unexpected mixed-type remainder-assignment error: {other:?}"),
    }

    let mix2 = h
        .click("#mix2")
        .expect_err("Number and BigInt remainder assignment should fail");
    match mix2 {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("cannot mix BigInt and other types in arithmetic operations"))
        }
        other => panic!("unexpected mixed-type remainder-assignment error: {other:?}"),
    }

    Ok(())
}

#[test]
fn remainder_assignment_operator_rejects_bigint_zero_divisor() -> Result<()> {
    let html = r#"
        <button id='zero'>zero</button>
        <script>
          document.getElementById('zero').addEventListener('click', () => {
            let x = 3n;
            x %= 0n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let zero = h
        .click("#zero")
        .expect_err("BigInt remainder assignment by zero should fail");
    match zero {
        Error::ScriptRuntime(msg) => assert!(msg.contains("division by zero")),
        other => panic!("unexpected BigInt remainder-assignment-by-zero error: {other:?}"),
    }
    Ok(())
}

#[test]
fn right_shift_operator_handles_number_rules_and_coercion() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            5 >> 2,
            -5 >> 2,
            9 >> 2,
            -9 >> 2,
            100 >> 32,
            100 >> 33,
            4294967297 >> 0,
            "8" >> 1,
            true >> 1,
            undefined >> 1,
            Infinity >> 1,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1,-2,2,-3,100,50,1,4,0,0,0")?;
    Ok(())
}

#[test]
fn right_shift_operator_supports_bigint() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            String(9n >> 2n),
            String(-9n >> 2n),
            String(8n >> -1n),
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "2,-3,16")?;
    Ok(())
}

#[test]
fn right_shift_operator_rejects_mixed_bigint_and_number() -> Result<()> {
    let html = r#"
        <button id='mix1'>mix1</button>
        <button id='mix2'>mix2</button>
        <button id='mix3'>mix3</button>
        <script>
          document.getElementById('mix1').addEventListener('click', () => {
            const v = 1n >> 1;
          });
          document.getElementById('mix2').addEventListener('click', () => {
            const v = 1 >> 1n;
          });
          document.getElementById('mix3').addEventListener('click', () => {
            const v = "1" >> 1n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    for selector in ["#mix1", "#mix2", "#mix3"] {
        let err = h
            .click(selector)
            .expect_err("mixed BigInt and Number right shift should fail");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("cannot mix BigInt and other types in bitwise operations"))
            }
            other => panic!("unexpected mixed-type right shift error: {other:?}"),
        }
    }

    Ok(())
}

#[test]
fn unsigned_right_shift_operator_handles_number_rules_and_coercion() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            5 >>> 2,
            -5 >>> 2,
            9 >>> 2,
            -9 >>> 2,
            100 >>> 32,
            100 >>> 33,
            4294967297 >>> 0,
            "8" >>> 1,
            true >>> 1,
            undefined >>> 1,
            Infinity >>> 1,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1,1073741822,2,1073741821,100,50,1,4,0,0,0")?;
    Ok(())
}

#[test]
fn unsigned_right_shift_operator_rejects_bigint_values() -> Result<()> {
    let html = r#"
        <button id='mix1'>mix1</button>
        <button id='mix2'>mix2</button>
        <button id='mix3'>mix3</button>
        <script>
          document.getElementById('mix1').addEventListener('click', () => {
            const v = 1n >>> 1n;
          });
          document.getElementById('mix2').addEventListener('click', () => {
            const v = 1n >>> 1;
          });
          document.getElementById('mix3').addEventListener('click', () => {
            const v = 1 >>> 1n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    for selector in ["#mix1", "#mix2", "#mix3"] {
        let err = h
            .click(selector)
            .expect_err("unsigned right shift with BigInt should fail");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("BigInt values do not support unsigned right shift"))
            }
            other => panic!("unexpected unsigned right shift BigInt error: {other:?}"),
        }
    }

    Ok(())
}

#[test]
fn unsigned_right_shift_assignment_operator_handles_number_rules() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let a = 5;
          const ra = (a >>>= 2);

          let b = -5;
          const rb = (b >>>= 2);

          let c = 100;
          c >>>= 32;

          let d = 100;
          d >>>= 33;

          let e = "8";
          const re = (e >>>= 1);

          document.getElementById('result').textContent =
            String(ra) + ':' + String(a) + ':' +
            String(rb) + ':' + String(b) + ':' +
            String(c) + ':' + String(d) + ':' +
            String(re) + ':' + String(e);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:1:1073741822:1073741822:100:50:4:4")?;
    Ok(())
}

#[test]
fn unsigned_right_shift_assignment_operator_rejects_bigint_values() -> Result<()> {
    let html = r#"
        <button id='mix1'>mix1</button>
        <button id='mix2'>mix2</button>
        <button id='mix3'>mix3</button>
        <script>
          document.getElementById('mix1').addEventListener('click', () => {
            let x = 5n;
            x >>>= 2n;
          });
          document.getElementById('mix2').addEventListener('click', () => {
            let y = 5n;
            y >>>= 2;
          });
          document.getElementById('mix3').addEventListener('click', () => {
            let z = 5;
            z >>>= 2n;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    for selector in ["#mix1", "#mix2", "#mix3"] {
        let err = h
            .click(selector)
            .expect_err("unsigned right shift assignment with BigInt should fail");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("BigInt values do not support unsigned right shift"))
            }
            other => panic!("unexpected unsigned right shift assignment BigInt error: {other:?}"),
        }
    }

    Ok(())
}

#[test]
fn right_shift_assignment_operator_handles_number_and_bigint() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let a = 5;
          const ra = (a >>= 2);

          let b = -5;
          const rb = (b >>= 2);

          let c = 5n;
          const rc = (c >>= 2n);

          let d = 100;
          d >>= 32;

          document.getElementById('result').textContent =
            String(ra) + ':' + String(a) + ':' +
            String(rb) + ':' + String(b) + ':' +
            String(rc) + ':' + String(c) + ':' +
            String(d);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:1:-2:-2:1:1:100")?;
    Ok(())
}

#[test]
fn right_shift_assignment_operator_rejects_mixed_bigint_and_number() -> Result<()> {
    let html = r#"
        <button id='mix1'>mix1</button>
        <button id='mix2'>mix2</button>
        <button id='mix3'>mix3</button>
        <script>
          document.getElementById('mix1').addEventListener('click', () => {
            let x = 5n;
            x >>= 2;
          });
          document.getElementById('mix2').addEventListener('click', () => {
            let y = 5;
            y >>= 2n;
          });
          document.getElementById('mix3').addEventListener('click', () => {
            let z = 5n;
            z >>= "2";
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    for selector in ["#mix1", "#mix2", "#mix3"] {
        let err = h
            .click(selector)
            .expect_err("mixed BigInt and Number right shift assignment should fail");
        match err {
            Error::ScriptRuntime(msg) => {
                assert!(msg.contains("cannot mix BigInt and other types in bitwise operations"))
            }
            other => panic!("unexpected mixed-type right shift assignment error: {other:?}"),
        }
    }

    Ok(())
}

#[test]
fn spread_syntax_in_function_calls_and_constructor_calls_works() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function sum(x, y, z) {
            return x + y + z;
          }
          function collect(v, w, x, y, z) {
            return [v, w, x, y, z].join(',');
          }
          function Point(x, y, z) {
            this.x = x;
            this.y = y;
            this.z = z;
          }

          const args = [0, 1];
          const numbers = [1, 2, 3];
          const pointFields = [7, 8, 9];
          const point = new Point(...pointFields);

          document.getElementById('result').textContent = [
            sum(...numbers),
            collect(-1, ...args, 2, ...[3]),
            ((...chars) => chars.join(''))(...'abc'),
            [point.x, point.y, point.z].join(','),
          ].join(':');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "6:-1,0,1,2,3:abc:7,8,9")?;
    Ok(())
}

#[test]
fn spread_syntax_in_array_literals_supports_copy_concat_and_conditional_elements() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const parts = ['shoulders', 'knees'];
          const lyrics = ['head', ...parts, 'and', 'toes'];

          const arr = [1, 2, 3];
          const arr2 = [...arr];
          arr2.push(4);

          const isSummer = false;
          const fruits = ['apple', 'banana', ...(isSummer ? ['watermelon'] : [])];

          const nested = [[1], [2], [3]];
          const shallow = [...nested];
          shallow.shift().shift();

          document.getElementById('result').textContent =
            lyrics.join('|') + ':' +
            arr.join(',') + ':' +
            arr2.join(',') + ':' +
            fruits.join(',') + ':' +
            JSON.stringify(nested);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "head|shoulders|knees|and|toes:1,2,3:1,2,3,4:apple,banana:[[],[2],[3]]",
    )?;
    Ok(())
}

#[test]
fn spread_syntax_in_object_literals_supports_merge_override_and_primitive_sources() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const obj1 = { foo: 'bar', x: 42 };
          const obj2 = { foo: 'baz', y: 13 };
          const merged = { x: 41, ...obj1, ...obj2, y: 9 };

          const primitiveSpread = { ...true, ...'test', ...10 };

          const isSummer = false;
          const fruits = {
            apple: 10,
            banana: 5,
            ...(isSummer && { watermelon: 30 }),
          };

          const nullishSpread = { a: 1, ...null, ...undefined, b: 2 };

          window.setterCalled = 0;
          const spreadTarget = {
            set foo(value) {
              window.setterCalled += 1;
            },
            ...{ foo: 1 },
          };

          const merge = (...objects) => ({ ...objects });
          const mergeReduce = (...objects) =>
            objects.reduce((acc, cur) => ({ ...acc, ...cur }), {});
          const mergedObj1 = merge(obj1, obj2);
          const mergedObj2 = mergeReduce(obj1, obj2);

          document.getElementById('result').textContent = [
            merged.foo + ':' + merged.x + ':' + merged.y,
            primitiveSpread[0] + primitiveSpread[3],
            Object.keys(primitiveSpread).join(','),
            String(Object.hasOwn(fruits, 'watermelon')),
            String(Object.hasOwn(nullishSpread, 'a')) + ':' + String(Object.hasOwn(nullishSpread, 'b')),
            String(window.setterCalled) + ':' + String(spreadTarget.foo),
            Object.keys(mergedObj1).join(','),
            mergedObj2.foo + ':' + mergedObj2.x + ':' + mergedObj2.y,
          ].join('|');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "baz:42:9|tt|0,1,2,3|false|true:true|0:1|0,1|baz:42:13",
    )?;
    Ok(())
}

#[test]
fn spread_syntax_requires_iterables_in_array_literals_and_call_arguments() -> Result<()> {
    let html = r#"
        <button id='arr'>arr</button>
        <button id='call'>call</button>
        <script>
          document.getElementById('arr').addEventListener('click', () => {
            const values = [...{ key1: 'value1' }];
          });
          document.getElementById('call').addEventListener('click', () => {
            const f = () => {};
            f(...{ key1: 'value1' });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;

    for selector in ["#arr", "#call"] {
        let err = h
            .click(selector)
            .expect_err("spreading a non-iterable should fail");
        match err {
            Error::ScriptRuntime(msg) => assert!(msg.contains("spread source is not iterable")),
            other => panic!("unexpected spread non-iterable error: {other:?}"),
        }
    }

    Ok(())
}

#[test]
fn less_than_operator_handles_number_string_bigint_and_special_values() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            5 < 3,
            3 < 3,
            3n < 5,
            "aa" < "ab",
            "a" < "b",
            "a" < "a",
            "a" < "3",
            "5" < 3,
            "3" < 3,
            "3" < 5,
            "hello" < 5,
            5 < "hello",
            "5" < 3n,
            "3" < 5n,
            5n < 3,
            3 < 5n,
            true < false,
            false < true,
            0 < true,
            true < 1,
            null < 0,
            null < 1,
            undefined < 3,
            3 < undefined,
            3 < NaN,
            NaN < 3,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "false,false,true,true,true,false,false,false,false,true,false,false,false,true,false,true,false,true,true,false,false,true,false,false,false,false",
    )?;
    Ok(())
}

#[test]
fn less_than_operator_handles_bigint_string_non_integral_edge_cases() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            "1.5" < 2n,
            2n < "2.5",
            "2e0" < 3n,
            1n < "001",
            1n < " 2 ",
            " 2 " < 3n,
            1n < "not-a-bigint",
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "false,false,false,false,true,true,false")?;
    Ok(())
}

#[test]
fn greater_than_operator_handles_number_string_bigint_and_special_values() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            5 > 3,
            3 > 3,
            3n > 5,
            "ab" > "aa",
            "5" > 3,
            "3" > 3,
            "3" > 5,
            "hello" > 5,
            5 > "hello",
            "5" > 3n,
            "3" > 5n,
            5n > 3,
            3 > 5n,
            true > false,
            false > true,
            true > 0,
            true > 1,
            null > 0,
            1 > null,
            undefined > 3,
            3 > undefined,
            3 > NaN,
            NaN > 3,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true,false,false,true,true,false,false,false,false,true,false,true,false,true,false,true,false,false,true,false,false,false,false",
    )?;
    Ok(())
}

#[test]
fn greater_than_or_equal_operator_handles_number_string_bigint_and_special_values() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            5 >= 3,
            3 >= 3,
            3n >= 5,
            "ab" >= "aa",
            "a" >= "b",
            "a" >= "a",
            "a" >= "3",
            "5" >= 3,
            "3" >= 3,
            "3" >= 5,
            "hello" >= 5,
            5 >= "hello",
            5n >= 3,
            3 >= 3n,
            3 >= 5n,
            true >= false,
            true >= true,
            false >= true,
            true >= 0,
            true >= 1,
            null >= 0,
            1 >= null,
            undefined >= 3,
            3 >= undefined,
            3 >= NaN,
            NaN >= 3,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true,true,false,true,false,true,true,true,true,false,false,false,true,true,false,true,true,false,true,true,true,true,false,false,false,false",
    )?;
    Ok(())
}

#[test]
fn less_than_or_equal_operator_handles_number_string_bigint_and_special_values() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const out = [
            5 <= 3,
            3 <= 3,
            3n <= 5,
            "aa" <= "ab",
            "a" <= "b",
            "a" <= "a",
            "a" <= "3",
            "5" <= 3,
            "3" <= 3,
            "3" <= 5,
            "hello" <= 5,
            5 <= "hello",
            "5" <= 3n,
            "3" <= 5n,
            5n <= 3,
            3 <= 3n,
            3 <= 5n,
            true <= false,
            true <= true,
            false <= true,
            true <= 0,
            true <= 1,
            null <= 0,
            1 <= null,
            undefined <= 3,
            3 <= undefined,
            3 <= NaN,
            NaN <= 3,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "false,true,true,true,true,true,false,false,true,true,false,false,false,true,false,true,true,false,true,true,false,true,true,false,false,false,false,false",
    )?;
    Ok(())
}

#[test]
fn less_than_or_equal_operator_special_equivalence_edge_cases_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const sameObject = {};
          const leftDate = new Date(0);
          const rightDate = new Date(0);
          const badBigIntString = 'not-a-bigint';

          const out = [
            null <= 0,
            (null < 0) || (null == 0),
            undefined <= null,
            undefined == null,
            sameObject <= sameObject,
            sameObject == sameObject,
            leftDate <= rightDate,
            (leftDate < rightDate) || (leftDate == rightDate),
            1n <= badBigIntString,
            1n > badBigIntString,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true,false,false,true,false,true,true,false,false,false",
    )?;
    Ok(())
}

#[test]
fn greater_than_or_equal_operator_special_equivalence_edge_cases_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const sameObject = {};
          const leftDate = new Date(0);
          const rightDate = new Date(0);
          const badBigIntString = 'not-a-bigint';

          const out = [
            null >= 0,
            (null > 0) || (null == 0),
            undefined >= null,
            undefined == null,
            sameObject >= sameObject,
            sameObject == sameObject,
            leftDate >= rightDate,
            (leftDate > rightDate) || (leftDate == rightDate),
            1n >= badBigIntString,
            1n < badBigIntString,
          ];

          document.getElementById('result').textContent = out.join(',');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text(
        "#result",
        "true,false,false,true,false,true,true,false,false,false",
    )?;
    Ok(())
}

#[test]
fn dom_properties_and_attribute_methods_work() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').setAttribute('data-x', 'v1');
            document.getElementById('box').className = 'a b';
            document.getElementById('box').id = 'box2';
            document.getElementById('box2').name = 'named';
            const x = document.getElementById('box2').getAttribute('data-x');
            document.getElementById('result').textContent =
              document.getElementById('box2').name + ':' + document.getElementById('box2').className + ':' + x;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_exists("#box2")?;
    h.assert_text("#result", "named:a b:v1")?;
    Ok(())
}

#[test]
fn dataset_property_read_write_works() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').dataset.userId = 'u42';
            document.getElementById('box').dataset.planType = 'pro';
            document.getElementById('result').textContent =
              document.getElementById('box').dataset.userId + ':' +
              document.getElementById('box').getAttribute('data-user-id') + ':' +
              document.getElementById('box').dataset.planType;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "u42:u42:pro")?;
    Ok(())
}

#[test]
fn disabled_property_read_write_works() -> Result<()> {
    let html = r#"
        <input id='name' value='init'>
        <button id='toggle'>toggle-disabled</button>
        <button id='enable'>enable</button>
        <p id='result'></p>
        <script>
          document.getElementById('toggle').addEventListener('click', () => {
            document.getElementById('name').disabled = true;
            document.getElementById('result').textContent =
              document.getElementById('name').disabled + ':' +
              document.getElementById('name').getAttribute('disabled');
          });
          document.getElementById('enable').addEventListener('click', () => {
            document.getElementById('name').disabled = false;
            document.getElementById('result').textContent =
              document.getElementById('name').disabled + ':' +
              document.getElementById('name').getAttribute('disabled');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#toggle")?;
    h.assert_text("#result", "true:true")?;
    h.click("#enable")?;
    h.assert_text("#result", "false:null")?;
    Ok(())
}

#[test]
fn readonly_property_read_write_and_type_text_is_ignored() -> Result<()> {
    let html = r#"
        <input id='name' value='init' readonly>
        <button id='make-editable'>editable</button>
        <button id='confirm'>confirm</button>
        <p id='result'></p>
        <script>
          document.getElementById('make-editable').addEventListener('click', () => {
            document.getElementById('name').readonly = false;
            document.getElementById('result').textContent = document.getElementById('name').readonly;
          });
          document.getElementById('confirm').addEventListener('click', () => {
            document.getElementById('result').textContent =
              document.getElementById('name').readonly + ':' +
              document.getElementById('name').value;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.type_text("#name", "changed")?;
    h.assert_value("#name", "init")?;
    h.click("#make-editable")?;
    h.type_text("#name", "changed")?;
    h.assert_value("#name", "changed")?;
    h.click("#confirm")?;
    h.assert_text("#result", "false:changed")?;
    Ok(())
}

#[test]
fn required_property_read_write_works() -> Result<()> {
    let html = r#"
        <input id='name' required>
        <button id='unset'>unset</button>
        <button id='set'>set</button>
        <p id='result'></p>
        <script>
          document.getElementById('set').addEventListener('click', () => {
            document.getElementById('name').required = true;
            document.getElementById('result').textContent = document.getElementById('name').required;
          });
          document.getElementById('unset').addEventListener('click', () => {
            document.getElementById('name').required = false;
            document.getElementById('result').textContent = document.getElementById('name').required;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#unset")?;
    h.assert_text("#result", "false")?;
    h.click("#set")?;
    h.assert_text("#result", "true")?;
    Ok(())
}

#[test]
fn style_property_read_write_works() -> Result<()> {
    let html = r#"
        <div id='box' style='color: blue;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').style.backgroundColor = 'red';
            document.getElementById('box').style.color = '';
            document.getElementById('result').textContent =
              document.getElementById('box').style.backgroundColor + ':' +
              document.getElementById('box').style.color + ':' +
              document.getElementById('box').getAttribute('style');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "red::background-color: red;")?;
    Ok(())
}

#[test]
fn offset_and_scroll_properties_are_read_only_and_queryable() -> Result<()> {
    let html = r#"
        <div id='box' style='width: 120px; height: 90px;'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            document.getElementById('result').textContent =
              box.offsetWidth + ':' + box.offsetHeight + ':' +
              box.offsetTop + ':' + box.offsetLeft + ':' +
              box.scrollWidth + ':' + box.scrollHeight + ':' +
              box.scrollTop + ':' + box.scrollLeft;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "0:0:0:0:0:0:0:0")?;
    Ok(())
}

#[test]
fn offset_property_assignment_is_rejected() -> Result<()> {
    let html = r#"
        <div id='box'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('box').scrollTop = 10;
            document.getElementById('box').offsetWidth = 100;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("scrollTop/offsetWidth assignment should fail");
    assert!(format!("{err}").contains("is read-only"));
    Ok(())
}

#[test]
fn dataset_camel_case_mapping_works() -> Result<()> {
    let html = r#"
        <div id='box' data-user-id='u1' data-plan-type='starter'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.dataset.accountStatus = 'active';
            document.getElementById('result').textContent =
              box.dataset.userId + ':' +
              box.dataset.planType + ':' +
              box.getAttribute('data-account-status');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "u1:starter:active")?;
    Ok(())
}

#[test]
fn element_core_properties_and_aria_reflection_work() -> Result<()> {
    let html = r#"
        <div id='box' class='x y'>
          <span id='a'>A</span>
          <span id='b'>B</span>
        </div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const box = document.getElementById('box');
            box.slot = 'hero';
            box.role = 'region';
            box.ariaLabel = 'Main panel';
            box.ariaBusy = 'true';
            box.elementTiming = 'paint';

            const first = box.firstElementChild;
            const last = box.lastElementChild;
            const next = first.nextElementSibling;
            const prev = last.previousElementSibling;

            document.getElementById('result').textContent =
              box.tagName + ':' +
              box.localName + ':' +
              box.namespaceURI + ':' +
              box.childElementCount + ':' +
              box.children.length + ':' +
              first.id + ':' +
              last.id + ':' +
              next.id + ':' +
              prev.id + ':' +
              box.clientWidth + ':' +
              box.clientHeight + ':' +
              box.clientLeft + ':' +
              box.clientTop + ':' +
              box.currentCSSZoom + ':' +
              box.scrollLeftMax + ':' +
              box.scrollTopMax + ':' +
              (box.shadowRoot === null) + ':' +
              (box.assignedSlot === null) + ':' +
              (box.prefix === null) + ':' +
              box.slot + ':' +
              box.getAttribute('slot') + ':' +
              box.role + ':' +
              box.getAttribute('role') + ':' +
              box.ariaLabel + ':' +
              box.getAttribute('aria-label') + ':' +
              box.ariaBusy + ':' +
              box.getAttribute('aria-busy') + ':' +
              box.elementTiming + ':' +
              box.getAttribute('elementtiming') + ':' +
              box.classList.length + ':' +
              box.part.length + ':' +
              !!box.attributes;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "DIV:div:http://www.w3.org/1999/xhtml:2:2:a:b:b:a:0:0:0:0:1:0:0:true:true:true:hero:hero:region:region:Main panel:Main panel:true:true:paint:paint:2:0:true",
    )?;
    Ok(())
}

#[test]
fn aria_element_reference_properties_resolve_id_refs() -> Result<()> {
    let html = r#"
        <input
          id='field'
          aria-activedescendant='opt2'
          aria-controls='panel1 panel2'
          aria-describedby='desc'
          aria-details='detail'
          aria-errormessage='err'
          aria-flowto='next1 next2'
          aria-labelledby='lbl'
          aria-owns='owned1 owned2'
        >
        <div id='panel1'></div>
        <div id='panel2'></div>
        <p id='desc'></p>
        <div id='detail'></div>
        <p id='err'></p>
        <span id='next1'></span>
        <span id='next2'></span>
        <label id='lbl'></label>
        <div id='owned1'></div>
        <div id='owned2'></div>
        <div id='opt2'></div>
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('btn').addEventListener('click', () => {
            const field = document.getElementById('field');
            const active = field.ariaActiveDescendantElement;
            const controls = field.ariaControlsElements;
            const described = field.ariaDescribedByElements;
            const details = field.ariaDetailsElements;
            const errors = field.ariaErrorMessageElements;
            const flow = field.ariaFlowToElements;
            const labelled = field.ariaLabelledByElements;
            const owns = field.ariaOwnsElements;

            document.getElementById('result').textContent =
              active.id + ':' +
              controls.length + ':' +
              controls[0].id + ':' +
              controls[1].id + ':' +
              described[0].id + ':' +
              details[0].id + ':' +
              errors[0].id + ':' +
              flow[0].id + ':' +
              flow[1].id + ':' +
              labelled[0].id + ':' +
              owns.length + ':' +
              owns[1].id;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text(
        "#result",
        "opt2:2:panel1:panel2:desc:detail:err:next1:next2:lbl:2:owned2",
    )?;
    Ok(())
}

#[test]
fn class_expression_supports_anonymous_class_values() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          const Rectangle = class {
            constructor(height, width) {
              this.height = height;
              this.width = width;
            }

            area() {
              return this.height * this.width;
            }
          };

          document.getElementById('btn').addEventListener('click', () => {
            document.getElementById('result').textContent = String(new Rectangle(5, 8).area());
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "40")?;
    Ok(())
}

#[test]
fn named_class_expression_name_is_local_to_class_body() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const Foo = class NamedFoo {
            innerType() {
              return typeof NamedFoo;
            }
          };

          const first = new Foo();
          document.getElementById('result').textContent =
            first.innerType() + ':' + typeof NamedFoo;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "function:undefined")?;
    Ok(())
}

#[test]
fn class_declaration_supports_constructor_and_new() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          class Polygon {
            constructor(height, width) {
              this.area = height * width;
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            const polygon = new Polygon(4, 3);
            document.getElementById('result').textContent = String(polygon.area);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "12")?;
    Ok(())
}

#[test]
fn new_operator_supports_omitted_argument_list_and_property_access_callee() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function Plain(color) {
            this.color = color || 'red';
          }
          const registry = {
            Car: function(make) {
              this.make = make || 'Default';
            },
          };

          const a = new Plain;
          const b = new registry.Car('Eagle');
          const c = new registry.Car;
          document.getElementById('result').textContent = a.color + ':' + b.make + ':' + c.make;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "red:Eagle:Default")?;
    Ok(())
}

#[test]
fn new_operator_uses_non_primitive_constructor_return_and_ignores_primitive_return() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function ReturnsObject() {
            this.tag = 'instance';
            return { tag: 'override' };
          }
          function ReturnsPrimitive() {
            this.tag = 'kept';
            return 1;
          }

          const a = new ReturnsObject();
          const b = new ReturnsPrimitive();
          document.getElementById('result').textContent = a.tag + ':' + b.tag;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "override:kept")?;
    Ok(())
}

#[test]
fn new_operator_uses_current_constructor_prototype_value() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function C() {}
          const first = new C();
          C.prototype = { marker: 'next' };

          document.getElementById('btn').addEventListener('click', () => {
            const second = new C();
            document.getElementById('result').textContent =
              (second instanceof C) + ':' +
              (first instanceof C) + ':' +
              String(second.marker) + ':' +
              String(first.marker);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "true:false:next:undefined")?;
    Ok(())
}

#[test]
fn new_target_in_function_is_constructor_or_undefined() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function Foo() {
            const seen = new.target === Foo ? 'Foo' : String(new.target);
            if (new.target) {
              this.seen = seen;
            }
            return seen;
          }

          document.getElementById('btn').addEventListener('click', () => {
            const called = Foo();
            const constructed = new Foo();
            document.getElementById('result').textContent = called + ':' + constructed.seen;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "undefined:Foo")?;
    Ok(())
}

#[test]
fn new_target_in_derived_and_base_constructors_points_to_invoked_class() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class A {
            constructor() {
              this.baseSeen = new.target === B ? 'B' : String(new.target);
            }
          }

          class B extends A {
            constructor() {
              super();
              this.derivedSeen = new.target === B ? 'B' : String(new.target);
            }
          }

          const b = new B();
          document.getElementById('result').textContent = b.baseSeen + ':' + b.derivedSeen;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "B:B")?;
    Ok(())
}

#[test]
fn new_target_in_arrow_function_inherits_outer_binding() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          function plain() {
            const read = () => (new.target === undefined ? 'undefined' : 'set');
            return read();
          }

          function Box() {
            const read = () => (new.target === Box ? 'Box' : String(new.target));
            this.fromArrow = read();
          }

          const instance = new Box();
          document.getElementById('result').textContent = plain() + ':' + instance.fromArrow;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "undefined:Box")?;
    Ok(())
}

#[test]
fn new_target_is_undefined_in_class_static_block() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class C {
            static {
              window.staticNewTarget = String(new.target);
            }
          }

          document.getElementById('result').textContent = window.staticNewTarget;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "undefined")?;
    Ok(())
}

#[test]
fn new_target_outside_function_or_class_body_is_rejected() {
    let err = Harness::from_html("<script>new.target;</script>")
        .expect_err("new.target outside function should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("new.target")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn class_methods_resolve_through_prototype_and_bind_this() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          class Rectangle {
            constructor(height, width) {
              this.height = height;
              this.width = width;
            }

            area() {
              return this.height * this.width;
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            const rectangle = new Rectangle(5, 2);
            document.getElementById('result').textContent = String(rectangle.area());
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "10")?;
    Ok(())
}

#[test]
fn class_declaration_is_block_scoped() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          class Box {
            constructor() {
              this.value = 1;
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            const before = new Box().value;
            let inside = 0;
            {
              class Box {
                constructor() {
                  this.value = 2;
                }
              }
              inside = new Box().value;
            }
            const after = new Box().value;
            document.getElementById('result').textContent = before + ':' + inside + ':' + after;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "1:2:1")?;
    Ok(())
}

#[test]
fn class_constructor_cannot_be_called_without_new() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          class Polygon {
            constructor() {}
          }

          document.getElementById('btn').addEventListener('click', () => {
            Polygon();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("class constructor call without new should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("without 'new'")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn class_extends_supports_super_constructor_and_inherited_methods() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          class Animal {
            constructor(name) {
              this.name = name;
            }

            speak() {
              return 'base:' + this.name;
            }
          }

          class Dog extends Animal {
            constructor(name) {
              super(name);
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            const dog = new Dog('pochi');
            document.getElementById('result').textContent = dog.speak();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "base:pochi")?;
    Ok(())
}

#[test]
fn class_extends_uses_default_super_constructor_and_super_method_calls() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          class Base {
            constructor(name) {
              this.name = name;
            }

            label() {
              return this.name;
            }
          }

          class Child extends Base {
            label() {
              return super.label() + ':child';
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            const child = new Child('neo');
            document.getElementById('result').textContent = child.label();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "neo:child")?;
    Ok(())
}

#[test]
fn super_keyword_class_demo_works_for_constructor_and_method_lookup() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Foo {
            constructor(name) {
              this.name = name;
            }
            getNameSeparator() {
              return '-';
            }
          }

          class FooBar extends Foo {
            constructor(name, index) {
              super(name);
              this.index = index;
            }
            getNameSeparator() {
              return '/';
            }
            getFullName() {
              return this.name + super.getNameSeparator() + this.index;
            }
          }

          const firstFooBar = new FooBar('foo', 1);
          document.getElementById('result').textContent =
            firstFooBar.name + '|' + firstFooBar.getFullName();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "foo|foo-1")?;
    Ok(())
}

#[test]
fn super_keyword_supports_static_methods_getters_and_bracket_access() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Rectangle {
            static logNbSides() {
              return 'I have 4 sides';
            }
          }

          class Square extends Rectangle {
            static logDescription() {
              return `${super.logNbSides()} which are all equal`;
            }
          }

          class Base {
            get baseLabel() {
              return this.name + '-base';
            }
            get score() {
              return this.name.length;
            }
          }

          class Child extends Base {
            constructor(name) {
              super();
              this.name = name;
            }
            readLabel() {
              return super.baseLabel;
            }
            readByKey(key) {
              return super[key];
            }
          }

          const child = new Child('neo');
          document.getElementById('result').textContent =
            Square.logDescription() + '|' + child.readLabel() + '|' + child.readByKey('score');
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "I have 4 sides which are all equal|neo-base|3")?;
    Ok(())
}

#[test]
fn super_property_assignment_sets_on_this_and_can_invoke_super_setter() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class A {
            set y(v) {
              this._y = 'setter:' + v;
            }
          }

          class B extends A {
            setValues() {
              super.x = 1;
              super.y = 2;
              return String(this.hasOwnProperty('x')) + ':' + String(this.x) + ':' + this._y;
            }
          }

          const instance = new B();
          document.getElementById('result').textContent = instance.setValues();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true:1:setter:2")?;
    Ok(())
}

#[test]
fn super_keyword_in_object_literals_works_with_proto_setter() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const obj1 = {
            method1() {
              return 'method 1';
            },
            separator: '-',
          };

          const obj2 = {
            __proto__: obj1,
            method2() {
              return super.method1();
            },
            method3() {
              return 'A' + super['separator'] + 'B';
            },
          };

          document.getElementById('result').textContent = obj2.method2() + '|' + obj2.method3();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "method 1|A-B")?;
    Ok(())
}

#[test]
fn delete_super_property_throws_runtime_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          class Base {
            foo() {}
          }
          class Derived extends Base {
            drop() {
              delete super.foo;
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            const instance = new Derived();
            instance.drop();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("deleting super properties should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(
            msg.contains("Cannot delete super property"),
            "unexpected runtime error: {msg}"
        ),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn class_extends_default_constructor_forwards_all_arguments() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          class Base {
            constructor(a, b, c) {
              this.total = a + b + c;
            }
          }

          class Child extends Base {}

          document.getElementById('btn').addEventListener('click', () => {
            const child = new Child(1, 2, 3);
            document.getElementById('result').textContent = String(child.total);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "6")?;
    Ok(())
}

#[test]
fn class_extends_old_style_constructor_function_and_prototype_methods() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function OldStyleClass() {
            this.someProperty = 1;
          }
          OldStyleClass.prototype.someMethod = function () {
            return this.someProperty + 1;
          };

          class ChildClass extends OldStyleClass {}

          document.getElementById('btn').addEventListener('click', () => {
            const child = new ChildClass();
            document.getElementById('result').textContent = String(child.someMethod());
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "2")?;
    Ok(())
}

#[test]
fn class_extends_accepts_expression_superclass() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <p id='result'></p>
        <script>
          function pick(base) {
            return base;
          }

          class Animal {
            constructor(name) {
              this.name = name;
            }
            label() {
              return this.name;
            }
          }

          class Dog extends pick(Animal) {}

          document.getElementById('btn').addEventListener('click', () => {
            const dog = new Dog('pochi');
            document.getElementById('result').textContent = dog.label();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#btn")?;
    h.assert_text("#result", "pochi")?;
    Ok(())
}

#[test]
fn class_extends_rejects_non_constructible_superclasses() {
    for source in [
        "<script>const parent = () => {}; class Child extends parent {}</script>",
        "<script>function* parent() {}; class Child extends parent {}</script>",
    ] {
        let err = Harness::from_html(source)
            .expect_err("class extends non-constructible callable should fail");
        match err {
            Error::ScriptRuntime(msg) => assert!(msg.contains("not a constructor")),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn class_extends_requires_constructor_superclass() {
    let err =
        Harness::from_html("<script>const parent = {}; class Child extends parent {}</script>")
            .expect_err("class extends non-constructor should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("not a constructor")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn class_extends_null_default_constructor_throws_on_instantiation() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          class NullBase extends null {}

          document.getElementById('btn').addEventListener('click', () => {
            new NullBase();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("extends null default constructor should fail at runtime");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("not a constructor")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn class_extends_null_allows_custom_constructor_return_object() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class NullClass extends null {
            constructor() {
              return { tag: 'ok' };
            }
          }

          const value = new NullClass();
          document.getElementById('result').textContent =
            String(value.tag) + ':' + String(value instanceof NullClass);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ok:false")?;
    Ok(())
}

#[test]
fn super_call_without_derived_superclass_reports_error() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          class Base {
            constructor() {
              super();
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            new Base();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("super() outside derived class should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("derived class constructor")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn class_constructor_rejects_multiple_definitions() {
    let err =
        Harness::from_html("<script>class C { constructor() {} constructor(value) {} }</script>")
            .expect_err("multiple class constructors should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("multiple constructors")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn class_constructor_rejects_async_generator_and_accessor_forms() {
    for source in [
        "<script>class C { async constructor() {} }</script>",
        "<script>class C { *constructor() {} }</script>",
        "<script>class C { get constructor() { return 1; } }</script>",
        "<script>class C { set constructor(value) {} }</script>",
    ] {
        let err = Harness::from_html(source).expect_err("invalid constructor forms should fail");
        match err {
            Error::ScriptParse(msg) => assert!(msg.contains("constructor")),
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn class_constructor_supports_default_parameters() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Person {
            constructor(name = 'Anonymous') {
              this.name = name;
            }
          }

          const anonymous = new Person();
          const otto = new Person('Otto');
          document.getElementById('result').textContent = anonymous.name + ':' + otto.name;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "Anonymous:Otto")?;
    Ok(())
}

#[test]
fn base_class_constructor_ignores_primitive_return_values() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class ParentClass {
            constructor() {
              this.name = 'ParentClass';
              return 1;
            }
          }

          const instance = new ParentClass();
          document.getElementById('result').textContent = instance.name;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "ParentClass")?;
    Ok(())
}

#[test]
fn derived_class_constructor_rejects_non_undefined_primitive_return_value() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          class ParentClass {}

          class ChildClass extends ParentClass {
            constructor() {
              return 1;
            }
          }

          document.getElementById('btn').addEventListener('click', () => {
            new ChildClass();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("derived constructor primitive return should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("only return object or undefined")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn derived_class_constructor_allows_object_return_value() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class ParentClass {}

          class ChildClass extends ParentClass {
            constructor() {
              return { name: 'override' };
            }
          }

          const value = new ChildClass();
          document.getElementById('result').textContent = value.name;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "override")?;
    Ok(())
}

#[test]
fn const_reassignment_throws_runtime_error() {
    let err = Harness::from_html("<script>const number = 42; number = 99;</script>")
        .expect_err("const reassignment should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Assignment to constant variable")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn const_object_property_mutation_is_allowed() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const obj = { key: 'value' };
          obj.key = 'otherValue';
          document.getElementById('result').textContent = obj.key;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "otherValue")?;
    Ok(())
}

#[test]
fn const_declaration_list_supports_initializer_dependencies() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const a = 1, b = a + 1, c = b + 1;
          document.getElementById('result').textContent = `${a}:${b}:${c}`;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:2:3")?;
    Ok(())
}

#[test]
fn lexical_declaration_is_rejected_in_single_statement_if_context() {
    let err = Harness::from_html("<script>if (true) const a = 1;</script>")
        .expect_err("single-statement lexical declaration should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("lexical declaration cannot appear in a single-statement context"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn const_redeclaration_in_same_scope_is_rejected() {
    let err = Harness::from_html("<script>const value = 1; const value = 2;</script>")
        .expect_err("const redeclaration should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("already been declared")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn const_array_destructuring_binding_is_immutable() {
    let err = Harness::from_html("<script>const [a] = [1]; a = 2;</script>")
        .expect_err("const destructuring reassignment should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Assignment to constant variable")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn const_object_destructuring_respects_block_scope() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          const value = 1;
          {
            const { value } = { value: 2 };
          }
          document.getElementById('result').textContent = String(value);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn let_declaration_list_supports_initializer_dependencies() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let a = 1, b = a + 1, c = b + 1;
          document.getElementById('result').textContent = `${a}:${b}:${c}`;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:2:3")?;
    Ok(())
}

#[test]
fn let_reassignment_is_allowed() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let value = 1;
          value = value + 2;
          document.getElementById('result').textContent = String(value);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn let_redeclaration_in_same_scope_is_rejected() {
    match Harness::from_html("<script>let value = 1; let value = 2;</script>") {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("already been declared")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("let redeclaration should fail"),
    }
}

#[test]
fn let_in_tdz_throws_before_initialization() {
    match Harness::from_html("<script>{ foo; let foo = 2; }</script>") {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("before initialization")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("accessing let before initialization should fail"),
    }
}

#[test]
fn typeof_let_in_tdz_throws() {
    match Harness::from_html("<script>{ typeof i; let i = 10; }</script>") {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("before initialization")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("typeof in TDZ should fail"),
    }
}

#[test]
fn typeof_const_in_tdz_throws() {
    match Harness::from_html("<script>{ typeof c; const c = 10; }</script>") {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("before initialization")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("typeof in TDZ for const should fail"),
    }
}

#[test]
fn typeof_class_in_tdz_throws() {
    match Harness::from_html("<script>{ typeof C; class C {} }</script>") {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("before initialization")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("typeof in TDZ for class should fail"),
    }
}

#[test]
fn let_initializer_cannot_reference_shadowed_outer_binding() {
    match Harness::from_html(
        "<script>function test(){ var foo = 33; if (foo) { let foo = foo + 55; } } test();</script>",
    ) {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("before initialization")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("let initializer should not read shadowed outer binding"),
    }
}

#[test]
fn let_declaration_cannot_share_name_with_function_parameter() {
    match Harness::from_html("<script>function foo(a){ let a = 1; } foo(2);</script>") {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("already been declared")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("let declaration and parameter name collision should fail"),
    }
}

#[test]
fn let_declaration_cannot_share_name_with_catch_binding() {
    match Harness::from_html("<script>try { throw 1; } catch (e) { let e = 2; }</script>") {
        Err(Error::ScriptRuntime(msg)) => assert!(msg.contains("already been declared")),
        Err(_) => panic!("unexpected error kind"),
        Ok(_) => panic!("let declaration and catch binding collision should fail"),
    }
}

#[test]
fn private_instance_fields_methods_and_assignment_work() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Counter {
            #count = 1;

            increment() {
              this.#count = this.#count + 1;
            }

            value() {
              return this.#count;
            }
          }

          const counter = new Counter();
          counter.increment();
          counter.increment();
          document.getElementById('result').textContent = String(counter.value());
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "3")?;
    Ok(())
}

#[test]
fn private_in_operator_checks_class_brand() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class C {
            #x = 1;

            hasX(obj) {
              return #x in obj;
            }
          }

          const c = new C();
          document.getElementById('result').textContent =
            String(c.hasX(c)) + ':' + String(c.hasX({}));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "true:false")?;
    Ok(())
}

#[test]
fn private_accessors_support_get_and_set() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Box {
            #value = 0;

            get #decorated() {
              return this.#value;
            }

            set #decorated(v) {
              this.#value = v;
            }

            setValue(v) {
              this.#decorated = v;
            }

            getValue() {
              return this.#decorated;
            }
          }

          const box = new Box();
          box.setValue(7);
          document.getElementById('result').textContent = String(box.getValue());
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "7")?;
    Ok(())
}

#[test]
fn private_static_field_and_method_work_via_constructor_brand() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Token {
            static #seed = 40;
            static #bump() {
              return 2;
            }

            read() {
              return this.constructor.#seed + this.constructor.#bump();
            }
          }

          const token = new Token();
          document.getElementById('result').textContent = String(token.read());
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "42")?;
    Ok(())
}

#[test]
fn private_member_access_rejects_unbranded_receiver() -> Result<()> {
    let html = r#"
        <button id='btn'>run</button>
        <script>
          class C {
            #x = 1;

            read(obj) {
              return obj.#x;
            }
          }

          const c = new C();
          document.getElementById('btn').addEventListener('click', () => {
            c.read({});
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#btn")
        .expect_err("private member access on unbranded receiver should fail");
    match err {
        Error::ScriptRuntime(msg) => assert!(msg.contains("Cannot read private member #x")),
        other => panic!("unexpected error: {other:?}"),
    }
    Ok(())
}

#[test]
fn delete_private_member_is_syntax_error() {
    let err = Harness::from_html(
        "<script>class C { #x = 1; drop(){ delete this.#x; } } new C().drop();</script>",
    )
    .expect_err("deleting private members should fail at parse time");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("private elements cannot be deleted")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn public_class_fields_define_instance_and_static_properties() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class ClassWithField {
            instanceField;
            instanceFieldWithInitializer = 'instance field';
            static staticField;
            static staticFieldWithInitializer = 'static field';
          }

          const instance = new ClassWithField();
          document.getElementById('result').textContent =
            String(instance.instanceField) + ':' +
            String(instance.instanceFieldWithInitializer) + ':' +
            String(ClassWithField.staticField) + ':' +
            String(ClassWithField.staticFieldWithInitializer);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "undefined:instance field:undefined:static field")?;
    Ok(())
}

#[test]
fn public_instance_fields_initialize_before_base_constructor_and_after_super() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Base {
            baseField = 1;

            constructor() {
              this.log = 'base:' + String(this.baseField) + ':' + String(this.derivedField);
            }
          }

          class Derived extends Base {
            derivedField = this.baseField + 1;

            constructor() {
              super();
              this.log = this.log + '|derived:' + String(this.derivedField);
            }
          }

          const instance = new Derived();
          document.getElementById('result').textContent = instance.log;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "base:1:undefined|derived:2")?;
    Ok(())
}

#[test]
fn public_instance_fields_initialize_in_declaration_order() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class C {
            a = 1;
            b = this.c;
            c = this.a + 1;
            d = this.c + 1;
          }

          const instance = new C();
          document.getElementById('result').textContent =
            String(instance.d) + ':' + String(instance.b);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "3:undefined")?;
    Ok(())
}

#[test]
fn public_fields_define_own_properties_without_triggering_base_setter() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Base {
            set field(value) {
              this.setterCalled = true;
            }
          }

          class DerivedWithField extends Base {
            field = 1;
          }

          const instance = new DerivedWithField();
          document.getElementById('result').textContent =
            String(instance.field) + ':' + String(instance.setterCalled);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:undefined")?;
    Ok(())
}

#[test]
fn public_class_fields_reject_constructor_field_name() {
    let err = Harness::from_html("<script>class C { constructor = 1; }</script>")
        .expect_err("field name constructor should fail");
    match err {
        Error::ScriptParse(msg) => assert!(msg.contains("field name cannot be constructor")),
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn public_class_fields_reject_static_prototype_name() {
    for source in [
        "<script>class C { static prototype = 1; }</script>",
        "<script>class C { static prototype() {} }</script>",
        "<script>class C { static get prototype() { return 1; } }</script>",
        "<script>class C { static set prototype(value) {} }</script>",
    ] {
        let err =
            Harness::from_html(source).expect_err("static property named prototype should fail");
        match err {
            Error::ScriptParse(msg) => {
                assert!(msg.contains("static class property name cannot be prototype"))
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

#[test]
fn public_static_fields_are_writable_after_class_definition() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class C {
            static value = 1;
          }

          C.value = C.value + 4;
          document.getElementById('result').textContent = String(C.value);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "5")?;
    Ok(())
}

#[test]
fn class_extends_inherits_static_methods_via_constructor_chain() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Base {
            static label() {
              return 'base';
            }
          }

          class Child extends Base {}
          document.getElementById('result').textContent = Child.label();
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "base")?;
    Ok(())
}

#[test]
fn public_class_fields_support_computed_names_evaluated_once() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let counter = 0;
          function nextKey() {
            counter = counter + 1;
            return 'f' + String(counter);
          }

          class C {
            [nextKey()] = 1;
          }

          document.getElementById('result').textContent =
            String(counter) + ':' +
            String(new C().f1) + ':' +
            String(new C().f1) + ':' +
            String(new C().f2);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1:1:1:undefined")?;
    Ok(())
}

#[test]
fn public_static_field_initializer_can_read_this_constructor() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class C {
            static base = 4;
            static doubled = this.base * 2;
          }

          document.getElementById('result').textContent = String(C.doubled);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "8")?;
    Ok(())
}

#[test]
fn class_static_block_runs_once_at_class_evaluation_time() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let count = 0;
          class C {
            static {
              count = count + 1;
            }
          }

          new C();
          new C();
          document.getElementById('result').textContent = String(count);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "1")?;
    Ok(())
}

#[test]
fn class_static_fields_and_blocks_run_in_declaration_order() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class C {
            static trace = 'a';
            static {
              this.trace = this.trace + 'b';
            }
            static trace2 = C.trace + 'c';
            static {
              this.trace2 = this.trace2 + 'd';
            }
          }

          document.getElementById('result').textContent = C.trace2;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "abcd")?;
    Ok(())
}

#[test]
fn class_static_block_can_call_super_static_method() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class Base {
            static value() {
              return 'base';
            }
          }

          class Child extends Base {
            static label;
            static {
              this.label = super.value() + '-child';
            }
          }

          document.getElementById('result').textContent = Child.label;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "base-child")?;
    Ok(())
}

#[test]
fn class_static_block_var_scope_is_local_and_does_not_leak() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          var y = 'Outer y';
          let before;

          class A {
            static field = 'Inner y';
            static {
              before = String(y);
              var y = this.field;
            }
          }

          document.getElementById('result').textContent = before + ':' + y;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "undefined:Outer y")?;
    Ok(())
}

#[test]
fn class_static_block_initializes_superclass_before_subclass() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          class A {
            static trace = 'A1';
            static {
              this.trace = this.trace + ':A2';
            }
          }

          class B extends A {
            static trace = 'B1';
            static {
              this.trace = A.trace + ':' + this.trace + ':B2';
            }
          }

          document.getElementById('result').textContent = B.trace;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "A1:A2:B1:B2")?;
    Ok(())
}

#[test]
fn class_static_block_rejects_super_call() {
    let err =
        Harness::from_html("<script>class A {} class B extends A { static { super(); } }</script>")
            .expect_err("super() in static block should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("super() is not allowed in class static initialization block"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn class_static_block_rejects_arguments_object() {
    let err = Harness::from_html("<script>class C { static { arguments; } }</script>")
        .expect_err("arguments in static block should fail");
    match err {
        Error::ScriptParse(msg) => {
            assert!(msg.contains("arguments is not allowed in class static initialization block"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn class_static_block_can_share_private_member_access() -> Result<()> {
    let html = r#"
        <p id='result'></p>
        <script>
          let getValue;

          class D {
            #privateField;

            constructor(v) {
              this.#privateField = v;
            }

            static {
              getValue = (d) => d.#privateField;
            }
          }

          document.getElementById('result').textContent = getValue(new D('private'));
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#result", "private")?;
    Ok(())
}
