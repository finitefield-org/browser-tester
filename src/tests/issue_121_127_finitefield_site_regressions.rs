use super::*;

#[test]
fn issue_121_global_file_constructor_supports_new_file_instances() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        try {
          const f = new File(['ab', 'cd'], 'sample.csv', {
            type: 'text/csv',
            lastModified: 1234,
          });
          f.arrayBuffer().then((buf) => {
            document.getElementById('out').textContent = [
              typeof File,
              String(window.File === File),
              f.name,
              f.type,
              String(f.size),
              String(f.lastModified),
              String(buf.byteLength),
            ].join('|');
          });
        } catch (err) {
          document.getElementById('out').textContent =
            'err:' + String(err && err.message ? err.message : err);
        }
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "function|true|sample.csv|text/csv|4|1234|4")?;
    Ok(())
}

#[test]
fn issue_121_input_files_accepts_file_list_assignment_from_data_transfer() -> Result<()> {
    let html = r#"
      <input id='a' type='file'>
      <input id='b' type='file'>
      <p id='out'></p>
      <script>
        const out = document.getElementById('out');
        const file = new File(['sample'], 'sample.csv', { type: 'text/csv' });
        const dt = new DataTransfer();
        dt.items.add(file);

        const a = document.getElementById('a');
        const b = document.getElementById('b');
        a.files = dt.files;
        b['files'] = dt.files;

        const fa = a.files[0];
        const fb = b.files[0];
        fa.arrayBuffer().then((ab) => {
          fb.arrayBuffer().then((bb) => {
            out.textContent = [
              String(a.files.length),
              String(b.files.length),
              fa.name,
              fa.type,
              String(ab.byteLength),
              String(bb.byteLength),
            ].join('|');
          });
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "1|1|sample.csv|text/csv|6|6")?;
    Ok(())
}

#[test]
fn issue_122_124_intl_number_format_fixed_digits_remain_stable() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const f2 = new Intl.NumberFormat('en', {
          minimumFractionDigits: 2,
          maximumFractionDigits: 2,
        });
        const f1 = new Intl.NumberFormat('en', {
          minimumFractionDigits: 1,
          maximumFractionDigits: 1,
        });
        document.getElementById('out').textContent = [
          f2.format(28.000000000000004),
          f2.format(43.55555555555556),
          f1.format(11.5),
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "28.00|43.56|11.5")?;
    Ok(())
}

#[test]
fn issue_123_external_script_src_data_url_executes_before_inline_script() -> Result<()> {
    let html = r#"
      <div id='out'></div>
      <script src='data:text/javascript,window.__probe_value=123'></script>
      <script>
        document.getElementById('out').textContent = String(window.__probe_value || 0);
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "123")?;
    Ok(())
}

#[test]
fn worker_bootstrap_from_function_to_string_runs_worker_script() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        function workerMain() {
          self.onmessage = (event) => {
            postMessage('pong:' + String(event.data));
          };
        }
        const blob = new Blob([`(${workerMain.toString()})()`], {
          type: 'text/javascript',
        });
        const url = URL.createObjectURL(blob);
        const worker = new Worker(url);
        URL.revokeObjectURL(url);
        worker.onmessage = (event) => {
          document.getElementById('out').textContent = String(event.data || '');
        };
        worker.postMessage('ping');
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "pong:ping")?;
    Ok(())
}

#[test]
fn function_to_string_returns_marker_for_user_defined_function() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        function probe() {}
        document.getElementById('out').textContent = probe.toString();
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    let dom = harness.dump_dom("#out")?;
    assert!(
        dom.contains("__bt_function_ref__("),
        "function.toString should return marker, dom={dom}",
    );
    Ok(())
}

