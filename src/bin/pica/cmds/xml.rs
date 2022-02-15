use std::fs::File;
use std::io::{self, Write};

use clap::Arg;
use pica::ReaderBuilder;
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;
use xml::EmitterConfig;

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::{App, CliArgs, CliResult};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct XmlConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

pub(crate) fn cli() -> App {
    App::new("xml")
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
                .help("Comma-separated list of column names."),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("--output")
                .value_name("file")
                .help("Write output to <file> instead of stdout."),
        )
        .arg(Arg::new("filename"))
}

pub(crate) fn run(args: &CliArgs, config: &Config) -> CliResult<()> {
    let skip_invalid = skip_invalid_flag!(args, config.xml, config.global);

    let mut reader = ReaderBuilder::new()
        .skip_invalid(skip_invalid)
        .from_path_or_stdin(args.value_of("filename"))?;

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
                        .attr("occurrence", &occurrence.to_string()),
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

    xml_writer.write(XmlEvent::end_element())?;
    writer.flush()?;

    Ok(())
}
