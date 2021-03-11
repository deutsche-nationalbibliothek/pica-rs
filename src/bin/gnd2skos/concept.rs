use sophia::graph::MutableGraph;
use sophia::term::literal::Literal;

pub type StrLiteral = Literal<Box<str>>;

#[macro_export]
macro_rules! push_value {
    ($label:expr, $field:expr, $prefix:expr, $suffix:expr) => {
        if let Some(value) = $field {
            $label.push_str($prefix);
            $label.push_str(&value);
            $label.push_str($suffix);
        }
    };
    ($label:expr, $field:expr, $prefix:expr) => {
        if let Some(value) = $field {
            $label.push_str($prefix);
            $label.push_str(&value);
        }
    };
    ($label:expr, $field:expr) => {
        if let Some(value) = $field {
            $label.push_str(&value);
        }
    };
}

#[macro_export]
macro_rules! push_list {
    ($label:expr, $values:expr, $sep:expr, $prefix:expr, $suffix:expr) => {
        if !$values.is_empty() {
            $label.push_str($prefix);
            $label.push_str(&$values.join($sep));
            $label.push_str($suffix);
        }
    };
    ($label:expr, $values:expr, $sep:expr, $prefix:expr) => {
        if !$values.is_empty() {
            $label.push_str($prefix);
            $label.push_str(&$values.join($sep));
        }
    };
}

pub trait Concept {
    fn skosify<G: MutableGraph>(&self, graph: &mut G);
}
