use parse::parse_format;
use pica_record::{FieldRef, SubfieldRef};
use thiserror::Error;
use winnow::Parser;

mod parse;

#[derive(Debug, Clone, PartialEq)]
pub struct Format(Vec<Fragment>);

impl Format {
    pub fn fragments(&self) -> impl Iterator<Item = &Fragment> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fragment {
    Atom(Atom),
    Group(Group),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Atom {
    codes: Vec<char>,
    prefix: Option<String>,
    suffix: Option<String>,
}

impl Atom {
    fn format_subfield(
        &self,
        buf: &mut String,
        subfield: &SubfieldRef,
        options: &FormatOptions,
    ) {
        if !self.codes.contains(&subfield.code()) {
            return;
        }

        let mut value = subfield.value().to_string();
        if options.strip_overread_char {
            value = value.replacen('@', "", 1);
        }

        if !value.is_empty() {
            if let Some(ref prefix) = self.prefix {
                buf.push_str(prefix);
            }

            buf.push_str(&value);

            if let Some(ref suffix) = self.suffix {
                buf.push_str(suffix);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    atoms: Vec<Atom>,
}

impl Fragment {
    fn format(
        &self,
        buf: &mut String,
        field: &FieldRef,
        options: &FormatOptions,
    ) {
        match self {
            Self::Atom(atom) => {
                if let Some(subfield) = atom
                    .codes
                    .iter()
                    .find_map(|code| field.find(|s| s.code() == *code))
                {
                    atom.format_subfield(buf, subfield, options);
                }
            }
            Self::Group(group) => {
                field.subfields().iter().for_each(|subfield| {
                    group.atoms.iter().for_each(|atom| {
                        atom.format_subfield(buf, subfield, options);
                    });
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FormatOptions {
    strip_overread_char: bool,
}

impl FormatOptions {
    pub fn new(strip_overread_char: bool) -> Self {
        Self {
            strip_overread_char,
        }
    }
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            strip_overread_char: true,
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{0} is not a valid format string")]
pub struct ParseFormatError(String);

impl Format {
    /// Creates a new format from a string slice.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_format::Format;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let _fmt = Format::new("a")?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new<T>(fmt: T) -> Result<Self, ParseFormatError>
    where
        T: AsRef<str>,
    {
        parse_format
            .parse(fmt.as_ref())
            .map_err(|_| ParseFormatError(fmt.as_ref().into()))
    }
}

pub trait FormatExt {
    fn format(&self, fmt: &Format, options: &FormatOptions) -> String;
}

impl FormatExt for FieldRef<'_> {
    /// Formats a field reference according to the format string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_format::{Format, FormatExt};
    /// use pica_record::FieldRef;
    ///
    /// # fn main() { example().unwrap(); }
    /// fn example() -> anyhow::Result<()> {
    ///     let field =
    ///         FieldRef::from_bytes(b"041A \x1faGoethe\x1e").unwrap();
    ///     let format = Format::new("a")?;
    ///     let options = Default::default();
    ///     assert_eq!(field.format(&format, &options), "Goethe");
    ///     Ok(())
    /// }
    /// ```
    fn format(&self, fmt: &Format, options: &FormatOptions) -> String {
        let mut buf = String::new();
        fmt.fragments().for_each(|fragment| {
            fragment.format(&mut buf, self, options);
        });

        buf
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type TestResult = anyhow::Result<()>;

    #[test]
    fn test_format_subject_headings() -> TestResult {
        let opts = FormatOptions::default();
        let fmt = Format::new("a (' / ' x <|> ' (' g ')')")?;

        let data = "041A \x1faPlymouth\x1fgMarke\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(field.format(&fmt, &opts), "Plymouth (Marke)");

        let data = "041A \x1faSchlacht um Berlin\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(field.format(&fmt, &opts), "Schlacht um Berlin");

        let data = "041A \x1faDas @Gute\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(field.format(&fmt, &opts), "Das Gute");

        let data =
            "041A \x1faBarletta\x1fxDisfida di Barletta\x1fgMotiv\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(
            field.format(&fmt, &opts),
            "Barletta / Disfida di Barletta (Motiv)"
        );

        Ok(())
    }

    #[test]
    fn test_format_geographic_names() -> TestResult {
        let opts = FormatOptions::default();
        let fmt = Format::new("a (' (' [gz] ')' <|> ' / ' x)")?;

        let data = "065A \x1faArgolis\x1fzNord\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(field.format(&fmt, &opts), "Argolis (Nord)");

        let data = "065A \x1faUSA\x1fxSüdstaaten\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(field.format(&fmt, &opts), "USA / Südstaaten");

        let data = "065A \x1faSanta Maria Maggiore\x1fgRom\
            \x1fxKrippenkapelle\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(
            field.format(&fmt, &opts),
            "Santa Maria Maggiore (Rom) / Krippenkapelle"
        );

        Ok(())
    }

    #[test]
    fn test_format_corporate_bodies() -> TestResult {
        let opts = FormatOptions::default();
        let fmt =
            Format::new("a (' (' g ')' <|> ' / ' [xb] <|> ', ' n)")?;

        let data = "029A \x1faThe @Hitmakers\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(field.format(&fmt, &opts), "The Hitmakers");

        let data = "029A \x1faDeutschland\x1fgBundesrepublik\
                    \x1fbAuswärtiges Amt\x1fbBibliothek\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(
            field.format(&fmt, &opts),
            "Deutschland (Bundesrepublik) / Auswärtiges Amt / Bibliothek"
        );

        let data = "029A \x1faTōkai Daigaku\x1fbKōgakubu\x1fn2\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(
            field.format(&fmt, &opts),
            "Tōkai Daigaku / Kōgakubu, 2"
        );

        Ok(())
    }

    #[test]
    fn test_format_conferences() -> TestResult {
        let opts = FormatOptions::default();
        let fmt = Format::new(
            "(n ' ') a (', ' d <|> ' (' c ')' <|> ' / ' [bx])",
        )?;

        let data = "030A \x1faInternationale Hofer Filmtage\
                    \x1fn13.\x1fd1979\x1fcHof (Saale)\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(
            field.format(&fmt, &opts),
            "13. Internationale Hofer Filmtage, 1979 (Hof (Saale))"
        );

        let data = "030A \x1faOECD\x1fb\
                    Ministerial Meeting on Science of OECD Countries\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(
            field.format(&fmt, &opts),
            "OECD / Ministerial Meeting on Science of OECD Countries"
        );

        Ok(())
    }

    #[test]
    fn test_format_works() -> TestResult {
        let opts = FormatOptions::default();
        let fmt =
            Format::new("a (' (' [fg] ')' <|> ', ' n <|> '. ' p)")?;

        let data = "022A \x1faVerfassung\x1ff2011\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(field.format(&fmt, &opts), "Verfassung (2011)");

        let data = "022A \x1faFaust\x1fn1\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(field.format(&fmt, &opts), "Faust, 1");

        let data = "022A \x1faFaust\x1fgVolksbuch\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(field.format(&fmt, &opts), "Faust (Volksbuch)");

        let data = "022A \x1faBibel\x1fpPetrusbrief\x1fn1.-2.\x1e";
        let field = FieldRef::from_bytes(data.as_bytes())?;
        assert_eq!(
            field.format(&fmt, &opts),
            "Bibel. Petrusbrief, 1.-2."
        );

        Ok(())
    }
}
