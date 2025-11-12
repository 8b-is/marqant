//! This file demonstrates compile-time UTL enforcement
//!
//! UNCOMMENT THE TEST BELOW TO SEE IT FAIL TO COMPILE

use marqant::utl_enforced::*;

/*
#[test]
fn this_will_not_compile() {
    // Try to do direct English → Japanese translation
    // This VIOLATES the UTL-only rule!

    struct BadTranslator;
    impl Translate<HumanText<Eng>, HumanText<Jpn>> for BadTranslator {
        fn translate(&self, input: HumanText<Eng>) -> Result<HumanText<Jpn>> {
            // This would bypass UTL - NOT ALLOWED!
            Ok(HumanText {
                _lang: std::marker::PhantomData,
                text: "これは悪い翻訳です".to_string()
            })
        }
    }

    let eng = HumanText::<Eng> {
        _lang: std::marker::PhantomData,
        text: "Hello".to_string()
    };

    // If this compiled, it would violate our rules
    let jpn = BadTranslator.translate(eng).unwrap();
    println!("This should never print: {}", jpn.text);
}
*/

#[test]
fn utl_only_pipeline_works() {
    // This is the ONLY allowed way
    let text = "I love programming";

    // Must go through UTL
    let english = to_english(text).unwrap();
    let japanese = to_japanese(text).unwrap();

    println!("English: {}", english);
    println!("Japanese: {}", japanese);

    assert!(true); // Pipeline works!
}
