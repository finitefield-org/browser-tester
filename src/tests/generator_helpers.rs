use super::*;

#[test]
fn generator_instances_support_protocol_tags_and_control_methods() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const GeneratorFunction = (function* () {}).constructor;
            const GeneratorPrototype = GeneratorFunction.prototype.prototype;

            const iter = (function* () {
              yield 1;
              yield 2;
            })();
            const first = iter.next();
            const selfFactory = iter[Symbol.iterator];
            const self = selfFactory();
            const constructorTag = iter.constructor[Symbol.toStringTag];
            const generatorTag = iter[Symbol.toStringTag];
            const prototypeConstructorMatches =
              GeneratorPrototype.constructor === GeneratorFunction.prototype;
            const iteratorInstance = iter instanceof Iterator;

            const returned = iter.return(99);
            const afterReturn = iter.next();

            const thrower = (function* () {
              yield 'x';
              yield 'y';
            })();
            const thrownFirst = thrower.next().value;
            let thrown = '';
            try {
              thrower.throw('boom');
            } catch (error) {
              thrown = error;
            }
            const afterThrowDone = thrower.next().done;

            document.getElementById('out').textContent =
              first.value + ':' + first.done + '|' +
              returned.value + ':' + returned.done + ':' + afterReturn.done + '|' +
              thrownFirst + ':' + thrown + ':' + afterThrowDone + '|' +
              constructorTag + ':' + generatorTag + '|' +
              prototypeConstructorMatches + ':' + iteratorInstance + ':' +
              (typeof selfFactory) + ':' + (self === iter);
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#out",
        "1:false|99:true:true|x:boom:true|GeneratorFunction:Generator|true:true:object:true",
    )?;
    Ok(())
}

#[test]
fn infinite_generator_example_advances_values_lazily() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          const infinite = function* () {
            let index = 0;
            while (true) {
              yield index;
              index += 1;
            }
          };

          document.getElementById('run').addEventListener('click', () => {
            const generator = infinite();
            const first = generator.next();
            const second = generator.next();
            const third = generator.next();
            document.getElementById('out').textContent =
              first.value + ':' + first.done + ',' +
              second.value + ':' + second.done + ',' +
              third.value + ':' + third.done;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "0:false,1:false,2:false")?;
    Ok(())
}

#[test]
fn generator_declaration_is_hoisted_and_yield_star_delegates_values() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const beforeType = typeof declaredLater;
            const iter = declaredLater(10);
            const first = iter.next();
            const second = iter.next();
            const third = iter.next();
            const fourth = iter.next();

            document.getElementById('out').textContent =
              beforeType + '|' +
              first.value + ':' + first.done + ',' +
              second.value + ':' + second.done + ',' +
              third.value + ':' + third.done + ',' +
              fourth.value + ':' + fourth.done;

            function* declaredLater(base = 0) {
              yield base;
              yield* [base + 1, base + 2];
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "function|10:false,11:false,12:false,undefined:true")?;
    Ok(())
}

#[test]
fn yield_star_operator_delegates_to_generators_and_other_iterables() -> Result<()> {
    let html = r#"
        <p id='out'></p>
        <script>
          function* g1() {
            yield 2;
            yield 3;
            yield 4;
          }

          function* g2() {
            yield 1;
            yield* g1();
            yield 5;
          }

          function* g3(...args) {
            yield* [1, 2];
            yield* "34";
            yield* args;
          }

          const a = g2().toArray().join(',');
          const b = g3(5, 6).toArray().join(',');
          document.getElementById('out').textContent = a + '|' + b;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "1,2,3,4,5|1,2,3,4,5,6")?;
    Ok(())
}

#[test]
fn yield_star_expression_returns_delegated_generator_completion_value() -> Result<()> {
    let html = r#"
        <p id='out'></p>
        <script>
          function* g4() {
            yield* [1, 2, 3];
            return "foo";
          }

          function* g5() {
            const g4ReturnValue = yield* g4();
            return g4ReturnValue;
          }

          const gen = g5();
          const first = gen.next();
          const second = gen.next();
          const third = gen.next();
          const fourth = gen.next();
          const fifth = gen.next();

          document.getElementById('out').textContent =
            String(first.value) + ':' + String(first.done) + ',' +
            String(second.value) + ':' + String(second.done) + ',' +
            String(third.value) + ':' + String(third.done) + ',' +
            String(fourth.value) + ':' + String(fourth.done) + ',' +
            String(fifth.value) + ':' + String(fifth.done);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "1:false,2:false,3:false,foo:true,undefined:true")?;
    Ok(())
}

