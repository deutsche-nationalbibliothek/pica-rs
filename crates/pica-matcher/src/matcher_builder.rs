use pica_utils::NormalizationForm;

use crate::{ParseMatcherError, RecordMatcher};

pub struct MatcherBuilder {
    matcher: RecordMatcher,
    nf: Option<NormalizationForm>,
}

type Result<T> = std::result::Result<T, ParseMatcherError>;

impl MatcherBuilder {
    pub fn new(
        matcher: String,
        nf: Option<NormalizationForm>,
    ) -> Result<Self> {
        let matcher = RecordMatcher::try_from(
            NormalizationForm::translit_opt(matcher, nf).as_bytes(),
        )?;

        Ok(Self { matcher, nf })
    }

    pub fn and(mut self, matcher: Vec<String>) -> Result<Self> {
        for predicate in matcher.iter() {
            self.matcher &= RecordMatcher::try_from(
                NormalizationForm::translit_opt(predicate, self.nf)
                    .as_bytes(),
            )?;
        }

        Ok(self)
    }

    pub fn or(mut self, matcher: Vec<String>) -> Result<Self> {
        for predicate in matcher.iter() {
            self.matcher |= RecordMatcher::try_from(
                NormalizationForm::translit_opt(predicate, self.nf)
                    .as_bytes(),
            )?;
        }

        Ok(self)
    }

    pub fn not(mut self, matcher: Vec<String>) -> Result<Self> {
        for predicate in matcher.iter() {
            self.matcher &= !RecordMatcher::try_from(
                NormalizationForm::translit_opt(predicate, self.nf)
                    .as_bytes(),
            )?;
        }

        Ok(self)
    }

    pub fn build(self) -> RecordMatcher {
        self.matcher
    }
}
