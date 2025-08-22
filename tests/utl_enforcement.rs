//! Test that UTL enforcement actually works

use marqant::utl_enforced::*;

#[test]
fn test_raw_to_english_works() {
    // This SHOULD work - goes through UTL
    let result = to_english("I love you").unwrap();
    assert!(result.contains("I"));
    assert!(result.contains("love"));
    assert!(result.contains("you"));
}

#[test]
fn test_raw_to_japanese_works() {
    // This SHOULD work - goes through UTL
    let result = to_japanese("I think").unwrap();
    assert!(result.contains("私"));
    assert!(result.contains("考える"));
}

#[test]
fn test_utl_required() {
    // Create raw text
    let raw = RawText("Hello world".to_string());
    
    // Can convert to UTL
    let utl = RawToUtl.translate(raw).unwrap();
    assert!(!utl.tokens.is_empty());
    
    // Can convert UTL to English
    let eng = UtlToHuman::<Eng>::new().translate(utl.clone()).unwrap();
    assert!(!eng.text.is_empty());
    
    // Can convert UTL to Japanese  
    let jpn = UtlToHuman::<Jpn>::new().translate(utl).unwrap();
    assert!(!jpn.text.is_empty());
}

// THIS SECTION DEMONSTRATES WHAT CANNOT BE DONE
// Uncomment any of these to see compile errors!

/*
#[test]
fn test_direct_human_to_human_fails() {
    // THIS WILL NOT COMPILE - no way to go English → Japanese directly
    let eng = HumanText::<Eng> { 
        _lang: std::marker::PhantomData, 
        text: "Hello".to_string() 
    };
    
    // ERROR: No implementation exists for this!
    let jpn: HumanText<Jpn> = some_magic_translator(eng); // ← COMPILE ERROR!
}
*/

/*
#[test]
fn test_cannot_skip_utl() {
    // THIS WILL NOT COMPILE - cannot go Raw → Human directly
    let raw = RawText("Hello".to_string());
    
    // ERROR: No Translate<RawText, HumanText<Eng>> implementation!
    let eng: HumanText<Eng> = DirectTranslator.translate(raw); // ← COMPILE ERROR!
}
*/

/*
#[test]
fn test_cannot_reverse_pipeline() {
    // THIS WILL NOT COMPILE - cannot go Human → Raw
    let eng = HumanText::<Eng> {
        _lang: std::marker::PhantomData,
        text: "Hello".to_string()
    };
    
    // ERROR: No Translate<HumanText<Eng>, RawText> implementation!
    let raw: RawText = ReverseTranslator.translate(eng); // ← COMPILE ERROR!
}
*/

#[test]
fn test_forbid_function_always_errors() {
    // The honeypot function always returns an error
    let result = forbid_human_to_human::<Eng, Jpn>();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("FORBIDDEN"));
}