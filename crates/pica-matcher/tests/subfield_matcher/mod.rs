use bstr::B;
use pica_matcher::subfield_matcher::*;
use pica_matcher::{MatcherOptions, ParseMatcherError};
use pica_record::SubfieldRef;

macro_rules! subfield {
    ($code:expr, $value:expr) => {
        SubfieldRef::new($code, $value)
    };
}

#[test]
fn exists_matcher_new() {
    let subfield = subfield!('0', "119232022");
    let options = MatcherOptions::default();

    let matcher = ExistsMatcher::new(vec!['0']);
    assert!(matcher.is_match(&subfield, &options));

    let matcher = ExistsMatcher::new(vec!['2', '3']);
    assert!(!matcher.is_match(&subfield, &options));
}

#[test]
fn exists_matcher_try_from() {
    let subfield = subfield!('0', "119232022");
    let options = MatcherOptions::default();

    let matcher = ExistsMatcher::try_from("0?".as_bytes()).unwrap();
    assert!(matcher.is_match(&subfield, &options));

    assert!(matches!(
        ExistsMatcher::try_from("ä?".as_bytes()).unwrap_err(),
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));
}

#[test]
fn exists_matcher_is_match() -> anyhow::Result<()> {
    let matcher = ExistsMatcher::try_from(B("1?"))?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('1', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));

    assert!(matcher.is_match(
        [&subfield!('3', "def"), &subfield!('1', "hij"),],
        &options
    ));

    let matcher = ExistsMatcher::try_from(B("[a12]?"))?;
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('1', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('9', "abc"), &options));
    assert!(matcher.is_match(
        [
            &subfield!('3', "def"),
            &subfield!('9', "hij"),
            &subfield!('2', "bsg"),
        ],
        &options
    ));

    Ok(())
}

#[test]
fn relational_matcher_new() {
    let matcher = RelationMatcher::new("0 == 'abc'");
    assert!(matcher
        .is_match(&subfield!('0', "abc"), &MatcherOptions::new()));
}

#[test]
#[should_panic]
fn relational_matcher_new_panic() {
    let _matcher = RelationMatcher::new("! == 'abc'");
}

#[test]
fn relational_matcher_is_matcher_equal() {
    // case sensitive
    let matcher = RelationMatcher::new("0 == 'abc'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "ABC"), &options));
    assert!(!matcher.is_match(&subfield!('1', "abc"), &options));
    assert!(matcher.is_match(
        [
            &subfield!('3', "def"),
            &subfield!('0', "abc"),
            &subfield!('2', "bsg"),
        ],
        &options
    ));

    // case insensitive
    let matcher = RelationMatcher::new("0 == 'abc'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "ABC"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 == 'abc'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [
            &subfield!('3', "def"),
            &subfield!('0', "abc"),
            &subfield!('2', "bsg"),
        ],
        &options
    ));

    assert!(!matcher.is_match(
        [&subfield!('3', "def"), &subfield!('2', "bsg")],
        &options
    ));
}

#[test]
fn relational_matcher_not_equal() {
    // case sensitive
    let matcher = RelationMatcher::new("0 != 'abc'");
    let options = MatcherOptions::default();

    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "ABC"), &options));
    assert!(!matcher.is_match(&subfield!('1', "abc"), &options));
    assert!(!matcher.is_match(
        [&subfield!('0', "abc"), &subfield!('2', "bsg"),],
        &options
    ));

    // case insensitive
    let matcher = RelationMatcher::new("0 != 'abc'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(!matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "ABC"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 != 'abc'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [&subfield!('3', "def"), &subfield!('0', "bsg"),],
        &options
    ));

    assert!(!matcher.is_match(
        [
            &subfield!('3', "def"),
            &subfield!('0', "abc"),
            &subfield!('2', "bsg"),
        ],
        &options
    ));
}

