use std::sync::Arc;

use crate::permutation_graph::permutation_node::PermutationNode;

#[derive(Debug)]
pub struct NodeAndIndex {
  node: Arc<PermutationNode>,
  index: usize,
}

impl NodeAndIndex {
  pub fn new(node: &Arc<PermutationNode>) -> NodeAndIndex {
    NodeAndIndex {
      node: Arc::clone(node),
      index: 0,
    }
  }

  pub fn get_node(&self) -> &Arc<PermutationNode> {
    &self.node
  }
}

impl Iterator for NodeAndIndex {
  type Item = NodeAndIndex;

  fn next(&mut self) -> Option<Self::Item> {
    let valid_permutations = self.node.get_valid_permutations();

    let next_node = valid_permutations.get(self.index)?.upgrade()?;
    self.index += 1;
    Some(NodeAndIndex::new(&next_node))
  }
}

impl PartialEq for NodeAndIndex {
  fn eq(&self, other: &Self) -> bool {
    self.node == other.node
  }
}

#[cfg(test)]
mod test {
  use crate::permutation_graph::utility::test::{
    build_node_graph, create_mock_permutation_node, set_up_node_vector,
  };

  use super::*;

  #[test]
  fn can_create_node_and_index_struct() {
    let permutation_node = create_mock_permutation_node();
    let node_and_index = NodeAndIndex::new(&permutation_node);

    assert_eq!(node_and_index.index, 0);
    assert_eq!(node_and_index.node, permutation_node);
  }

  #[test]
  fn can_get_next_node() {
    let nodes = set_up_node_vector(2);
    build_node_graph(&nodes);

    let mut node_and_index = NodeAndIndex::new(&nodes[0]);

    if let None = node_and_index.next() {
      panic!("Should have a next node");
    }
  }

  #[test]
  fn next_node_is_none_if_none_left() {
    let nodes = set_up_node_vector(2);
    build_node_graph(&nodes);

    let mut node_and_index = NodeAndIndex::new(&nodes[0]);

    if let None = node_and_index.next() {
      panic!("Should have a next node");
    }
    if let Some(_) = node_and_index.next() {
      panic!("Should not have a next node");
    }
  }

  #[test]
  fn can_get_node() {
    let nodes = set_up_node_vector(2);
    build_node_graph(&nodes);

    let node = &nodes[0];
    let node_and_index = NodeAndIndex::new(node);

    assert_eq!(node_and_index.get_node(), node);
  }

  #[test]
  fn can_check_equality() {
    let nodes = set_up_node_vector(2);

    let node1 = &nodes[0];
    let node2 = &nodes[1];

    let node_and_index1 = NodeAndIndex::new(node1);
    let node_and_index2 = NodeAndIndex::new(node2);

    assert_eq!(&node_and_index1, &node_and_index1);
    assert!(!(&node_and_index1 == &node_and_index2));
  }
}
