use bstr::BString;
use quickcheck::{Arbitrary, Gen};

use crate::Subfield;

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
