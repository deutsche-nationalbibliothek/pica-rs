use std::str::FromStr;
use std::sync::OnceLock;

use pica_format::{Format, FormatExt, FormatOptions};
use pica_record::ByteRecord;

type TestResult = anyhow::Result<()>;

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
fn test_format_from_str() -> TestResult {
    let ada = ByteRecord::from_bytes(ada_lovelace()).expect("record");
    let fmt = Format::from_str("028A{ a <$> (', ' d <*> ' ' c) }")?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["Lovelace, Ada King of".to_string()]);

    Ok(())
}

#[test]
fn test_format_predicate() -> TestResult {
    let ada = ByteRecord::from_bytes(ada_lovelace()).expect("record");
    let fmt =
        Format::new("028[A@]{ a <$> (', ' d <*> ' ' c) | 4 == 'nafr'}");
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["Byron, Ada Augusta".to_string()]);

    Ok(())
}

#[test]
fn test_format_strip_overread_char() -> TestResult {
    let fmt =
        Format::new("029[A@]{ a <$> (' (' g ')' <*> ' / ' [xb])}");

    let data = b"029A \x1faThe @Hitmakers\x1e\n";
    let record = ByteRecord::from_bytes(data).expect("record");
    let options = FormatOptions::new().strip_overread_char(false);
    assert_eq!(record.format(&fmt, &options), vec!["The @Hitmakers"],);

    Ok(())
}

#[test]
fn test_format_quantifier() -> TestResult {
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
    assert_eq!(result, vec!["GND-SC: 28p GND-SC: 9.5p "]);

    let fmt = Format::from_str("042A{ ( a )..1  }")?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["28p"]);

    Ok(())
}

#[test]
fn test_format_modifier_trim() -> TestResult {
    let ada = ByteRecord::from_bytes(ada_lovelace()).expect("record");
    let fmt = Format::from_str("042A{ (?T 'GND-SC: ' a.. ' ') }")?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["GND-SC: 28p GND-SC: 9.5p"]);

    Ok(())
}

#[test]
fn test_format_modifier_uppercase() -> TestResult {
    let ada = ByteRecord::from_bytes(ada_lovelace()).expect("record");
    let fmt = Format::from_str("028A{ a <$> (?U ', ' d <*> ' ' c) }")?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["Lovelace, ADA KING OF".to_string()]);

    Ok(())
}

#[test]
fn test_format_modifier_lowercase() -> TestResult {
    let ada = ByteRecord::from_bytes(ada_lovelace()).expect("record");
    let fmt = Format::from_str("028A{ a <$> (?L ', ' d <*> ' ' c) }")?;
    let result = ada.format(&fmt, &Default::default());
    assert_eq!(result, vec!["Lovelace, ada king of".to_string()]);

    Ok(())
}

#[test]
fn test_format_conference() -> TestResult {
    let fmt = Format::new(
        "030[A@]{(n ' ') <*> a <$> (', ' d <*> ' (' [cg] ')')}",
    );

    let data = b"030A \x1faInternationale Hofer Filmtage\
               \x1fn13.\x1fd1979\x1fcHof (Saale)\x1e\n";
    let record = ByteRecord::from_bytes(data).expect("record");
    assert_eq!(
        record.format(&fmt, &Default::default()),
        vec!["13. Internationale Hofer Filmtage, 1979 (Hof (Saale))"]
    );

    let data = b"030@ \x1faNockherberg\x1fgVeranstaltung\x1e\n";
    let record = ByteRecord::from_bytes(data).expect("record");
    assert_eq!(
        record.format(&fmt, &Default::default()),
        vec!["Nockherberg (Veranstaltung)"],
    );

    Ok(())
}

