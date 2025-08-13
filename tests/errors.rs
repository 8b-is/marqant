use marqant::Marqant;

#[test]
fn decompress_invalid_headers_error() {
    assert!(Marqant::decompress_marqant("not marqant").is_err());
    assert!(Marqant::decompress_marqant("MARQANT_V1 0 0\n").is_err());
}