#[test]
fn yield_star_throws_when_operand_is_not_iterable() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <script>
          function* bad() {
            yield* 123;
          }

          document.getElementById('run').addEventListener('click', () => {
            const iter = bad();
            iter.next();
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    let err = h
        .click("#run")
        .expect_err("yield* over non-iterable should fail");
    match err {
        Error::ScriptRuntime(msg) => {
            assert!(msg.contains("expected an array-like or iterable source"))
        }
        other => panic!("unexpected yield* non-iterable error: {other:?}"),
    }
    Ok(())
}

#[test]
fn function_line_break_before_generator_star_is_allowed() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            function
            * splitDeclaration() {
              yield 'ok';
            }

            const iter = splitDeclaration();
            const first = iter.next();
            document.getElementById('out').textContent =
              (typeof splitDeclaration) + ':' + first.value + ':' + first.done;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "function:ok:false")?;
    Ok(())
}

#[test]
fn generator_declarations_are_not_constructable() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          function* makeValue() {
            yield 1;
          }

          document.getElementById('run').addEventListener('click', () => {
            let out = 'no-error';
            try {
              new makeValue();
            } catch (error) {
              out = String(error).includes('not a constructor') ? 'not-constructable' : String(error);
            }
            document.getElementById('out').textContent = out;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "not-constructable")?;
    Ok(())
}

#[test]
fn generator_function_expression_can_be_assigned_and_iterated() -> Result<()> {
    let html = r#"
        <p id='out'></p>
        <script>
          const foo = function* () {
            yield 'a';
            yield 'b';
            yield 'c';
          };

          let str = '';
          for (const val of foo()) {
            str += val;
          }

          document.getElementById('out').textContent = str;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "abc")?;
    Ok(())
}

#[test]
fn yield_operator_pauses_and_resumes_generator_iteration() -> Result<()> {
    let html = r#"
        <p id='out'></p>
        <script>
          function* foo(index) {
            while (index < 2) {
              yield index;
              index++;
            }
          }

          const iterator = foo(0);
          const first = iterator.next();
          const second = iterator.next();
          const done = iterator.next();
          document.getElementById('out').textContent =
            String(first.value) + ':' + String(first.done) + '|' +
            String(second.value) + ':' + String(second.done) + '|' +
            String(done.value) + ':' + String(done.done);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "0:false|1:false|undefined:true")?;
    Ok(())
}

#[test]
fn yield_without_expression_yields_undefined_value() -> Result<()> {
    let html = r#"
        <p id='out'></p>
        <script>
          function* sample() {
            yield;
            yield 1;
          }

          const iter = sample();
          const first = iter.next();
          const second = iter.next();
          const third = iter.next();
          document.getElementById('out').textContent =
            String(first.value) + ':' + String(first.done) + '|' +
            String(second.value) + ':' + String(second.done) + '|' +
            String(third.value) + ':' + String(third.done);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "undefined:false|1:false|undefined:true")?;
    Ok(())
}

#[test]
fn named_generator_function_expression_uses_local_name_binding() -> Result<()> {
    let html = r#"
        <p id='out'></p>
        <script>
          const gen = function* localName(value = 6) {
            yield typeof localName;
            yield value * value;
          };

          const iter = gen();
          const first = iter.next();
          const second = iter.next();
          const done = iter.next();
          document.getElementById('out').textContent =
            first.value + ':' + second.value + ':' + done.done + ':' + typeof localName;
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "function:36:true:undefined")?;
    Ok(())
}

#[test]
fn named_generator_function_expression_name_binding_is_read_only() -> Result<()> {
    let html = r#"
        <p id='out'></p>
        <script>
          const fn = function* localName() {
            let status = 'no-throw';
            try {
              localName = 1;
            } catch (error) {
              status = 'threw';
            }
            yield typeof localName + ':' + status;
          };

          const iter = fn();
          document.getElementById('out').textContent = String(iter.next().value);
        </script>
        "#;

    let h = Harness::from_html(html)?;
    h.assert_text("#out", "function:threw")?;
    Ok(())
}

#[test]
fn function_star_expression_statement_without_name_is_rejected() {
    let err = Harness::from_html("<script>function* () { yield 1; }</script>")
        .expect_err("anonymous function* at statement start should parse as declaration");
    match err {
        Error::ScriptParse(msg) => {
            assert!(
                msg.contains("function declaration requires a function name")
                    || msg.contains("expected function name")
            )
        }
        other => panic!("unexpected error: {other:?}"),
    }
}
