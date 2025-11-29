use crate::permutation_graph::permutation_node::StrongNodeVector;

#[derive(Debug, PartialEq)]
pub enum ComparisonStatus {
  End(usize),
  Methods(StrongNodeVector, StrongNodeVector),
  NextIndex(usize),
}
