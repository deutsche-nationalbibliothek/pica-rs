use unicode_normalization::UnicodeNormalization;

pub(crate) fn translit_maybe(value: &str, translit: Option<&str>) -> String {
    match translit {
        Some("nfc") => value.nfc().collect::<String>(),
        Some("nfkc") => value.nfkc().collect::<String>(),
        Some("nfd") => value.nfd().collect::<String>(),
        Some("nfkd") => value.nfkd().collect::<String>(),
        None => value.to_string(),
        _ => panic!("Unknown unicode normal form"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translit_maybe() {
        let expected = vec![
            // Input = NFC
            ("Am\u{0e9}lie", None, "Am\u{0e9}lie"), // no-translit
            ("Am\u{0e9}lie", Some("nfc"), "Am\u{0e9}lie"), // NFC -> NFC
            ("Am\u{0e9}lie", Some("nfkc"), "Am\u{0e9}lie"), // NFC -> NFKC
            ("Am\u{0e9}lie", Some("nfd"), "Ame\u{301}lie"), // NFC -> NFD
            ("Am\u{0e9}lie", Some("nfkd"), "Ame\u{301}lie"), // NFC -> NFD
            // Input = NFD
            ("Ame\u{301}lie", None, "Ame\u{301}lie"), // no-translit
            ("Ame\u{301}lie", Some("nfd"), "Ame\u{301}lie"), // NFD -> NFD
            ("Ame\u{301}lie", Some("nfkd"), "Ame\u{301}lie"), // NFD -> NFD
            ("Ame\u{301}lie", Some("nfc"), "Am\u{0e9}lie"), // NFD -> NFC
            ("Ame\u{301}lie", Some("nfkc"), "Am\u{0e9}lie"), // NFD -> NFC
        ];

        for (input, translit, output) in expected {
            assert_eq!(translit_maybe(input, translit), output);
        }
    }

    #[test]
    #[should_panic(expected = "Unknown unicode normal form")]
    fn test_translit_maybe_panic() {
        translit_maybe("foo", Some("foo"));
    }
}
