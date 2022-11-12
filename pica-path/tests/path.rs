use bstr::ByteSlice;
use once_cell::sync::OnceCell;
use pica_path::{Path, PathExt};
use pica_record::ByteRecord;

fn ada_lovelace() -> &'static [u8] {
    use std::path::Path;
    use std::{env, fs};

    static DATA: OnceCell<Vec<u8>> = OnceCell::new();
    DATA.get_or_init(|| {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = Path::new(&manifest_dir)
            .join("../tests/data/119232022.dat");
        eprintln!("{:?}", path);
        fs::read_to_string(&path).unwrap().as_bytes().to_vec()
    })
}

#[test]
fn test_path_simple() -> anyhow::Result<()> {
    let record = ByteRecord::from_bytes(ada_lovelace())?;
    let path = Path::new("003@.0");

    assert_eq!(record.path(&path), vec![&b"119232022".as_bstr()]);

    Ok(())
}

#[test]
fn test_path_codes() -> anyhow::Result<()> {
    let record = ByteRecord::from_bytes(ada_lovelace())?;
    let path = Path::new("047A/03.[er]");

    assert_eq!(
        record.path(&path),
        vec![&b"DE-386".as_bstr(), &b"DE-576".as_bstr()]
    );

    Ok(())
}
