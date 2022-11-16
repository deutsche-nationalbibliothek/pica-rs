use std::collections::HashMap;

use indicatif::{
    HumanCount, MultiProgress, ProgressBar, ProgressStyle,
};

#[derive(Default)]
struct Stats {
    records: u64,
    errors: u64,
    warnings: u64,
    infos: u64,
    checks: u64,
}

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

    pub fn update_stats(
        &mut self,
        key: &str,
        checks: usize,
        errors: usize,
        warnings: usize,
        infos: usize,
    ) {
        self.stats.entry(key.into()).and_modify(|e| {
            e.checks += checks as u64;
            e.warnings += warnings as u64;
            e.errors += errors as u64;
            e.infos += infos as u64;
            e.records += 1;
        });
    }

    pub fn update(&mut self) {
        let mut checks = 0;
        let mut errors = 0;
        let mut warnings = 0;
        let mut infos = 0;
        self.records += 1;

        for (key, bar) in self.bars.iter() {
            let stats = self.stats.get(key).unwrap();
            checks += stats.checks;
            errors += stats.errors;
            warnings += stats.warnings;
            infos += stats.infos;

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
            "⇒ {} records, {} checks\n⇒ {} errors, {} warnings, {} infos",
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
