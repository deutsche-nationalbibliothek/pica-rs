use std::collections::HashMap;

use indicatif::{
    HumanCount, MultiProgress, ProgressBar, ProgressStyle,
};

use crate::stats::Stats;

pub struct Progress {
    stats: HashMap<String, Stats>,
    bars: HashMap<String, ProgressBar>,
    summary: ProgressBar,
    footer: ProgressBar,
    records: u64,
}

impl Progress {
    pub fn new(ids: Vec<String>) -> Self {
        let mut bars: HashMap<String, ProgressBar> = HashMap::new();
        let mut stats: HashMap<String, Stats> = HashMap::new();
        let root = MultiProgress::new();

        for id in ids.into_iter() {
            if !stats.contains_key(&id) {
                stats.insert(id.to_string(), Stats::default());
                let bar = root.add(ProgressBar::new_spinner());
                bar.set_style(
                    ProgressStyle::with_template("↪ {msg}").unwrap(),
                );
                bars.insert(id.to_string(), bar);
            }
        }

        let summary = root.add(ProgressBar::new_spinner());
        summary
            .set_style(ProgressStyle::with_template("{msg}").unwrap());

        let footer = root.add(ProgressBar::new_spinner());
        footer.set_style(
            ProgressStyle::with_template("⏱ {elapsed_precise}")
                .unwrap(),
        );

        Self {
            summary,
            footer,
            stats,
            bars,
            records: 0,
        }
    }

    pub fn update_stats(&mut self, name: &str, stats: &Stats) {
        self.stats.entry(name.into()).and_modify(|e| {
            e.records += stats.records;
            e.checks += stats.checks;
            e.errors += stats.errors;
            e.warnings += stats.warnings;
            e.infos += stats.infos;
        });
    }

    pub fn update(&mut self) {
        let mut errors = 0;
        let mut warnings = 0;
        let mut infos = 0;
        let mut checks = 0;

        self.records += 1;

        for (key, bar) in self.bars.iter() {
            let stats = self.stats.get(key).unwrap();
            errors += stats.errors;
            warnings += stats.warnings;
            infos += stats.infos;
            checks += stats.checks;

            bar.set_message(format!(
                "{}: {} records, {} errors, {} warnings, {} infos",
                key,
                HumanCount(stats.records),
                HumanCount(stats.errors),
                HumanCount(stats.warnings),
                HumanCount(stats.infos),
            ));
        }

        self.summary.inc(1);
        self.summary.set_message(format!(
            "⇒ {} records, {} checks, {} errors, {} warnings, {} infos",
            HumanCount(self.records),
            HumanCount(checks),
            HumanCount(errors),
            HumanCount(warnings),
            HumanCount(infos),
        ));

        self.footer.inc(1);
    }

    pub fn finish(&self) {
        for (_, bar) in self.bars.iter() {
            bar.finish();
        }

        self.summary.finish();
        self.footer.finish();
    }
}
