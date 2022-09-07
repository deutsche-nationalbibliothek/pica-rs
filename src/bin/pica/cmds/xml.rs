use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};

use clap::Arg;
use pica::{Reader, ReaderBuilder};
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;
use xml::EmitterConfig;

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::{CliArgs, CliResult, Command};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct XmlConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

pub(crate) fn cli() -> Command {
    Command::new("xml")
        .about("Serialize records to PICA XML")
        .arg(
            Arg::new("skip-invalid")
                .short('s')
                .long("skip-invalid")
                .help("skip invalid records"),
        )
        .arg(
            Arg::new("translit")
                .long("--translit")
                .value_name("translit")
                .possible_values(["nfd", "nfkd", "nfc", "nfkc"])
                .help("If present, transliterate output into the selected normalform.")
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(
            Arg::new("filenames")
                .help(
                    "Read one or more files in normalized PICA+ format. If the file \
                    ends with .gz the content is automatically decompressed. With no \
                    <filenames>, or when filename is -, read from standard input (stdin).")
                .value_name("filenames")
                .multiple_values(true)
        )
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid =
        skip_invalid_flag!(args, config.xml, config.global);

    let mut writer: Box<dyn Write> = match args.value_of("output") {
        Some(filename) => Box::new(File::create(filename)?),
        None => Box::new(io::stdout()),
    };

    let mut xml_writer = EmitterConfig::new()
        .perform_indent(true)
        .normalize_empty_elements(true)
        .pad_self_closing(true)
        .create_writer(&mut writer);

    xml_writer.write(
        XmlEvent::start_element("collection")
            .ns("xs", "http://www.w3.org/2001/XMLSchema")
            .attr("targetNamespace", "info:srw/schema/5/picaXML-v1.0")
            .default_ns("info:srw/schema/5/picaXML-v1.0"),
    )?;

    let filenames = args
        .values_of_t::<OsString>("filenames")
        .unwrap_or_else(|_| vec![OsString::from("-")]);

    for filename in filenames {
        let builder = ReaderBuilder::new().skip_invalid(skip_invalid);
        let mut reader: Reader<Box<dyn Read>> = match filename.to_str()
        {
            Some("-") => builder.from_reader(Box::new(io::stdin())),
            _ => builder.from_path(filename)?,
        };

        for result in reader.records() {
            let record = result?;

            xml_writer.write(
                XmlEvent::start_element("record")
                    .default_ns("info:srw/schema/5/picaXML-v1.0"),
            )?;

            for field in record.iter() {
                if let Some(occurrence) = field.occurrence() {
                    xml_writer.write(
                        XmlEvent::start_element("datafield")
                            .attr("tag", &field.tag().to_string())
                            .attr(
                                "occurrence",
                                &occurrence.to_string(),
                            ),
                    )?;
                } else {
                    xml_writer.write(
                        XmlEvent::start_element("datafield")
                            .attr("tag", &field.tag().to_string()),
                    )?;
                }

                for subfield in field.iter() {
                    xml_writer.write(
                        XmlEvent::start_element("subfield")
                            .attr("code", &subfield.code().to_string()),
                    )?;

                    let value = translit_maybe(
                        &subfield.value().to_string(),
                        args.value_of("translit"),
                    );
                    xml_writer.write(XmlEvent::characters(&value))?;
                    xml_writer.write(XmlEvent::end_element())?;
                }

                xml_writer.write(XmlEvent::end_element())?;
            }

            xml_writer.write(XmlEvent::end_element())?;
        }
    }

    xml_writer.write(XmlEvent::end_element())?;
    writer.flush()?;

    Ok(())
}