#[test]
fn relational_matcher_starts_with() {
    // case sensitive
    let matcher = RelationMatcher::new("0 =^ 'ab'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "def"), &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 =^ 'ab'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('0', "abc"), &options));
    assert!(matcher.is_match(&subfield!('0', "aBc"), &options));
    assert!(!matcher.is_match(&subfield!('0', "def"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 =^ 'ab'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [&subfield!('0', "baab"), &subfield!('0', "abba"),],
        &options
    ));

    assert!(!matcher.is_match(
        [&subfield!('0', "def"), &subfield!('1', "abc"),],
        &options
    ));
}

#[test]
fn relational_matcher_ends_with() {
    // case sensitive
    let matcher = RelationMatcher::new("0 =$ 'ab'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abab"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abba"), &options));

    // case insensitive
    let matcher = RelationMatcher::new("0 =$ 'ab'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('0', "abab"), &options));
    assert!(matcher.is_match(&subfield!('0', "abab"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("0 =$ 'ab'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        [&subfield!('0', "baba"), &subfield!('0', "abab")],
        &options
    ));

    assert!(!matcher.is_match(
        [&subfield!('0', "def"), &subfield!('1', "aab")],
        &options
    ));
}

#[test]
fn relational_matcher_similar() {
    // default threshold
    let matcher = RelationMatcher::new("a =* 'Heike'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "Heike"), &options));
    assert!(!matcher.is_match(&subfield!('a', "Heiko"), &options));

    // threshold set
    let matcher = RelationMatcher::new("a =* 'Heike'");
    let options = MatcherOptions::new().strsim_threshold(0.7);

    assert!(matcher.is_match(&subfield!('a', "Heike"), &options));
    assert!(matcher.is_match(&subfield!('a', "Heiko"), &options));

    // default threshold
    let matcher = RelationMatcher::new("a =* 'Heike'");
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('a', "Heike"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("a =* 'Heike'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![&subfield!('a', "Heiko"), &subfield!('a', "Heike")],
        &options
    ));
}

#[test]
fn relational_matcher_contains() {
    // default options
    let matcher = RelationMatcher::new("a =? 'aba'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('a', "aba"), &options));
    assert!(matcher.is_match(&subfield!('a', "xabax"), &options));
    assert!(!matcher.is_match(&subfield!('a', "abba"), &options));

    // case ignore
    let matcher = RelationMatcher::new("a =? 'AbA'");
    let options = MatcherOptions::default().case_ignore(true);

    assert!(matcher.is_match(&subfield!('a', "aba"), &options));
    assert!(matcher.is_match(&subfield!('a', "xabax"), &options));
    assert!(!matcher.is_match(&subfield!('a', "abba"), &options));

    // multiple subfields
    let matcher = RelationMatcher::new("a =? 'aba'");
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![&subfield!('a', "XabbaX"), &subfield!('a', "YabaY")],
        &options
    ));
}

#[test]
fn regex_matcher_new() {
    let _matcher = RegexMatcher::new(vec!['0'], "^T[gpsu][1z]$", false);
}

#[test]
#[should_panic]
fn regex_matcher_new_panic1() {
    RegexMatcher::new(vec!['0'], "^T[[gpsu][1z]$", false);
}

#[test]
fn regex_matcher_try_from() {
    assert!(RegexMatcher::try_from(B("0 =~ '^T[gpsu][1z]$'")).is_ok());

    let error =
        RegexMatcher::try_from(B("0 =~ '^Tp[[1z]$'")).unwrap_err();
    assert!(matches!(
        error,
        ParseMatcherError::InvalidSubfieldMatcher(_)
    ));
}

#[test]
fn regex_matcher_is_match() {
    // case sensitive
    let matcher = RegexMatcher::new(vec!['0'], "^ab", false);
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "abba"), &options));
    assert!(!matcher.is_match(&subfield!('0', "bba"), &options));
    assert!(!matcher.is_match(&subfield!('a', "abba"), &options));

    // case insensitive
    let matcher = RegexMatcher::new(vec!['0'], "^ab", false);
    let options = MatcherOptions::new().case_ignore(true);

    assert!(matcher.is_match(&subfield!('0', "abba"), &options));
    assert!(matcher.is_match(&subfield!('0', "abba"), &options));

    // invert match
    let matcher = RegexMatcher::new(vec!['0'], "^ab", true);
    let options = MatcherOptions::default();

    assert!(matcher.is_match(&subfield!('0', "baba"), &options));
    assert!(!matcher.is_match(&subfield!('0', "abba"), &options));

    // multiple subfields
    let matcher = RegexMatcher::new(vec!['0'], "^ab", false);
    let options = MatcherOptions::default();

    assert!(matcher.is_match(
        vec![&subfield!('0', "foobar"), &subfield!('0', "abba")],
        &options
    ));

    assert!(!matcher.is_match(
        vec![&subfield!('0', "foo"), &subfield!('0', "bar")],
        &options
    ));
}

// #[test]
// fn in_matcher() -> anyhow::Result<()> {
//     // case sensitive
//     let matcher = InMatcher::new("0 in ['abc', 'def']");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1f0abc")?, &options));
//     assert!(matcher
//         .is_match(&subfield!('0', "def")?, &options));
//     assert!(!matcher
//         .is_match(&subfield!(b"\x1f0hij")?, &options));
//     assert!(!matcher
//         .is_match(&subfield!('0', "def")?, &options));

//     // case insensitive
//     let matcher = InMatcher::new("0 in ['abc', 'def']");
//     let options = MatcherOptions::new().case_ignore(true);

//     assert!(matcher
//         .is_match(&subfield!(b"\x1f0abc")?, &options));
//     assert!(matcher
//         .is_match(&subfield!(b"\x1f0ABC")?, &options));

//     // multiple subfields
//     let matcher = InMatcher::new("0 in ['abc', 'def']");
//     let options = MatcherOptions::default();

//     assert!(matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0hij")?,
//             &subfield!(b"\x1f0abc")?,
//         ],
//         &options
//     ));

//     let matcher = InMatcher::new("a in ['000', '999']");
//     let options = MatcherOptions::default();

//     assert!(matcher.is_match(
//         vec![
//             &subfield!(b"\x1fa000")?,
//             &subfield!(b"\x1fzxyz")?,
//         ],
//         &options
//     ));

//     let matcher = InMatcher::new("a not in ['000', '999']");
//     let options = MatcherOptions::default();

//     assert!(!matcher.is_match(
//         vec![
//             &subfield!(b"\x1fa000")?,
//             &subfield!(b"\x1fzxyz")?,
//         ],
//         &options
//     ));

//     Ok(())
// }

// #[test]
// fn cardinality_matcher_eq() -> anyhow::Result<()> {
//     let matcher = CardinalityMatcher::new("#0 == 2");
//     let options = MatcherOptions::default();

//     assert!(!matcher
//         .is_match(&subfield!(b"\x1fXabc"), &options));
//     assert!(!matcher
//         .is_match(&subfield!(b"\x1f0abc"), &options));
//     assert!(matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//         ],
//         &options
//     ));
//     assert!(!matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//             &subfield!(b"\x1f0hij")?,
//         ],
//         &options
//     ));

//     Ok(())
// }

// #[test]
// fn cardinality_matcher_ne() -> anyhow::Result<()> {
//     let matcher = CardinalityMatcher::new("#0 != 2");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1fXabc"), &options));
//     assert!(matcher
//         .is_match(&subfield!(b"\x1f0abc"), &options));
//     assert!(!matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//         ],
//         &options
//     ));
//     assert!(matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//             &subfield!(b"\x1f0hij")?,
//         ],
//         &options
//     ));

//     Ok(())
// }

// #[test]
// fn cardinality_matcher_ge() -> anyhow::Result<()> {
//     let matcher = CardinalityMatcher::new("#0 >= 2");
//     let options = MatcherOptions::default();

//     assert!(!matcher
//         .is_match(&subfield!(b"\x1fXabc"), &options));
//     assert!(!matcher
//         .is_match(&subfield!(b"\x1f0abc"), &options));
//     assert!(matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//         ],
//         &options
//     ));
//     assert!(matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//             &subfield!(b"\x1f0hij")?,
//         ],
//         &options
//     ));

//     Ok(())
// }

// #[test]
// fn cardinality_matcher_gt() -> anyhow::Result<()> {
//     let matcher = CardinalityMatcher::new("#0 > 2");
//     let options = MatcherOptions::default();

//     assert!(!matcher
//         .is_match(&subfield!(b"\x1fXabc"), &options));
//     assert!(!matcher
//         .is_match(&subfield!(b"\x1f0abc"), &options));
//     assert!(!matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//         ],
//         &options
//     ));
//     assert!(matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//             &subfield!(b"\x1f0hij")?,
//         ],
//         &options
//     ));

//     Ok(())
// }

// #[test]
// fn cardinality_matcher_le() -> anyhow::Result<()> {
//     let matcher = CardinalityMatcher::new("#0 <= 2");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1fXabc"), &options));
//     assert!(matcher
//         .is_match(&subfield!(b"\x1f0abc"), &options));
//     assert!(matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//         ],
//         &options
//     ));
//     assert!(!matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//             &subfield!(b"\x1f0hij")?,
//         ],
//         &options
//     ));

//     Ok(())
// }

// #[test]
// fn cardinality_matcher_lt() -> anyhow::Result<()> {
//     let matcher = CardinalityMatcher::new("#0 < 2");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1fXabc"), &options));
//     assert!(matcher
//         .is_match(&subfield!(b"\x1f0abc"), &options));
//     assert!(!matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//         ],
//         &options
//     ));
//     assert!(!matcher.is_match(
//         vec![
//             &subfield!(b"\x1f0abc")?,
//             &subfield!('0', "def")?,
//             &subfield!(b"\x1f0hij")?,
//         ],
//         &options
//     ));
//     Ok(())
// }

// #[test]
// fn subfield_matcher_not() -> anyhow::Result<()> {
//     // group
//     let matcher = SubfieldMatcher::new("!(a == 'bcd')");
//     let options = MatcherOptions::default();

//     assert!(!matcher
//         .is_match(&subfield!(b"\x1fabcd")?, &options));
//     assert!(matcher
//         .is_match(&subfield!(b"\x1fbcde")?, &options));

//     // exists
//     let matcher = SubfieldMatcher::new("!a?");
//     let options = MatcherOptions::default();

//     assert!(!matcher
//         .is_match(&subfield!(b"\x1fabcd")?, &options));
//     assert!(matcher
//         .is_match(&subfield!(b"\x1fbcde")?, &options));

//     // not
//     let matcher = SubfieldMatcher::new("!!!(a == 'bcd')");
//     let options = MatcherOptions::default();

//     assert!(!matcher
//         .is_match(&subfield!(b"\x1fabcd")?, &options));
//     assert!(matcher
//         .is_match(&subfield!(b"\x1fbcde")?, &options));
// }

// #[test]
// fn subfield_matcher_group() -> anyhow::Result<()> {
//     // and
//     let matcher = SubfieldMatcher::new("(a =^ 'ab' && a =$ 'ba')");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1faabba")?,
// &options));     assert!(!matcher
//         .is_match(&subfield!(b"\x1fbcde")?, &options));

//     // or
//     let matcher = SubfieldMatcher::new("(a =^ 'ab' || a =^ 'ba')");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1faabba")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1fababa")?,
// &options));

//     // singleton
//     let matcher = SubfieldMatcher::new("(a == 'bcd')");
//     let options = MatcherOptions::default();
//     assert!(matcher
//         .is_match(&subfield!(b"\x1fabcd")?, &options));
//     assert!(!matcher
//         .is_match(&subfield!(b"\x1fbcde")?, &options));

//     // nested group
//     let matcher = SubfieldMatcher::new("(((a == 'bcd')))");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1fabcd")?, &options));

//     // not
//     let matcher = SubfieldMatcher::new("(!(a == 'bcd'))");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1fhijk")?, &options));

//     Ok(())
// }

// #[test]
// fn subfield_matcher_or() -> anyhow::Result<()> {
//     // singleton
//     let matcher =
//         SubfieldMatcher::new("a =^ 'ab' || a =^ 'bc' || a =^ 'cd'");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1faabab")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1fabcbc")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1facdcd")?,
// &options));     assert!(!matcher
//         .is_match(&subfield!(b"\x1fadede")?,
// &options));     assert!(!matcher
//         .is_match(&subfield!(b"\x1fbabab")?,
// &options));

//     // group
//     let matcher =
//         SubfieldMatcher::new("a =^ 'ab' || (a =^ 'bc' && a =$
// 'cd')");     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1faabab")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1fabccd")?,
// &options));     assert!(!matcher
//         .is_match(&subfield!(b"\x1fabcbc")?,
// &options));

//     // and
//     let matcher =
//         SubfieldMatcher::new("a =^ 'ab' || a =^ 'bc' && a =$ 'cd'");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1faabab")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1faabcd")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1fabccd")?,
// &options));     assert!(!matcher
//         .is_match(&subfield!(b"\x1fabcbc")?,
// &options));

//     // or
//     let matcher = SubfieldMatcher::new("!a? || b == 'x'");
//     let options = MatcherOptions::default();

//     assert!(!matcher
//         .is_match(&subfield!(b"\x1faabab")?,
// &options));

//     assert!(matcher.is_match(
//         vec![
//             &subfield!(b"\x1fabccd")?,
//             &subfield!(b"\x1fbx")?
//         ],
//         &options
//     ));

//     // not
//     let matcher = SubfieldMatcher::new("a == 'bcd' || !(a !=
// 'def')");     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1fabcd")?, &options));
//     assert!(matcher
//         .is_match(&subfield!(b"\x1fadef")?, &options));
//     assert!(!matcher
//         .is_match(&subfield!(b"\x1fahij")?, &options));

//     // boolean op precedence
//     let matcher =
//         SubfieldMatcher::new("(a =^ 'ab' || a =^ 'bc') && a =$
// 'cd'");     let options = MatcherOptions::default();

//     assert!(!matcher
//         .is_match(&subfield!(b"\x1faabab")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1faabcd")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1fabccd")?,
// &options));     assert!(!matcher
//         .is_match(&subfield!(b"\x1fabcbc")?,
// &options));

//     // multiple subfields
//     let matcher = SubfieldMatcher::new("#a == 2 || a =^ 'ab'");
//     let options = MatcherOptions::default();

//     assert!(matcher.is_match(
//         [
//             &subfield!(b"\x1fadef")?,
//             &subfield!(b"\x1fahij")?,
//         ],
//         &options
//     ));

//     assert!(matcher.is_match(
//         [
//             &subfield!(b"\x1fadef")?,
//             &subfield!(b"\x1fahij")?,
//             &subfield!(b"\x1faabc")?,
//         ],
//         &options
//     ));

//     Ok(())
// }

// #[test]
// fn subfield_matcher_and() -> anyhow::Result<()> {
//     // singleton
//     let matcher =
//         SubfieldMatcher::new("#a == 1 && a =^ 'ab' && a =$ 'ba'");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1faabba")?,
// &options));     assert!(!matcher
//         .is_match(&subfield!(b"\x1fababa")?,
// &options));     assert!(!matcher.is_match(
//         [
//             &subfield!(b"\x1faabba")?,
//             &subfield!(b"\x1fababa")?,
//         ],
//         &options
//     ));

//     // group
//     let matcher =
//         SubfieldMatcher::new("#a == 1 && (a =^ 'ab' || a =^ 'ba')");
//     let options = MatcherOptions::default();

//     assert!(matcher
//         .is_match(&subfield!(b"\x1faabba")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1fababa")?,
// &options));     assert!(!matcher.is_match(
//         [
//             &subfield!(b"\x1faabba")?,
//             &subfield!(b"\x1fababa")?,
//         ],
//         &options
//     ));

//     // not
//     let matcher =
//         SubfieldMatcher::new("#a == 1 && !(a =^ 'ab' || a =^ 'ba')");
//     let options = MatcherOptions::default();

//     assert!(!matcher
//         .is_match(&subfield!(b"\x1faabba")?,
// &options));     assert!(!matcher
//         .is_match(&subfield!(b"\x1fababa")?,
// &options));     assert!(matcher
//         .is_match(&subfield!(b"\x1facbcb")?,
// &options));

//     Ok(())
// }