#[test]
fn issue_125_mock_jpeg_and_png_files_work_with_create_image_bitmap() -> Result<()> {
    fn assert_decodes_image(name: &str, mime_type: &str) -> Result<()> {
        let html = r#"
          <input id='file' type='file' accept='image/jpeg,image/png'>
          <p id='out'></p>
          <script>
            const input = document.getElementById('file');
            const out = document.getElementById('out');
            input.addEventListener('change', () => {
              const file = input.files[0];
              createImageBitmap(file)
                .then((bmp) => {
                  out.textContent = `ok:${file.type}:${bmp.width}x${bmp.height}`;
                })
                .catch((err) => {
                  out.textContent = 'err:' + String(err && err.message ? err.message : err);
                });
            });
          </script>
        "#;

        let mut harness = Harness::from_html(html)?;
        let mut file = MockFile::new(name);
        file.size = 2048;
        file.mime_type = mime_type.to_string();
        harness.set_input_files("#file", &[file])?;
        harness.flush()?;
        harness.assert_text("#out", &format!("ok:{mime_type}:1x1"))?;
        Ok(())
    }

    assert_decodes_image("sample.jpg", "image/jpeg")?;
    assert_decodes_image("sample.png", "image/png")?;
    Ok(())
}

#[test]
fn issue_126_create_image_bitmap_settles_after_create_object_url_for_same_file() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/png'>
      <p id='out'></p>
      <script>
        const out = document.getElementById('out');
        const input = document.getElementById('file');

        input.addEventListener('change', () => {
          const file = input.files[0];
          const url = URL.createObjectURL(file);
          let settled = false;

          createImageBitmap(file)
            .then(() => {
              settled = true;
              out.textContent = url.startsWith('blob:bt-') ? 'ok' : 'bad-url';
            })
            .catch((err) => {
              settled = true;
              out.textContent = 'err:' + String(err && err.message ? err.message : err);
            });

          setTimeout(() => {
            if (!settled) out.textContent = 'pending';
          }, 0);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.png");
    file.size = 1024;
    file.mime_type = "image/png".to_string();
    harness.set_input_files("#file", &[file])?;
    harness.flush()?;
    harness.assert_text("#out", "ok")?;
    Ok(())
}

#[test]
fn image_pipeline_object_url_then_create_image_bitmap_from_input_file_is_supported() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/jpeg,image/png'>
      <p id='out'></p>
      <script>
        const out = document.getElementById('out');
        const input = document.getElementById('file');
        input.addEventListener('change', () => {
          const file = input.files[0];
          const url = URL.createObjectURL(file);
          createImageBitmap(file)
            .then((bmp) => {
              out.textContent = `ok:${file.type}:${bmp.width}x${bmp.height}:${String(url).startsWith('blob:bt-')}`;
            })
            .catch((err) => {
              out.textContent = `err:${file.type}:${String(err && err.message ? err.message : err)}`;
            });
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.jpg");
    file.size = 1024;
    file.mime_type = "image/jpeg".to_string();
    harness.set_input_files("#file", &[file])?;
    harness.flush()?;
    harness.assert_text("#out", "ok:image/jpeg:1x1:true")?;
    Ok(())
}

#[test]
fn image_queue_item_creation_path_succeeds_for_mock_jpeg_file() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/jpeg,image/png'>
      <p id='out'></p>
      <script>
        function loadImage(url) {
          return new Promise((resolve, reject) => {
            const image = new Image();
            image.onload = () => resolve(image);
            image.onerror = () => reject(new Error('imgerr'));
            image.src = url;
          });
        }

        async function decodeSource(file, objectUrl) {
          if (window.createImageBitmap) {
            try {
              const bitmap = await createImageBitmap(file);
              return { width: bitmap.width, height: bitmap.height };
            } catch (error) {
              window.__bmpErr = String(error && error.message ? error.message : error);
            }
          }
          const image = await loadImage(objectUrl);
          return { width: image.naturalWidth, height: image.naturalHeight };
        }

        async function createQueueItem(file) {
          const originalUrl = URL.createObjectURL(file);
          const meta = await decodeSource(file, originalUrl);
          return {
            id: `id-${Date.now()}-${Math.random().toString(16).slice(2)}`,
            width: meta.width,
            height: meta.height,
            type: file.type,
          };
        }

        document.getElementById('file').addEventListener('change', async () => {
          const out = document.getElementById('out');
          const file = document.getElementById('file').files[0];
          try {
            const item = await createQueueItem(file);
            out.textContent = `ok:${item.type}:${item.width}x${item.height}:${window.__bmpErr || ''}`;
          } catch (error) {
            out.textContent = `err:${String(error && error.message ? error.message : error)}:${window.__bmpErr || ''}`;
          }
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.jpg");
    file.size = 1_048_576;
    file.mime_type = "image/jpeg".to_string();
    harness.set_input_files("#file", &[file])?;
    harness.flush()?;
    harness.assert_text("#out", "ok:image/jpeg:1x1:")?;
    Ok(())
}

#[test]
fn array_from_input_files_preserves_mock_file_objects() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/jpeg,image/png'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('file');
        const out = document.getElementById('out');
        input.addEventListener('change', () => {
          const files = Array.from(input.files || []);
          const first = files[0];
          out.textContent = [
            String(files.length),
            typeof first,
            first && first.name ? first.name : '',
            first && first.type ? first.type : '',
            first && first.size ? String(first.size) : '',
          ].join('|');
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.jpg");
    file.size = 1024;
    file.mime_type = "image/jpeg".to_string();
    harness.set_input_files("#file", &[file])?;
    harness.flush()?;
    harness.assert_text("#out", "1|object|sample.jpg|image/jpeg|1024")?;
    Ok(())
}

#[test]
fn async_try_finally_preserves_returned_object_value() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        async function readMeta() {
          const source = { width: 3, height: 4 };
          try {
            return {
              width: source.width,
              height: source.height,
            };
          } finally {
            source.width = 99;
          }
        }

        readMeta().then((meta) => {
          document.getElementById('out').textContent = [
            typeof meta,
            String(meta && meta.width),
            String(meta && meta.height),
          ].join('|');
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "object|3|4")?;
    Ok(())
}

#[test]
fn async_try_finally_return_with_regex_property_keeps_object() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        function cleanup(_value) {}
        async function readMeta(file) {
          const source = await Promise.resolve({ width: 3, height: 4 });
          try {
            return {
              width: source.width,
              height: source.height,
              hasAlpha: /png|webp/i.test(file.type || file.name || ""),
            };
          } finally {
            cleanup(source);
          }
        }

        readMeta({ type: "image/png", name: "a.png" }).then((meta) => {
          document.getElementById('out').textContent = [
            typeof meta,
            String(meta && meta.width),
            String(meta && meta.height),
            String(meta && meta.hasAlpha),
          ].join('|');
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "object|3|4|true")?;
    Ok(())
}

#[test]
fn image_resizer_style_read_image_meta_returns_object() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        function cleanupDecodedSource(source) {
          if (!source) return;
          if (source.kind === "bitmap" && source.value && typeof source.value.close === "function") {
            source.value.close();
          }
        }

        async function decodeSource(_file, _objectUrl) {
          return {
            kind: "bitmap",
            value: { close() {} },
            width: 1,
            height: 2,
          };
        }

        async function readImageMeta(file, objectUrl) {
          const source = await decodeSource(file, objectUrl);
          try {
            return {
              width: source.width,
              height: source.height,
              hasAlpha: /png|webp/i.test(file.type || file.name || ""),
            };
          } finally {
            cleanupDecodedSource(source);
          }
        }

        async function run() {
          const meta = await readImageMeta({ type: "image/jpeg", name: "a.jpg" }, "blob:bt-1");
          document.getElementById('out').textContent = [
            typeof meta,
            String(meta && meta.width),
            String(meta && meta.height),
            String(meta && meta.hasAlpha),
          ].join('|');
        }

        run();
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "object|1|2|false")?;
    Ok(())
}

#[test]
fn image_resizer_template_queue_item_flow_returns_meta_dimensions() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/jpeg,image/png'>
      <p id='out'></p>
      <script>
        async function createQueueItem(file) {
          const originalUrl = URL.createObjectURL(file);
          const meta = await readImageMeta(file, originalUrl);
          return {
            id: "id-1",
            file,
            name: file.name || "image",
            mime: file.type || "",
            originalUrl,
            inputSize: file.size || 0,
            inputWidth: meta.width,
            inputHeight: meta.height,
            hasAlpha: meta.hasAlpha,
          };
        }

        async function readImageMeta(file, objectUrl) {
          const source = await decodeSource(file, objectUrl);
          try {
            return {
              width: source.width,
              height: source.height,
              hasAlpha: /png|webp/i.test(file.type || file.name || ""),
            };
          } finally {
            cleanupDecodedSource(source);
          }
        }

        async function decodeSource(file, objectUrl) {
          if (window.createImageBitmap) {
            try {
              const bitmap = await createImageBitmap(file);
              return { kind: "bitmap", value: bitmap, width: bitmap.width, height: bitmap.height };
            } catch (_error) {
            }
          }
          const image = await loadImage(objectUrl);
          return { kind: "image", value: image, width: image.naturalWidth, height: image.naturalHeight };
        }

        function cleanupDecodedSource(source) {
          if (!source) return;
          if (source.kind === "bitmap" && source.value && typeof source.value.close === "function") {
            source.value.close();
          }
        }

        function loadImage(_url) {
          return Promise.resolve({ naturalWidth: 5, naturalHeight: 6 });
        }

        const input = document.getElementById('file');
        const out = document.getElementById('out');
        input.addEventListener('change', async () => {
          const file = input.files[0];
          try {
            const item = await createQueueItem(file);
            out.textContent = `ok:${item.inputWidth}x${item.inputHeight}:${item.hasAlpha}`;
          } catch (error) {
            out.textContent = `err:${String(error && error.message ? error.message : error)}`;
          }
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.jpg");
    file.size = 1024;
    file.mime_type = "image/jpeg".to_string();
    harness.set_input_files("#file", &[file])?;
    harness.flush()?;
    harness.assert_text("#out", "ok:1x1:false")?;
    Ok(())
}

#[test]
fn regex_match_before_async_functions_does_not_break_following_await_flow() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/jpeg,image/png'>
      <p id='out'></p>
      <script>
        function validateImageFile(file) {
          const mime = String(file.type || "").toLowerCase();
          const name = String(file.name || "").toLowerCase();
          const extMatch = name.match(/\.([a-z0-9]+)$/);
          const ext = extMatch ? extMatch[1] : "";
          if (mime && ext === "jpeg") {
            return { ok: true };
          }
          return { ok: true };
        }

        async function createQueueItem(file) {
          const originalUrl = URL.createObjectURL(file);
          const meta = await readImageMeta(file, originalUrl);
          return {
            inputWidth: meta.width,
            inputHeight: meta.height,
            hasAlpha: meta.hasAlpha,
          };
        }

        async function readImageMeta(file, objectUrl) {
          const source = await decodeSource(file, objectUrl);
          try {
            return {
              width: source.width,
              height: source.height,
              hasAlpha: /png|webp/i.test(file.type || file.name || ""),
            };
          } finally {
            cleanupDecodedSource(source);
          }
        }

        async function decodeSource(file, _objectUrl) {
          try {
            const bitmap = await createImageBitmap(file);
            return { kind: "bitmap", value: bitmap, width: bitmap.width, height: bitmap.height };
          } catch (_error) {}
          return { kind: "image", value: null, width: 5, height: 6 };
        }

        function cleanupDecodedSource(source) {
          if (!source) return;
          if (source.kind === "bitmap" && source.value && typeof source.value.close === "function") {
            source.value.close();
          }
        }

        const input = document.getElementById('file');
        const out = document.getElementById('out');
        input.addEventListener('change', async () => {
          const files = Array.from(input.files || []);
          const file = files[0];
          const validation = validateImageFile(file);
          if (!validation.ok) {
            out.textContent = "invalid";
            return;
          }
          const queuePromise = createQueueItem(file);
          try {
            const item = await queuePromise;
            out.textContent = `ok:${String(queuePromise && typeof queuePromise.then === 'function')}:${item.inputWidth}x${item.inputHeight}:${item.hasAlpha}`;
          } catch (error) {
            out.textContent = `err:${String(queuePromise && typeof queuePromise.then === 'function')}:${String(error && error.message ? error.message : error)}`;
          }
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.jpeg");
    file.size = 1024;
    file.mime_type = "image/jpeg".to_string();
    harness.set_input_files("#file", &[file])?;
    harness.flush()?;
    harness.assert_text("#out", "ok:true:1x1:false")?;
    Ok(())
}

#[test]
fn try_return_value_survives_finally_function_call() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        function cleanup(_value) {
          return;
        }

        async function build() {
          try {
            return { width: 7, height: 8 };
          } finally {
            cleanup(1);
          }
        }

        build().then((meta) => {
          document.getElementById('out').textContent = [
            typeof meta,
            String(meta && meta.width),
            String(meta && meta.height),
          ].join('|');
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "object|7|8")?;
    Ok(())
}

#[test]
fn create_image_bitmap_settles_when_object_url_was_created_earlier_for_same_file() -> Result<()> {
    let html = r#"
      <input id='file' type='file' accept='image/png'>
      <button id='run'>run</button>
      <p id='out'></p>
      <script>
        const out = document.getElementById('out');
        const input = document.getElementById('file');
        let savedFile = null;

        input.addEventListener('change', () => {
          savedFile = input.files[0] || null;
          if (savedFile) {
            URL.createObjectURL(savedFile);
            out.textContent = 'ready';
          }
        });

        document.getElementById('run').addEventListener('click', () => {
          if (!savedFile) {
            out.textContent = 'no-file';
            return;
          }
          let settled = false;
          createImageBitmap(savedFile)
            .then((bmp) => {
              settled = true;
              out.textContent = `ok:${bmp.width}x${bmp.height}`;
            })
            .catch((err) => {
              settled = true;
              out.textContent = `err:${String(err && err.message ? err.message : err)}`;
            });
          setTimeout(() => {
            if (!settled) out.textContent = 'pending';
          }, 0);
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    let mut file = MockFile::new("sample.png");
    file.size = 1024;
    file.mime_type = "image/png".to_string();
    harness.set_input_files("#file", &[file])?;
    harness.click("#run")?;
    harness.flush()?;
    harness.assert_text("#out", "ok:1x1")?;
    Ok(())
}

#[test]
fn issue_127_mock_file_size_matches_array_buffer_byte_length() -> Result<()> {
    let html = r#"
      <input id='f' type='file'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('f');
        const out = document.getElementById('out');
        input.addEventListener('change', async () => {
          const file = input.files[0];
          const buf = await file.arrayBuffer();
          out.textContent = `size:${file.size},bytes:${buf.byteLength}`;
        });
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    let mut file = MockFile::new("manual.csv");
    file.size = 24;
    harness.set_input_files("#f", &[file])?;
    harness.flush()?;
    harness.assert_text("#out", "size:24,bytes:24")?;
    Ok(())
}

#[test]
fn issue_128_class_list_exposes_dom_token_list_methods() -> Result<()> {
    let html = r#"
      <div id='box' class='hidden ready'></div>
      <p id='out'></p>
      <script>
        const box = document.getElementById('box');
        const protoIsObject = typeof Object.getPrototypeOf(box.classList) === 'object';
        const toggled = box.classList.toggle('hidden');
        box.classList.remove('ready');
        box.classList.add('active');
        document.getElementById('out').textContent = [
          String(protoIsObject),
          typeof box.classList.toggle,
          String(toggled),
          String(box.classList.contains('hidden')),
          String(box.classList.contains('ready')),
          String(box.classList.contains('active')),
          String(box.classList.item(0)),
        ].join('|');
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true|function|false|false|false|true|active")?;
    Ok(())
}

#[test]
fn top_level_promise_then_callback_runs_before_timer_flush() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        Promise.resolve().then(() => {
          document.getElementById('out').textContent = 'then-ran';
        });
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "then-ran")?;
    Ok(())
}

#[test]
fn promise_callbacks_share_updated_outer_let_bindings() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        let settled = false;
        Promise.resolve().then(() => {
          settled = true;
        });
        Promise.resolve().then(() => {
          document.getElementById('out').textContent = String(settled);
        });
      </script>
    "#;

    let harness = Harness::from_html(html)?;
    harness.assert_text("#out", "true")?;
    Ok(())
}

#[test]
fn promise_callback_updates_outer_let_before_timeout_callback_reads_it() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        let settled = false;
        Promise.resolve().then(() => {
          settled = true;
        });
        setTimeout(() => {
          document.getElementById('out').textContent = String(settled);
        }, 0);
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "true")?;
    Ok(())
}

#[test]
fn promise_rejection_catch_updates_outer_let_before_timeout_guard() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        let settled = false;
        Promise.reject('boom').catch(() => {
          settled = true;
        });
        setTimeout(() => {
          document.getElementById('out').textContent = settled ? 'handled' : 'pending';
        }, 0);
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "handled")?;
    Ok(())
}

