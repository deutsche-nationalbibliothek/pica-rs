// use pica::Record;

pub trait Concept {
    fn idn(&self) -> String;
    fn pref_label(&self) -> Option<String>;
    fn alt_labels(&self) -> Vec<String>;
    fn hidden_labels(&self) -> Vec<String> {
        vec![]
    }
}
