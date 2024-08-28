use std::str::FromStr;
use std::sync::OnceLock;

use pica_format::{Format, FormatExt};
use pica_record::ByteRecord;

fn ada_lovelace() -> &'static [u8] {
    use std::path::Path;
    use std::{env, fs};

    static DATA: OnceLock<Vec<u8>> = OnceLock::new();
    DATA.get_or_init(|| {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&manifest_dir)
            .join("../pica-toolkit/tests/data/119232022.dat");
        fs::read_to_string(&path).unwrap().as_bytes().to_vec()
    })
}

#[test]
fn test_format() -> anyhow::Result<()> {
    let ada = ByteRecord::from_bytes(ada_lovelace()).expect("record");
    let fmt = Format::from_str("028A{ a <$> (', ' d <*> ' ' c) }")?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["Lovelace, Ada King of".to_string()]);

    Ok(())
}

#[test]
fn test_format_predicate() -> anyhow::Result<()> {
    let ada = ByteRecord::from_bytes(ada_lovelace()).expect("record");
    let fmt = Format::from_str(
        "028[A@]{ a <$> (', ' d <*> ' ' c) | 4 == 'nafr'}",
    )?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["Byron, Ada Augusta".to_string()]);

    Ok(())
}

#[test]
fn test_format_quantifier() -> anyhow::Result<()> {
    let ada = ByteRecord::from_bytes(ada_lovelace()).expect("record");
    let fmt = Format::from_str(
        "042A{ 'https://d-nb.info/standards/vocab/gnd/gnd-sc#' a }",
    )?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(
        result,
        vec!["https://d-nb.info/standards/vocab/gnd/gnd-sc#28p"
            .to_string()]
    );

    let fmt = Format::from_str("042A{ 'GND-SC: ' a.. ' ' }")?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["GND-SC: 28p GND-SC: 9.5p ".to_string()]);

    let fmt =
        Format::from_str("007[KN]{ ( a '-' <*> 0 <*> '-' v)..2  }")?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["".to_string()]);

    Ok(())
}
