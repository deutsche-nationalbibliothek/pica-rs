use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Write};

use clap::Parser;
use pica::{Reader, ReaderBuilder};
use serde::{Deserialize, Serialize};
use xml::writer::XmlEvent;
use xml::EmitterConfig;

use crate::config::Config;
use crate::skip_invalid_flag;
use crate::translit::translit_maybe;
use crate::util::CliResult;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct XmlConfig {
    pub(crate) skip_invalid: Option<bool>,
    pub(crate) gzip: Option<bool>,
}

#[derive(Parser, Debug)]
pub(crate) struct Xml {
    /// Skip invalid records that can't be decoded
    #[arg(long, short)]
    skip_invalid: bool,

    /// Transliterate output into the selected normalform <NF>
    /// (possible values: "nfd", "nfkd", "nfc" and "nfkc")
    #[arg(long,
          value_name = "NF",
          value_parser = ["nfd", "nfkd", "nfc", "nfkc"],
          hide_possible_values = true,
    )]
    translit: Option<String>,

    /// Write output to <filename> instead of stdout
    #[arg(short, long, value_name = "filename")]
    output: Option<OsString>,

    /// Read one or more files in normalized PICA+ format.
    #[arg(default_value = "-", hide_default_value = true)]
    filenames: Vec<OsString>,
}

impl Xml {
    pub(crate) fn run(self, config: &Config) -> CliResult<()> {
        let skip_invalid = skip_invalid_flag!(
            self.skip_invalid,
            config.xml,
            config.global
        );

        let mut writer: Box<dyn Write> = match self.output {
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
                .attr(
                    "targetNamespace",
                    "info:srw/schema/5/picaXML-v1.0",
                )
                .default_ns("info:srw/schema/5/picaXML-v1.0"),
        )?;

        for filename in self.filenames {
            let builder =
                ReaderBuilder::new().skip_invalid(skip_invalid);
            let mut reader: Reader<Box<dyn Read>> = match filename
                .to_str()
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
                            XmlEvent::start_element("subfield").attr(
                                "code",
                                &subfield.code().to_string(),
                            ),
                        )?;

                        let value = translit_maybe(
                            &subfield.value().to_string(),
                            self.translit.as_deref(),
                        );
                        xml_writer
                            .write(XmlEvent::characters(&value))?;
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
}
