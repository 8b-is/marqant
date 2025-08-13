use marqant::{read_mq_metadata, Marqant};

#[test]
fn read_meta_mq2_parses_numbers_and_fields() {
    let mq = "MQ2~UNI~5F~1A~0F~03~text\n~T\n~~~~\nDATA";
    let info = read_mq_metadata(mq).expect("meta");
    assert_eq!(info.kind, "MQ2");
    assert_eq!(info.variant.as_deref(), Some("UNI"));
    assert_eq!(info.timestamp.as_deref(), Some("5F"));
    assert_eq!(info.original_size, Some(0x1A));
    assert_eq!(info.compressed_size, Some(0x0F));
    assert_eq!(info.token_count, Some(0x03));
    assert_eq!(info.level.as_deref(), Some("text"));
}

#[test]
fn read_meta_mq2_dict_id_from_t_and_s() {
    let mq = "MQ2~UNI~00~00~00~00~text\n~Tabc\n~Sdef\n~~~~\n";
    let info = read_mq_metadata(mq).expect("meta");
    let id = info.dict_id.expect("dict id");
    assert!(id.starts_with("fnv1a64:"));
    assert_eq!(id.len(), "fnv1a64:".len() + 16);
}

#[test]
fn read_meta_marqant_and_unknown() {
    std::env::set_var("MARQANT_TEST_TS", "0");
    let md = "# T\n\n## H\n\nX\n";
    let mq = Marqant::compress_markdown(md).expect("compress");
    let info = read_mq_metadata(&mq).expect("meta");
    assert_eq!(info.kind, "MARQANT");
    assert_eq!(info.timestamp.as_deref(), Some("0"));

    let unk = read_mq_metadata("garbage").expect("meta");
    assert_eq!(unk.kind, "UNKNOWN");
}