#[test]
fn test_format_coroporate_body() -> TestResult {
    let fmt = Format::new(
        "029[A@]{ a <$> (' (' g ')' <*> ' / ' [xb] <*> ', ' n)}",
    );
    let options = Default::default();

    let data = b"029A \x1faThe @Hitmakers\x1e\n";
    let record = ByteRecord::from_bytes(data).expect("record");
    assert_eq!(record.format(&fmt, &options), vec!["The Hitmakers"],);

    let data = "029A \x1faDeutschland\x1fgBundesrepublik\
                \x1fbAuswärtiges Amt\x1fbBibliothek\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Deutschland (Bundesrepublik) / Auswärtiges Amt / Bibliothek"],
    );

    let data =
        "029@ \x1faUSA\x1fbArmy\x1fbInfantry Division\x1fn27\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["USA / Army / Infantry Division, 27"],
    );

    Ok(())
}

#[test]
fn test_format_person() -> TestResult {
    let fmt = Format::new(
        "028A{ [aP] <$> (', ' d <*> ' ' [nc] <*> ' (' l ')')}",
    );
    let options = Default::default();

    let data = "028A \x1fPFriedrich\x1fnII.\x1flPreußen, König\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Friedrich II. (Preußen, König)"],
    );

    let data = "028A \x1fdJohann Wolfgang\x1fcvon\x1faGoethe\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Goethe, Johann Wolfgang von"],
    );

    Ok(())
}

#[test]
fn test_format_geographic_name() -> TestResult {
    let fmt = Format::new("065A{ a <$> (' (' [gz] ')' <*> ' / ' x)}");
    let options = Default::default();

    let data = "065A \x1faArgolis\x1fzNord\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(record.format(&fmt, &options), vec!["Argolis (Nord)"]);

    let data = "065A \x1faUSA\x1fxSüdstaaten\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(record.format(&fmt, &options), vec!["USA / Südstaaten"]);

    let data = "065A \x1faSanta Maria Maggiore\
                \x1fgRom\x1fxKrippenkapelle\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Santa Maria Maggiore (Rom) / Krippenkapelle"]
    );

    Ok(())
}

#[test]
fn test_format_subject_heading() -> TestResult {
    let fmt = Format::new("041[A@]{ a <$> (' (' g ')' <*> ' / ' x)}");
    let options = Default::default();

    let data = "041A \x1faPlymouth\x1fgMarke\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(record.format(&fmt, &options), vec!["Plymouth (Marke)"]);

    let data = "041A \x1faBerlin\x1fg1945\x1fxEroberung\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Berlin (1945) / Eroberung"]
    );

    let data = "041A \x1faDas @Gute\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(record.format(&fmt, &options), vec!["Das Gute"]);

    let data =
        "041A \x1faBarletta\x1fxDisfida di Barletta\x1fgMotiv\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Barletta / Disfida di Barletta (Motiv)"]
    );

    let data = "041@ \x1faAbwasser\x1fxBeseitigung\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Abwasser / Beseitigung"]
    );

    Ok(())
}

#[test]
fn test_format_work() -> TestResult {
    let fmt = Format::new(
        "022A{ a <$> (', ' [nfh] <*> '. ' p <*> ' (' g ')' )}",
    );
    let options = Default::default();

    let data = "022A \x1faVerfassung\x1ff2011\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(record.format(&fmt, &options), vec!["Verfassung, 2011"]);

    let data = "022A \x1faFaust\x1fn1\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(record.format(&fmt, &options), vec!["Faust, 1"]);

    let data = "022A \x1faFaust\x1fgVolksbuch\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Faust (Volksbuch)"]
    );

    let data = "022A \x1faBibel\x1fpPetrusbrief\x1fn1.-2.\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Bibel. Petrusbrief, 1.-2."]
    );

    let data = "022A \x1faOtello\x1fgFilm\x1ff1986\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec!["Otello (Film), 1986"],
    );

    let data = "022A \x1faDer @gute Gott von Manhattan\
                \x1fhgesprochenes Wort\x1ff1958\x1fgWestphal\x1e\n";
    let record = ByteRecord::from_bytes(data.as_bytes())?;
    assert_eq!(
        record.format(&fmt, &options),
        vec![
            "Der gute Gott von Manhattan, gesprochenes Wort, \
        1958 (Westphal)"
        ],
    );

    Ok(())
}
