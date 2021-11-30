#[derive(Debug)]
pub struct MatcherFlags {
    pub ignore_case: bool,
    pub strsim_threshold: f64,
}

impl Default for MatcherFlags {
    fn default() -> Self {
        Self {
            ignore_case: false,
            strsim_threshold: 0.8,
        }
    }
}

impl MatcherFlags {
    pub fn new() -> Self {
        MatcherFlags::default()
    }

    pub fn ignore_case(mut self, yes: bool) -> Self {
        self.ignore_case = yes;
        self
    }

    pub fn strsim_threshold(mut self, threshold: f64) -> Self {
        self.strsim_threshold = threshold;
        self
    }
}
