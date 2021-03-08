#[macro_export]
macro_rules! add_namespace {
    ($graph:ident, $prefix:expr, $uri:expr) => {
        $graph.add_namespace(&rdf::namespace::Namespace::new(
            $prefix.to_string(),
            Uri::new($uri.to_string()),
        ));
    };
}

#[macro_export]
macro_rules! add_triple {
    ($graph:ident, $subject:expr, $predicate:expr, $object:expr) => {
        $graph.add_triple(&Triple::new($subject, $predicate, $object));
    };
}

#[macro_export]
macro_rules! xsd {
    ($label:expr) => {
        format!("http://www.w3.org/2001/XMLSchema#{}", $label);
    };
}

#[macro_export]
macro_rules! dcterms {
    ($label:expr) => {
        format!("http://purl.org/dc/terms/{}", $label);
    };
}

#[macro_export]
macro_rules! skos {
    ($label:expr) => {
        format!("http://www.w3.org/2004/02/skos/core#{}", $label);
    };
}

#[macro_export]
macro_rules! rdf {
    ($label:expr) => {
        format!("http://www.w3.org/1999/02/22-rdf-syntax-ns#{}", $label);
    };
}

#[macro_export]
macro_rules! gnd {
    ($idn:expr) => {
        format!("http://d-nb.info/gnd/{}", $idn);
    };
}
