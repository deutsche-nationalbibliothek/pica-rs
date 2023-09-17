use std::ops::AddAssign;

#[derive(Debug, Default)]
pub struct Stats {
    pub records: u64,
    pub checks: u64,
    pub errors: u64,
    pub warnings: u64,
    pub infos: u64,
}

impl Stats {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AddAssign for Stats {
    fn add_assign(&mut self, rhs: Self) {
        self.records += rhs.records;
        self.errors += rhs.errors;
        self.warnings += rhs.warnings;
        self.infos += rhs.infos;
    }
}
