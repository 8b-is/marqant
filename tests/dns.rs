use marqant::dns::resolve_dns_dict;
use std::collections::HashMap;

fn mock_dig(exit_code: i32, stdout: &str) {
    let script = format!("#!/bin/sh\necho '{}'\nexit {}", stdout, exit_code);
    let path = "target/debug/mock_dig";
    std::fs::write(path, script).unwrap();
    std::process::Command::new("chmod")
        .arg("+x")
        .arg(path)
        .status()
        .unwrap();
    std::env::set_var("MQ_DIG_CMD", path);
}

#[test]
fn resolve_dns_success() {
    let b64_key1 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "\x01");
    let b64_val1 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, "# ");
    let mock_stdout = format!("\"{}={}\"", b64_key1, b64_val1);
    mock_dig(0, &mock_stdout);
    let dict = resolve_dns_dict("test-ok").unwrap().unwrap();
    let mut expected = HashMap::new();
    expected.insert("\x01".to_string(), "# ".to_string());
    assert_eq!(dict, expected);
}

#[test]
fn resolve_dns_nxdomain() {
    mock_dig(1, ""); // dig returns non-zero for NXDOMAIN
    let dict = resolve_dns_dict("test-nx").unwrap();
    assert!(dict.is_none());
}

#[test]
fn resolve_dns_invalid_pair() {
    mock_dig(0, "\"invalid-pair\"");
    assert!(resolve_dns_dict("test-invalid").is_err());
}
