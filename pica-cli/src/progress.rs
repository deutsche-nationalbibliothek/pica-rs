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
                "{spinner} {msg}, elapsed: {elapsed_precise}",
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
    pub(crate) fn update(&mut self, invalid: bool) {
        self.bar.inc(1);

        if invalid {
            self.invalid += 1;
        } else {
            self.records += 1;
        }

        let per_sec = self.bar.per_sec();

        self.bar.set_message(format!(
            "{} records, {} invalid | {} records/s",
            HumanCount(self.records),
            HumanCount(self.invalid),
            HumanCount(per_sec.round() as u64),
        ));
    }

    #[inline(always)]
    pub(crate) fn finish(&self) {
        self.bar.finish();
    }
}
