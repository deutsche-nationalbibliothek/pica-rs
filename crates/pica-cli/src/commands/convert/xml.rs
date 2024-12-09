use std::ffi::OsString;
use std::fs::File;
use std::io::{self, stdout, BufWriter, Write};

use bstr::ByteSlice;
use pica_record::prelude::*;
use quick_xml::events::{
    BytesDecl, BytesEnd, BytesStart, BytesText, Event,
};
use quick_xml::writer::Writer;

pub(crate) struct XmlWriter {
    writer: Writer<BufWriter<Box<dyn Write>>>,
}

impl XmlWriter {
    pub(crate) fn new(output: Option<OsString>) -> io::Result<Self> {
        let inner: BufWriter<Box<dyn Write>> =
            if let Some(filename) = output {
                BufWriter::new(Box::new(File::create(filename)?))
            } else {
                BufWriter::new(Box::new(stdout()))
            };

        let mut writer = Writer::new_with_indent(inner, b' ', 4);
        writer
            .write_event(Event::Decl(BytesDecl::new(
                "1.0",
                Some("UTF-8"),
                None,
            )))
            .unwrap();

        let attributes = [
            ("targetNamespace", "info:srw/schema/5/picaXML-v1.0"),
            ("xmlns:xs", "http://www.w3.org/2001/XMLSchema"),
            ("xmlns", "info:srw/schema/5/picaXML-v1.0"),
        ];

        writer
            .write_event(Event::Start(
                BytesStart::from_content("collection", 0)
                    .with_attributes(attributes),
            ))
            .unwrap();

        Ok(Self { writer })
    }
}

impl ByteRecordWrite for XmlWriter {
    fn write_byte_record(
        &mut self,
        record: &ByteRecord,
    ) -> std::io::Result<()> {
        self.writer
            .create_element("record")
            .write_inner_content(|r| {
                for field in record.fields() {
                    r.create_element("datafield")
                        .with_attribute((
                            "tag",
                            field.tag().to_string().as_str(),
                        ))
                        .write_inner_content(|f| {
                            for subfield in field.subfields() {
                                f.create_element("subfield")
                                    .with_attribute((
                                        "code",
                                        subfield
                                            .code()
                                            .to_string()
                                            .as_str(),
                                    ))
                                    .write_text_content(BytesText::new(
                                        subfield
                                            .value()
                                            .to_str()
                                            .unwrap(),
                                    ))
                                    .unwrap();
                            }

                            Ok::<(), std::io::Error>(())
                        })
                        .unwrap();
                }

                Ok::<(), std::io::Error>(())
            })
            .unwrap();

        Ok(())
    }

    fn finish(&mut self) -> io::Result<()> {
        self.writer
            .write_event(Event::End(BytesEnd::new("collection")))
            .unwrap();
        Ok(())
    }
}