#[test]
fn worker_global_exposes_constructor_aliases_and_static_method_identity() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const source = `
          self.onmessage = () => {
            const result = [
              String(globalThis.String === String),
              String(String['fromCharCode'] === globalThis.String.fromCharCode),
              String(globalThis.Symbol === Symbol),
              String(Symbol['for'] === globalThis.Symbol.for),
              String(globalThis.Int8Array === Int8Array),
              String(Int8Array['of'] === globalThis.Int8Array.of),
              Array.from(new globalThis['Int8Array']([7, 8])).join(','),
              String(globalThis.Number === Number),
              String(globalThis.BigInt === BigInt),
              String(Number['parseInt']('11', 2)),
              String(BigInt['asIntN'](8, 257n))
            ].join('|');
            postMessage(result);
          };
        `;
        const blob = new Blob([source], { type: 'text/javascript' });
        const url = URL.createObjectURL(blob);
        const worker = new Worker(url);
        URL.revokeObjectURL(url);
        worker.onmessage = (event) => {
          document.getElementById('out').textContent = String(event.data || '');
          worker.terminate();
        };
        worker.postMessage('run');
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "true|true|true|true|true|true|7,8|true|true|3|1")?;
    Ok(())
}

#[test]
fn worker_global_exposes_constructor_surface_breadth() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const source = `
          self.onmessage = () => {
            const url = new globalThis['URL']('/worker?x=1', 'https://example.com/base/');
            const map = new globalThis['Map']([['k', 1]]);
            const set = new globalThis['Set'](['v']);
            const params = new globalThis['URLSearchParams']('a=1&b=2');
            const buffer = new globalThis['ArrayBuffer'](4);
            const result = [
              String(globalThis.URL === URL),
              String(globalThis.Map === Map),
              String(globalThis.Set === Set),
              String(globalThis.URLSearchParams === URLSearchParams),
              String(globalThis.ArrayBuffer === ArrayBuffer),
              String(Map.call === Number.call),
              URL.name,
              String(Map.length),
              URLSearchParams.name,
              String(ArrayBuffer.length),
              url.href,
              String(map.get('k')),
              String(set.has('v')),
              params.toString(),
              String(buffer.byteLength)
            ].join('|');
            postMessage(result);
          };
        `;
        const blob = new Blob([source], { type: 'text/javascript' });
        const url = URL.createObjectURL(blob);
        const worker = new Worker(url);
        URL.revokeObjectURL(url);
        worker.onmessage = (event) => {
          document.getElementById('out').textContent = String(event.data || '');
          worker.terminate();
        };
        worker.postMessage('run');
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text(
        "#out",
        "true|true|true|true|true|true|URL|0|URLSearchParams|1|https://example.com/worker?x=1|1|true|a=1&b=2|4",
    )?;
    Ok(())
}

