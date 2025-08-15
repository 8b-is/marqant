#[test]
fn test_example_md_roundtrip_all() {
    use std::fs;
    use std::io::Write;
    use std::process::{Command, Stdio};
    let dir = "example-md";
    for entry in fs::read_dir(dir).expect("read_dir example-md") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().map(|e| e == "md").unwrap_or(false) {
            let filename = path.file_name().unwrap().to_string_lossy();
            let markdown = fs::read_to_string(&path).expect("read markdown");
            // Step 1: Encode markdown to mq (using uni-encode)
            let mut encode = Command::new("cargo")
                .args(["run", "--quiet", "--", "uni-encode"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to start mq uni-encode");
            encode
                .stdin
                .as_mut()
                .unwrap()
                .write_all(markdown.as_bytes())
                .unwrap();
            let encoded = encode
                .wait_with_output()
                .expect("Failed to run mq uni-encode")
                .stdout;
            // Step 2: Decode mq back to markdown (using uni-decode)
            let mut decode = Command::new("cargo")
                .args(["run", "--quiet", "--", "uni-decode"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to start mq uni-decode");
            decode.stdin.as_mut().unwrap().write_all(&encoded).unwrap();
            let decoded = decode
                .wait_with_output()
                .expect("Failed to run mq uni-decode")
                .stdout;
            let decoded_str = String::from_utf8_lossy(&decoded);
            // Step 3: Compare original and round-tripped markdown (ignoring trailing whitespace)
            let normalize = |s: &str| s.lines().map(str::trim_end).collect::<Vec<_>>().join("\n");
            let orig = normalize(&markdown);
            let roundtrip = normalize(&decoded_str);
            if orig != roundtrip {
                println!(
                    "\n[{}] MQ ENCODED:\n{}\n\nDECODED OUTPUT:\n{}\n",
                    filename,
                    String::from_utf8_lossy(&encoded),
                    decoded_str
                );
            }
            assert_eq!(
                orig, roundtrip,
                "[{}] Round-trip markdown did not match!\nOriginal:\n{}\n\nRoundtrip:\n{}",
                filename, orig, roundtrip
            );
        }
    }
}
#[test]
fn test_markdown_to_mq_and_back_roundtrip() {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // Sample markdown input
    let markdown = r#"# Title

- a
- b

## Head

Some content
"#;

    // Step 1: Encode markdown to mq (using uni-encode)
    let mut encode = Command::new("cargo")
        .args(["run", "--quiet", "--", "uni-encode"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start mq uni-encode");
    encode
        .stdin
        .as_mut()
        .unwrap()
        .write_all(markdown.as_bytes())
        .unwrap();
    let encoded = encode
        .wait_with_output()
        .expect("Failed to run mq uni-encode")
        .stdout;

    // Step 2: Decode mq back to markdown (using uni-decode)
    let mut decode = Command::new("cargo")
        .args(["run", "--quiet", "--", "uni-decode"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start mq uni-decode");
    decode.stdin.as_mut().unwrap().write_all(&encoded).unwrap();
    let decoded = decode
        .wait_with_output()
        .expect("Failed to run mq uni-decode")
        .stdout;
    let decoded_str = String::from_utf8_lossy(&decoded);

    // Step 3: Compare original and round-tripped markdown (ignoring trailing whitespace)
    let normalize = |s: &str| s.lines().map(str::trim_end).collect::<Vec<_>>().join("\n");
    let orig = normalize(markdown);
    let roundtrip = normalize(&decoded_str);
    assert_eq!(
        orig, roundtrip,
        "Round-trip markdown did not match!\nOriginal:\n{}\n\nRoundtrip:\n{}",
        orig, roundtrip
    );
}
use marqant::read_mq_metadata;
use marqant::Marqant;
use marqant::{mq2_uni_decode, mq2_uni_encode};

#[test]
fn compress_decompress_roundtrip_basic() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let md = "# Title\n\n## Head\n\nSome content\n";
    let mq = Marqant::compress_markdown(md).expect("compress");
    assert!(mq.starts_with("MARQANT"));
    let back = Marqant::decompress_marqant(&mq).expect("decompress");
    assert_eq!(md.trim(), back.trim());
}

#[test]
fn uni_roundtrip_bytes() {
    let data = b"# T\n\n- a\n- b\n";
    let enc = mq2_uni_encode(data).expect("enc");
    let dec = mq2_uni_decode(&enc).expect("dec");
    assert_eq!(data.as_slice(), dec.as_slice());
}

#[test]
fn compress_has_deterministic_header_and_sorted_dict() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    // Deterministic timestamp in tests via now_timestamp() and sorted dict
    let md = "# A\n\n## B\n\n**bold** text with link [x](y)\n";
    let mq = Marqant::compress_markdown(md).expect("compress");
    let lines: Vec<&str> = mq.lines().collect();
    let header_parts: Vec<&str> = lines[0].split_whitespace().collect();
    assert_eq!(header_parts[0], "MARQANT");
    assert_eq!(header_parts[1], "0");
    // Ensure dictionary is before separator and sorted by token key
    let sep = lines.iter().position(|l| *l == "---").expect("sep");
    let dict = &lines[1..sep];
    let mut sorted = dict.to_vec();
    sorted.sort();
    assert_eq!(dict, &sorted[..]);
}

fn assert_snapshot(name: &str, actual: &str) {
    use std::{env, fs, path::Path};
    let p = Path::new("tests/snapshots").join(format!("{name}.snap"));
    if env::var("SNAPSHOT_UPDATE").as_deref() == Ok("1") {
        fs::create_dir_all(p.parent().unwrap()).ok();
        fs::write(&p, actual).unwrap();
    } else {
        let expected = fs::read_to_string(&p).unwrap();
        assert_eq!(expected, actual, "snapshot {name} mismatch");
    }
}

#[test]
fn snapshot_basic_compress() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let md = "# Title\n\n## Head\n\nSome content\n";
    let mq = Marqant::compress_markdown(md).expect("compress");
    assert_snapshot("mq_basic", &mq);
}

#[test]
fn metadata_reader_marqant_v1() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let md = "# Title\n\n## Head\n\nSome content\n";
    let mq = Marqant::compress_markdown(md).expect("compress");
    let info = read_mq_metadata(&mq).expect("meta");
    assert_eq!(info.kind, "MARQANT");
    assert_eq!(info.timestamp.as_deref(), Some("0"));
    assert_eq!(info.original_size, Some(md.len() as u64));
    assert!(info.compressed_size.is_some());
}
