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
        Error::ScriptParse(msg) => assert!(msg.contains("non-generator") || msg.contains("non-async")),
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
fn function_declaration_invocation_syntax_is_rejected() {
    let err = Harness::from_html("<script>function foo() { console.log('foo'); }();</script>")
        .expect_err("function declaration invocation form should fail");
    match err {
        Error::ScriptParse(msg) => assert!(!msg.is_empty()),
        other => panic!("unexpected error: {other:?}"),
    }
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
