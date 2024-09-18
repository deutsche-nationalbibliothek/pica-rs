/// Options and flags which can be used to configure a matcher.
#[derive(Debug)]
pub struct MatcherOptions {
    /// The threshold for string similarity comparisons.
    pub(crate) strsim_threshold: f64,
    /// Whether to ignore case when comparing values or not.
    pub(crate) case_ignore: bool,
}

impl Default for MatcherOptions {
    /// Creates [MatcherOptions] with default settings.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::MatcherOptions;
    ///
    /// let _options = MatcherOptions::default();
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[inline]
    fn default() -> Self {
        Self {
            strsim_threshold: 0.8,
            case_ignore: false,
        }
    }
}

impl MatcherOptions {
    /// Create new [MatcherOptions].
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::MatcherOptions;
    ///
    /// let _options = MatcherOptions::new();
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether to ignore case when comparing strings or not.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::MatcherOptions;
    ///
    /// let _options = MatcherOptions::new().case_ignore(true);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn case_ignore(mut self, yes: bool) -> Self {
        self.case_ignore = yes;
        self
    }

    /// Set the similarity threshold for the similar operator (`=*`).
    ///
    /// # Example
    ///
    /// ```rust
    /// use pica_record::matcher::MatcherOptions;
    ///
    /// let _options = MatcherOptions::new().strsim_threshold(0.75);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn strsim_threshold(mut self, threshold: f64) -> Self {
        self.strsim_threshold = threshold;
        self
    }
}
