/// A single RDF term: the value bound to a variable in one result row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RDFTerm {
    pub value: Box<str>,
    pub kind: RDFType,
}

impl RDFTerm {
    pub fn is_iri(&self) -> bool {
        matches!(self.kind, RDFType::IRI)
    }

    pub fn is_literal(&self) -> bool {
        matches!(self.kind, RDFType::Literal { .. })
    }

    pub fn is_blank_node(&self) -> bool {
        matches!(self.kind, RDFType::BlankNode)
    }

    /// Returns the language tag if this is a language-tagged literal.
    pub fn lang(&self) -> Option<&str> {
        if let RDFType::Literal { lang, .. } = &self.kind {
            lang.as_deref()
        } else {
            None
        }
    }

    /// Returns the datatype IRI if this is a datatyped literal.
    pub fn datatype(&self) -> Option<&str> {
        if let RDFType::Literal { datatype, .. } = &self.kind {
            datatype.as_deref()
        } else {
            None
        }
    }
}

/// The type of an RDF term.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RDFType {
    /// An IRI.
    IRI,
    /// A literal, optionally with a language tag or datatype IRI.
    Literal {
        lang: Option<Box<str>>,
        datatype: Option<Box<str>>,
    },
    /// A blank node.
    BlankNode,
}
