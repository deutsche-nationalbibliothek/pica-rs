use sophia::graph::MutableGraph;
use sophia::term::literal::Literal;

pub type StrLiteral = Literal<Box<str>>;

pub trait Concept {
    fn skosify<G: MutableGraph>(&self, graph: &mut G);
}
