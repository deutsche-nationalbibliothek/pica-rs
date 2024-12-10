use crate::matcher::{ParseMatcherError, RecordMatcher};

pub struct RecordMatcherBuilder<S> {
    transform: Box<dyn Fn(S) -> String>,
    matcher: RecordMatcher,
}

impl<S: AsRef<str>> RecordMatcherBuilder<S> {
    pub fn new(matcher: S) -> Result<Self, ParseMatcherError> {
        let matcher = RecordMatcher::new(matcher.as_ref())?;
        let transform = Box::new(|s: S| s.as_ref().to_string());
        Ok(Self { matcher, transform })
    }

    pub fn with_transform<F>(
        matcher: S,
        transform: F,
    ) -> Result<Self, ParseMatcherError>
    where
        F: Fn(S) -> String + 'static,
    {
        let transform = Box::new(transform);
        let matcher = RecordMatcher::new(transform(matcher))?;

        Ok(Self { matcher, transform })
    }

    pub fn and(
        mut self,
        predicates: Vec<S>,
    ) -> Result<Self, ParseMatcherError> {
        for predicate in predicates {
            self.matcher &=
                RecordMatcher::new((self.transform)(predicate))?;
        }

        Ok(self)
    }

    pub fn or(
        mut self,
        predicates: Vec<S>,
    ) -> Result<Self, ParseMatcherError> {
        for predicate in predicates {
            self.matcher |=
                RecordMatcher::new((self.transform)(predicate))?;
        }

        Ok(self)
    }

    pub fn not(
        mut self,
        predicates: Vec<S>,
    ) -> Result<Self, ParseMatcherError> {
        for predicate in predicates {
            self.matcher &=
                !RecordMatcher::new((self.transform)(predicate))?;
        }

        Ok(self)
    }

    pub fn build(self) -> RecordMatcher {
        self.matcher
    }
}