#[test]
fn worker_bound_builtin_constructor_surface_and_instanceof_work() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const source = `
          self.onmessage = () => {
            const BoundMap = Map.bind(null, [['k', 1]]);
            const map = new BoundMap();
            const result = [
              BoundMap.name,
              String(BoundMap.length),
              String(BoundMap.prototype === undefined),
              String(map instanceof BoundMap),
              String(map instanceof Map),
              String(Object.getPrototypeOf(map) === Map.prototype),
              String(map.get('k'))
            ].join('|');
            postMessage(result);
          };
        `;
        const blob = new Blob([source], { type: 'text/javascript' });
        const url = URL.createObjectURL(blob);
        const worker = new Worker(url);
        URL.revokeObjectURL(url);
        worker.onmessage = (event) => {
          document.getElementById('out').textContent = String(event.data || '');
          worker.terminate();
        };
        worker.postMessage('run');
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text("#out", "bound Map|0|true|true|true|true|1")?;
    Ok(())
}

#[test]
fn worker_function_object_prototype_chain_and_metadata_work() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const source = `
          self.onmessage = () => {
            function named(a, b) {
              return a + b;
            }
            const bound = named.bind(null, 1);
            class WorkerBox {
              constructor(x, y = 1) {}
            }
            const fnProto = Object.getPrototypeOf(named);
            postMessage([
              String(Object.getPrototypeOf({}) === Object.prototype),
              String(Object.getPrototypeOf(Object) === fnProto),
              String(Object.getPrototypeOf(Map) === fnProto),
              named.name,
              String(named.length),
              WorkerBox.name,
              String(WorkerBox.length),
              String(Object.getPrototypeOf(bound) === fnProto),
              String(bound instanceof Object),
              bound.constructor.name
            ].join('|'));
          };
        `;
        const blob = new Blob([source], { type: 'text/javascript' });
        const url = URL.createObjectURL(blob);
        const worker = new Worker(url);
        URL.revokeObjectURL(url);
        worker.onmessage = (event) => {
          document.getElementById('out').textContent = String(event.data || '');
          worker.terminate();
        };
        worker.postMessage('run');
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text(
        "#out",
        "true|true|true|named|2|WorkerBox|1|true|true|Function",
    )?;
    Ok(())
}

