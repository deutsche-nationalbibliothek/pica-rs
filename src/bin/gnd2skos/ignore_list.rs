use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug)]
pub(crate) struct IgnoreList {
    inner: HashSet<(String, String)>,
}

impl Default for IgnoreList {
    fn default() -> Self {
        IgnoreList {
            inner: HashSet::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Record {
    idn: String,
    label: String,
}

impl IgnoreList {
    pub(crate) fn from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<Self, std::io::Error> {
        let mut list = HashSet::new();
        let mut reader = csv::Reader::from_path(path)?;
        for result in reader.deserialize() {
            let record: Record = result?;
            list.insert((record.label, record.idn));
        }

        Ok(IgnoreList { inner: list })
    }

    pub(crate) fn contains(&self, label: String, idn: String) -> bool {
        self.inner.contains(&(label, idn))
    }
}
