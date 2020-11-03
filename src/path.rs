#[derive(Debug, PartialEq)]
pub struct Path {
    tag: String,
    occurrence: Option<String>,
    code: char,
}

impl Path {
    pub fn new<S>(tag: S, occurrence: Option<S>, code: char) -> Path
    where
        S: Into<String>,
    {
        let occurrence = match occurrence {
            Some(o) => Some(o.into()),
            None => None,
        };

        Path {
            tag: tag.into(),
            occurrence,
            code,
        }
    }

    pub fn tag(&self) -> &String {
        &self.tag
    }

    pub fn occurrence(&self) -> &Option<String> {
        &self.occurrence
    }

    pub fn code(&self) -> char {
        self.code
    }
}
