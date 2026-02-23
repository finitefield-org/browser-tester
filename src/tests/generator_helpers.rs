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
