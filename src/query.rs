pub mod ask;
pub mod select;

#[derive(Debug)]
pub enum QueryType {
    Select,
    Construct,
    Describe,
    Ask,
}

impl From<&spargebra::Query> for QueryType {
    fn from(value: &spargebra::Query) -> Self {
        match value {
            spargebra::Query::Select { .. } => Self::Select,
            spargebra::Query::Construct { .. } => Self::Construct,
            spargebra::Query::Describe { .. } => Self::Describe,
            spargebra::Query::Ask { .. } => Self::Ask,
        }
    }
}
