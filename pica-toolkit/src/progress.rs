use indicatif::{HumanCount, ProgressBar, ProgressStyle};

pub(crate) struct Progress {
    bar: ProgressBar,
    records: u64,
    invalid: u64,
}

impl Progress {
    pub(crate) fn new(enable: bool) -> Self {
        let bar = if enable {
            ProgressBar::new_spinner()
        } else {
            ProgressBar::hidden()
        };

        bar.set_style(
            ProgressStyle::with_template(
                "{spinner} {msg}, {elapsed_precise}",
            )
            .unwrap(),
        );

        Self {
            bar,
            records: 0,
            invalid: 0,
        }
    }

    #[inline]
    pub(crate) fn record(&mut self) {
        self.records += 1;
        self.update();
    }

    #[inline]
    pub(crate) fn invalid(&mut self) {
        self.invalid += 1;
        self.update();
    }

    pub(crate) fn update(&mut self) {
        self.bar.inc(1);
        let per_sec = self.bar.per_sec();

        self.bar.set_message(format!(
            "records: {} invalid: {} | {} records/s",
            HumanCount(self.records),
            HumanCount(self.invalid),
            per_sec.round() as i64,
        ));
    }

    #[inline]
    pub(crate) fn finish(&self) {
        self.bar.finish();
    }
}