#[test]
fn worker_global_function_family_constructors_are_exposed_and_callable_work() -> Result<()> {
    let html = r#"
      <p id='out'></p>
      <script>
        const source = `
          self.onmessage = () => {
            const plain = Function('value', 'return value + 1;');
            const genFactory = GeneratorFunction('yield 1;');
            const asyncGenFactory = AsyncGeneratorFunction(
              'yield await Promise.resolve(2);'
            );
            const fnText = Function.toString();
            const genText = GeneratorFunction.toString();
            const asyncGenText = AsyncGeneratorFunction.toString();
            Promise.all([asyncGenFactory().next()]).then((results) => {
              postMessage([
                String(self.Function === Function),
                String(self.GeneratorFunction === GeneratorFunction),
                String(self.AsyncGeneratorFunction === AsyncGeneratorFunction),
                String(Object.getPrototypeOf(Function) === Function.prototype),
                String(Object.getPrototypeOf(GeneratorFunction) === Function.prototype),
                String(Object.getPrototypeOf(AsyncGeneratorFunction) === Function.prototype),
                plain.name,
                String(genFactory().next().value),
                String(results[0].value),
                String(fnText.includes('Function')),
                String(fnText === String(Function)),
                String(genText.includes('GeneratorFunction')),
                String(genText === String(GeneratorFunction)),
                String(asyncGenText.includes('AsyncGeneratorFunction')),
                String(asyncGenText === String(AsyncGeneratorFunction))
              ].join('|'));
            });
          };
        `;
        const blob = new Blob([source], { type: 'text/javascript' });
        const url = URL.createObjectURL(blob);
        const worker = new Worker(url);
        URL.revokeObjectURL(url);
        worker.onmessage = (event) => {
          document.getElementById('out').textContent = String(event.data || '');
          worker.terminate();
        };
        worker.postMessage('run');
      </script>
    "#;

    let mut harness = Harness::from_html(html)?;
    harness.flush()?;
    harness.assert_text(
        "#out",
        "true|true|true|true|true|true|anonymous|1|2|true|true|true|true|true|true",
    )?;
    Ok(())
}
