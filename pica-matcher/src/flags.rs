#[derive(Debug)]
pub struct MatcherFlags {
    pub case_ignore: bool,
    pub strsim_threshold: f64,
}

impl Default for MatcherFlags {
    fn default() -> Self {
        Self {
            case_ignore: false,
            strsim_threshold: 0.8,
        }
    }
}

impl MatcherFlags {
    /// Create new matcher flags.
    pub fn new() -> Self {
        MatcherFlags::default()
    }

    /// Whether to ignore case when comparing strings or not.
    pub fn case_ignore(mut self, yes: bool) -> Self {
        self.case_ignore = yes;
        self
    }

    /// Set the similarity threshold for the similar operator (`=*`).
    pub fn strsim_threshold(mut self, threshold: f64) -> Self {
        self.strsim_threshold = threshold;
        self
    }
}
