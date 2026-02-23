use super::*;

#[test]
fn async_generator_instances_support_protocol_tags_and_control_methods() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const AsyncGeneratorFunction = (async function* () {}).constructor;
            const AsyncGeneratorPrototype = AsyncGeneratorFunction.prototype.prototype;

            const iter = (async function* () {
              yield Promise.resolve(1);
              yield 2;
            })();
            const selfFactory = iter[Symbol.asyncIterator];
            const self = selfFactory();
            const constructorTag = iter.constructor[Symbol.toStringTag];
            const generatorTag = iter[Symbol.toStringTag];
            const prototypeConstructorMatches =
              AsyncGeneratorPrototype.constructor === AsyncGeneratorFunction.prototype;

            const firstPromise = iter.next();
            const returnPromise = iter.return(Promise.resolve(99));
            const afterReturnPromise = iter.next();

            const thrower = (async function* () {
              yield 'x';
              yield 'y';
            })();
            const thrownFirstPromise = thrower.next();
            const thrownPromise = thrower.throw('boom');
            const thrownReasonPromise = thrownPromise.then(
              () => 'noerr',
              (reason) => reason
            );
            const afterThrowPromise = thrower.next();

            Promise.all([
              firstPromise,
              returnPromise,
              afterReturnPromise,
              thrownFirstPromise,
              thrownReasonPromise,
              afterThrowPromise,
            ]).then((results) => {
              const first = results[0];
              const returned = results[1];
              const afterReturn = results[2];
              const thrownFirst = results[3];
              const thrownReason = results[4];
              const afterThrow = results[5];

              document.getElementById('out').textContent =
                first.value + ':' + first.done + '|' +
                returned.value + ':' + returned.done + ':' + afterReturn.done + '|' +
                thrownFirst.value + ':' + thrownReason + ':' + afterThrow.done + '|' +
                constructorTag + ':' + generatorTag + '|' +
                prototypeConstructorMatches + ':' + (self === iter) + '|' +
                (typeof firstPromise) + ':' +
                (typeof returnPromise) + ':' +
                (typeof thrownPromise);
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text(
        "#out",
        "1:false|99:true:true|x:boom:true|AsyncGeneratorFunction:AsyncGenerator|true:true|object:object:object",
    )?;
    Ok(())
}

#[test]
fn async_generator_iteration_resolves_delayed_promises() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          const delayedValue = function(time, value) {
            return new Promise((resolve) => {
              setTimeout(() => resolve(value), time);
            });
          };

          const generate = async function* () {
            yield delayedValue(20, 1);
            yield delayedValue(10, 2);
            yield delayedValue(5, 3);
            yield delayedValue(3, 4);
            yield delayedValue(2, 5);
            yield delayedValue(1, 6);
          };

          document.getElementById('run').addEventListener('click', () => {
            const generator = generate();
            Promise.all([
              generator.next(),
              generator.next(),
              generator.next(),
              generator.next(),
              generator.next(),
              generator.next(),
              generator.next(),
            ]).then((results) => {
              const values = results
                .slice(0, 6)
                .map((result) => result.value)
                .join(',');
              const done = results[6].done;
              document.getElementById('out').textContent = values + '|' + done;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "")?;
    h.flush()?;
    h.assert_text("#out", "1,2,3,4,5,6|true")?;
    Ok(())
}

#[test]
fn async_generator_declaration_is_hoisted_and_yields_promises() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const beforeType = typeof declaredLater;
            document.getElementById('out').textContent = beforeType;
            const iter = declaredLater(4);
            const first = iter.next();
            first.then((result) => {
              document.getElementById('out').textContent =
                document.getElementById('out').textContent +
                ':' + result.value + ':' + result.done;
            });

            async function
            * declaredLater(step = 1) {
              yield await Promise.resolve(step);
            }
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.flush()?;
    h.assert_text("#out", "function:4:false")?;
    Ok(())
}

#[test]
fn async_line_break_before_function_star_is_treated_as_asi() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const async = 'marker';
            async
            function* splitDeclaration() {
              yield 'ok';
            }

            const iter = splitDeclaration();
            const first = iter.next();
            document.getElementById('out').textContent =
              (typeof iter) + ':' + first.value + ':' + first.done + ':' + async;
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "object:ok:false:marker")?;
    Ok(())
}

#[test]
fn async_generator_next_rejects_when_yielded_promise_rejects() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          async function* failingGenerator() {
            yield Promise.reject('boom');
          }

          document.getElementById('run').addEventListener('click', () => {
            const iter = failingGenerator();
            iter.next().then(
              () => {
                document.getElementById('out').textContent = 'fulfilled';
              },
              (reason) => {
                document.getElementById('out').textContent = 'rejected:' + reason;
              }
            );
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "rejected:boom")?;
    Ok(())
}

#[test]
fn async_function_star_expression_can_be_used_in_expression_contexts() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          async function joinAll(factory) {
            let text = '';
            for await (const value of factory()) {
              text += value;
            }
            return text;
          }

          document.getElementById('run').addEventListener('click', () => {
            const iife = (async function* (x) {
              yield await Promise.resolve(x);
              yield await Promise.resolve(x + 1);
            })(10);

            Promise.all([
              joinAll(async function* () {
                yield await Promise.resolve('a');
                yield await Promise.resolve('b');
                yield await Promise.resolve('c');
              }),
              iife.next(),
              iife.next(),
              iife.next(),
            ]).then((values) => {
              const joined = values[0];
              const first = values[1];
              const second = values[2];
              const done = values[3];
              document.getElementById('out').textContent =
                joined + '|' +
                first.value + ',' + second.value + ',' + done.done;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "abc|10,11,true")?;
    Ok(())
}

#[test]
fn named_async_function_star_expression_uses_local_name_binding() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='out'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const self = 'outer';
            const gen = async function* self(value = 6) {
              yield typeof self;
              yield await Promise.resolve(value * value);
            };

            const iter = gen();
            Promise.all([iter.next(), iter.next(), iter.next()]).then(([first, second, done]) => {
              document.getElementById('out').textContent =
                self + '|' +
                first.value + ':' +
                second.value + ':' +
                done.done;
            });
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#out", "outer|function:36:true")?;
    Ok(())
}

#[test]
fn async_function_star_expression_statement_without_name_is_rejected() {
    let err = Harness::from_html("<script>async function* () { yield 1; }</script>")
        .expect_err("anonymous async function* at statement start should parse as declaration");
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
