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
