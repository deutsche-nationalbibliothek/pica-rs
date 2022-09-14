use bstr::BString;
use quickcheck::{Arbitrary, Gen};

use crate::{Subfield, Tag};

impl Arbitrary for Subfield {
    fn arbitrary(g: &mut Gen) -> Self {
        let code = (1..)
            .map(|_| char::arbitrary(g))
            .find(|c| c.is_ascii_alphanumeric())
            .unwrap();

        let value = String::arbitrary(g);

        Subfield(code, value.into())
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let values = self.1.shrink().map(BString::from);
        let code = self.0;

        Box::new(values.map(move |value| Subfield(code, value)))
    }
}

impl Arbitrary for Tag {
    fn arbitrary(g: &mut Gen) -> Self {
        let p1 = *g.choose(b"012").unwrap();
        let p2 = *g.choose(b"0123456789").unwrap();
        let p3 = *g.choose(b"0123456789").unwrap();
        let p4 = *g.choose(b"ABCDEFGHIJKLMNOPQRSTUVWXYZ@").unwrap();

        Tag::from_bytes(&[p1, p2, p3, p4]).unwrap()
    }
}
