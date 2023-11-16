// use std::path::Path;
// use std::sync::OnceLock;
// use std::{env, fs};

// use pica_matcher::RecordMatcher;
// use pica_record::RecordMut;

// fn ada_lovelace() -> &'static [u8] {
//     static DATA: OnceLock<Vec<u8>> = OnceLock::new();
//     DATA.get_or_init(|| {
//         let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
//         let path = Path::new(&manifest_dir)
//             .join("../pica-toolkit/tests/data/119232022.dat");
//         eprintln!("{:?}", path);
//         fs::read_to_string(&path).unwrap().as_bytes().to_vec()
//     })
// }

// #[test]
// fn record_matcher_exists() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;

//     let matcher = RecordMatcher::new("004B?")?;
//     assert!(matcher.is_match(&record, &Default::default()));

//     let matcher = RecordMatcher::new("028A.a?")?;
//     assert!(matcher.is_match(&record, &Default::default()));

//     Ok(())
// }

// #[test]
// fn record_matcher_cardinality() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher = RecordMatcher::new(
//         "#028[A@]{d =^ 'Ada' && a == 'Lovelace'} == 5",
//     )?;

//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }

// #[test]
// fn record_matcher_in() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher = RecordMatcher::new("002@.0 in ['Tpz', 'Tp1']")?;
//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }

// #[test]
// fn record_matcher_regex() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher = RecordMatcher::new("047A/03.[er] =~
// '^DE-\\\\d+6'")?;     assert!(matcher.is_match(&record,
// &Default::default()));     Ok(())
// }

// #[test]
// fn record_matcher_eq() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher = RecordMatcher::new("003@.0 == '119232022'")?;
//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }

// #[test]
// fn record_matcher_ne() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher = RecordMatcher::new("002@.0 != 'Ts1'")?;
//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }

// #[test]
// fn record_matcher_starts_with() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher =
//         RecordMatcher::new("003U.a =^ 'http://d-nb.info/gnd/'")?;
//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }

// #[test]
// fn record_matcher_ends_with() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher = RecordMatcher::new("042B.a =$ '-GB'")?;
//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }

// #[test]
// fn record_matcher_group() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher =
//         RecordMatcher::new("(002@.0 == 'Tp1' && 004B.a == 'pik')")?;
//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }

// #[test]
// fn record_matcher_not() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher =
//         RecordMatcher::new("!(002@.0 == 'Ts1' || 002@.0 =^ 'Tu')")?;
//     assert!(matcher.is_match(&record, &Default::default()));

//     let matcher = RecordMatcher::new("!012A.0?")?;
//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }

// #[test]
// fn record_matcher_and() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher =
//         RecordMatcher::new("002@.0 == 'Tp1' && 004B.a == 'pik'")?;
//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }

// #[test]
// fn record_matcher_or() -> anyhow::Result<()> {
//     let record = RecordMut::from_bytes(ada_lovelace())?;
//     let matcher =
//         RecordMatcher::new("002@.0 == 'Ts1' || 004B.a == 'pik'")?;
//     assert!(matcher.is_match(&record, &Default::default()));
//     Ok(())
// }
