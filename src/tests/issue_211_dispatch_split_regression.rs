use super::*;

#[test]
fn split_object_callable_dispatch_keeps_platform_and_dom_collection_receivers_working() -> Result<()>
{
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const parseHtml = Document.parseHTML;
            const parsed = parseHtml('<div id="x" class="alpha beta" data-k="v"></div>');
            const root = parsed.getElementById('x');
            const attrs = root.attributes;
            const attrEntries = Array.from(attrs.entries.call(attrs))
              .map((pair) => pair[1].name)
              .sort()
              .join(',');
            const classEntries = Array.from(root.classList.entries.call(root.classList))
              .map((pair) => pair[0] + ':' + pair[1])
              .join(',');

            document.getElementById('result').textContent = [
              root.getAttribute('data-k'),
              attrEntries,
              classEntries
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "v|class,data-k,id|0:alpha,1:beta")?;
    Ok(())
}

#[test]
fn split_object_callable_dispatch_keeps_text_codec_and_iterator_calls_working() -> Result<()> {
    let html = r#"
        <button id='run'>run</button>
        <p id='result'></p>
        <script>
          document.getElementById('run').addEventListener('click', () => {
            const encoder = new TextEncoder();
            const dest = new Uint8Array(8);
            const status = TextEncoder.prototype.encodeInto.call(encoder, 'A€', dest);

            const segmenter = new Intl.Segmenter('fr', { granularity: 'word' });
            const segments = Intl.Segmenter.prototype.segment.call(segmenter, 'Que ma');
            const iterator = segments[Symbol.iterator].call(segments);
            const first = iterator.next.call(iterator);
            const second = iterator.next.call(iterator);

            document.getElementById('result').textContent = [
              status.read,
              status.written,
              Array.from(dest).slice(0, status.written).join(','),
              first.value.segment,
              second.value.segment,
              first.done,
              second.done
            ].join('|');
          });
        </script>
        "#;

    let mut h = Harness::from_html(html)?;
    h.click("#run")?;
    h.assert_text("#result", "2|4|65,226,130,172|Que| |false|false")?;
    Ok(())
}

#[test]
fn split_script_ast_facade_keeps_parser_shapes_visible() -> Result<()> {
    let stmts = test_support::parse_block_statements(
        r#"
          class ParserBox extends Base {
            static {
              const parsed = Document.parseHTML('<div id="x"></div>');
              window.__parsed = parsed.getElementById('x').id;
            }
          }

          async function* make(source) {
            const [first, ...rest] = source;
            for await (const entry of source) {
              yield entry;
            }
            return first + rest.length;
          }
        "#,
    )?;

    let debug = format!("{stmts:#?}");
    assert!(debug.contains("ClassDecl"));
    assert!(debug.contains("ArrayDestructureAssign"));
    assert!(debug.contains("ForAwaitOf"));
    assert!(debug.contains("parseHTML"));
    Ok(())
}
