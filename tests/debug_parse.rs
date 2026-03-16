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
    h.assert_text("#result", "active").unwrap();
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
    h.assert_text("#result", "start:active").unwrap();
}

#[test]
fn debug_while_loop() {
    let html = r#"
    <button id='btn'>run</button>
    <p id='result'></p>
    <script>
      document.getElementById('btn').addEventListener('click', () => {
        let counter = 0;
        let text = '';
        while (counter < 3) {
          text += 'x';
          counter = counter + 1;
        };
        document.getElementById('result').textContent = text + ':' + counter;
      });
    </script>
    "#;

    let mut h = Harness::from_html(html).unwrap();
    h.click("#btn").unwrap();
    h.assert_text("#result", "xxx:3").unwrap();
}

#[test]
fn debug_for_loop() {
    let html = r#"
    <button id='btn'>run</button>
    <p id='result'></p>
    <script>
      document.getElementById('btn').addEventListener('click', () => {
        let text = '';
        for (let i = 0; i < 3; i = i + 1) {
          text += 'y';
        };
        document.getElementById('result').textContent = text;
      });
    </script>
    "#;

    let mut h = Harness::from_html(html).unwrap();
    h.click("#btn").unwrap();
    h.assert_text("#result", "yyy").unwrap();
}

#[test]
fn debug_if_block_and_next_statement_without_semicolon() {
    let html = r#"
    <button id='btn'>run</button>
    <p id='result'></p>
    <script>
      document.getElementById('btn').addEventListener('click', () => {
        let text = '';
        if (true) {
          text += 'x';
        }
        text += 'y';
        document.getElementById('result').textContent = text;
      });
    </script>
    "#;

    let mut h = Harness::from_html(html).unwrap();
    h.click("#btn").unwrap();
    h.assert_text("#result", "xy").unwrap();
}

#[test]
fn debug_parenthesized_formula_trace() {
    let html = r#"
    <div>
      <input id="formula" value="Al2(SO4)3" />
      <button id="go" type="button">go</button>
      <div id="out"></div>
    </div>
    <script>
    (() => {
      const weights = { Al: 26.982, S: 32.06, O: 15.999 };
      const input = document.getElementById("formula");
      const out = document.getElementById("out");

      function parserError(message) {
        return { message };
      }

      function createParser(source) {
        let index = 0;
        const trace = [];

        function current() {
          const ch = source[index] || "";
          trace.push("current:" + index + ":" + (ch || "<eof>"));
          return ch;
        }

        function consume() {
          const char = source[index] || "";
          trace.push("consume:" + index + ":" + (char || "<eof>"));
          index += 1;
          return char;
        }

        function isDigit(char) {
          return /[0-9]/.test(char);
        }

        function isUpper(char) {
          return /[A-Z]/.test(char);
        }

        function isLower(char) {
          return /[a-z]/.test(char);
        }

        function parseNumber() {
          const start = index;
          let sawDigit = false;
          while (isDigit(current())) {
            sawDigit = true;
            consume();
          }
          const raw = source.slice(start, index);
          if (!sawDigit) throw parserError("invalid number");
          return { raw, value: Number(raw) };
        }

        function parseOptionalMultiplier() {
          if (isDigit(current())) return parseNumber();
          return { raw: "", value: 1 };
        }

        function parseElementSymbol() {
          const first = current();
          if (!isUpper(first)) throw parserError("invalid symbol");
          let symbol = consume();
          if (isLower(current())) symbol += consume();
          if (!weights[symbol]) throw parserError("unknown element");
          return symbol;
        }

        function parseBracketGroup() {
          const open = consume();
          const close = open === "(" ? ")" : "]";
          trace.push("bracket-open:" + open + ":index=" + index + ":close=" + close);
          const inner = parseSequence(close, 1);
          trace.push("after-inner:index=" + index + ":close=" + close + ":current=" + (current() || "<eof>"));
          if (current() !== close) throw parserError("Bracket mismatch detected.");
          consume();
          const multiplier = parseOptionalMultiplier();
          return {
            counts: {},
            order: inner.order.slice(),
            normalized: open + inner.normalized + close + multiplier.raw
          };
        }

        function parseElementGroup() {
          const symbol = parseElementSymbol();
          const count = parseOptionalMultiplier();
          return {
            counts: { [symbol]: count.value },
            order: [symbol],
            normalized: symbol + count.raw
          };
        }

        function parseGroup() {
          const char = current();
          trace.push("parseGroup:index=" + index + ":char=" + (char || "<eof>"));
          if (char === "(" || char === "[") return parseBracketGroup();
          return parseElementGroup();
        }

        function parseSequence(stopChar, nesting) {
          trace.push("enter-seq:stop=" + (stopChar || "<empty>") + ":index=" + index);
          const order = [];
          let normalized = "";
          while (index < source.length && current() !== stopChar) {
            trace.push("loop-seq:stop=" + (stopChar || "<empty>") + ":index=" + index + ":current=" + (current() || "<eof>"));
            if (current() === ")" || current() === "]") throw parserError("unexpected close");
            const group = parseGroup(nesting);
            group.order.forEach((item) => {
              if (!order.includes(item)) order.push(item);
            });
            normalized += group.normalized;
          }
          trace.push("exit-seq:stop=" + (stopChar || "<empty>") + ":index=" + index + ":current=" + (current() || "<eof>"));
          return { counts: {}, order, normalized };
        }

        function parseFragment() {
          const body = parseSequence("", 1);
          return { order: body.order.slice(), normalized: body.normalized, trace };
        }

        return { parseFragment };
      }

      document.getElementById("go").addEventListener("click", () => {
        try {
          const parsed = createParser(input.value).parseFragment();
          out.textContent = parsed.normalized + "|" + parsed.trace.join(";");
        } catch (error) {
          out.textContent = error && error.message ? error.message : "unknown";
        }
      });
    })();
    </script>
    "#;

    let mut h = Harness::from_html(html).unwrap();
    h.click("#go").unwrap();
    println!("trace dom: {}", h.dump_dom("#out").unwrap());
}


#[test]
fn debug_recursive_closure_stop_char_and_index() {
    let html = r#"
    <button id="go" type="button">go</button>
    <div id="out"></div>
    <script>
    (() => {
      function make(source) {
        let index = 0;

        function current() {
          return source[index] || "";
        }

        function consume() {
          const char = source[index] || "";
          index += 1;
          return char;
        }

        function parseSequence(stopChar) {
          let seen = "";
          while (index < source.length && current() !== stopChar) {
            seen += consume();
          }
          return "seen=" + seen + "|stop=" + stopChar + "|curr=" + (current() || "<eof>") + "|index=" + index;
        }

        function parseBracketGroup() {
          const open = consume();
          const close = open === "(" ? ")" : "]";
          const before = String(close);
          const inner = parseSequence(close);
          return "before=" + before + "|after=" + (current() || "<eof>") + "|close=" + close + "|index=" + index + "|" + inner;
        }

        return parseBracketGroup();
      }

      document.getElementById("go").addEventListener("click", () => {
        document.getElementById("out").textContent = make("(SO4)3");
      });
    })();
    </script>
    "#;

    let mut h = Harness::from_html(html).unwrap();
    h.click("#go").unwrap();
    println!("recursive closure dom: {}", h.dump_dom("#out").unwrap());
}
