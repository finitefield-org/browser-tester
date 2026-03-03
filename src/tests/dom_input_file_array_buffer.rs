use super::*;

#[test]
fn input_file_mock_exposes_array_buffer_with_mock_bytes() -> Result<()> {
    let html = r#"
      <input id='upload' type='file'>
      <p id='out'></p>
      <script>
        const input = document.getElementById('upload');
        input.addEventListener('change', async () => {
          const file = input.files[0];
          const buffer = await file.arrayBuffer();
          const bytes = Array.from(new Uint8Array(buffer)).join(',');
          document.getElementById('out').textContent =
            file.name + ':' + file.size + ':' + bytes;
        });
      </script>
    "#;

    let mut h = Harness::from_html(html)?;
    let file = MockFile::new("payload.bin").with_bytes(&[1, 2, 3, 250]);
    h.set_input_files("#upload", &[file])?;
    h.assert_text("#out", "payload.bin:4:1,2,3,250")?;
    Ok(())
}
