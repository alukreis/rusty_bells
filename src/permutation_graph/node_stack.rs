mod node_and_index;

use std::sync::Arc;

use crate::permutation_graph::{
  node_stack::node_and_index::NodeAndIndex,
  permutation_node::{PermutationNode, StrongNodeVector},
  utility::{are_unique, factorial},
};

pub enum PushStatus {
  End,
  Next,
  HalfMethod(NodeChain),
}

pub enum NodeChain {
  Unique(StrongNodeVector),
  Duplicates,
}

pub struct NodeStack {
  stack_max: usize,
  node_stack: Vec<NodeAndIndex>,
}

impl NodeStack {
  pub fn new(first_node: &Arc<PermutationNode>) -> NodeStack {
    let stack_max = factorial(first_node.get_permutation().len()) / 2;

    NodeStack {
      stack_max,
      node_stack: Self::initialise_node_stack(first_node, stack_max),
    }
  }

  fn initialise_node_stack(
    first_node: &Arc<PermutationNode>,
    stack_max: usize,
  ) -> Vec<NodeAndIndex> {
    let mut node_stack = Vec::with_capacity(stack_max);
    node_stack.push(NodeAndIndex::new(first_node));

    node_stack
  }

  pub fn pop(&mut self) {
    self.node_stack.pop();
  }

  pub fn push_next(&mut self) -> PushStatus {
    if self.node_stack.len() == self.stack_max {
      return PushStatus::HalfMethod(self.get_unique_node_chain());
    }

    let node_and_index = self
      .node_stack
      .last_mut()
      .expect("Node stack should always contain at least one node");

    match node_and_index.next() {
      Some(node) => {
        self.node_stack.push(node);
        PushStatus::Next
      }
      None => PushStatus::End,
    }
  }

  pub fn is_empty(&self) -> bool {
    self.node_stack.is_empty()
  }

  pub fn get_unique_node_chain(&self) -> NodeChain {
    let chain = self
      .node_stack
      .iter()
      .map(|node_and_index| Arc::clone(node_and_index.get_node()))
      .collect();

    if are_unique(&chain) {
      NodeChain::Unique(chain)
    } else {
      NodeChain::Duplicates
    }
  }
}

#[cfg(test)]
mod test {
  use crate::permutation_graph::{
    node_stack::node_and_index::NodeAndIndex,
    utility::test::{build_node_graph, create_mock_permutation_node, set_up_node_vector},
  };

  use super::*;

  #[test]
  fn can_make_a_node_stack() {
    let node = create_mock_permutation_node();
    let node_stack = NodeStack::new(&node);
    let node_and_index = NodeAndIndex::new(&node);

    assert_eq!(node_stack.node_stack, vec![node_and_index]);
  }

  #[test]
  fn can_pop_the_stack() {
    let node = create_mock_permutation_node();
    let mut node_stack = NodeStack::new(&node);
    node_stack.pop();

    assert_eq!(node_stack.node_stack, vec![]);
  }

  #[test]
  fn can_push_next_node() {
    let nodes = set_up_node_vector(3);
    let node1 = &nodes[0];
    let mut node_stack = NodeStack::new(node1);

    node1.extract_valid_permutations(&nodes);

    let PushStatus::Next = node_stack.push_next() else {
      panic!("Should have a valid next node");
    };
  }

  #[test]
  fn push_next_indicates_if_no_more_next_nodes() {
    let nodes = set_up_node_vector(3);
    let node1 = &nodes[0];
    let mut node_stack = NodeStack::new(node1);

    node1.extract_valid_permutations(&nodes);

    node_stack.push_next();
    node_stack.pop(); // pop back to first node
    node_stack.push_next();
    node_stack.pop(); // pop back to first node

    let PushStatus::End = node_stack.push_next() else {
      panic!("Should be at the end of the valid nodes");
    };
  }

  #[test]
  fn push_next_passes_back_a_valid_half_method() {
    let nodes = set_up_node_vector(2);
    let node1 = &nodes[0];
    let mut node_stack = NodeStack::new(node1);

    node1.extract_valid_permutations(&nodes);

    node_stack.push_next();

    let PushStatus::HalfMethod(_) = node_stack.push_next() else {
      panic!("Should be a valid half method");
    };
  }

  #[test]
  fn can_get_unique_node_chain() {
    let nodes: StrongNodeVector = set_up_node_vector(2);
    build_node_graph(&nodes);
    let node1 = &nodes[0];

    let mut node_stack = NodeStack::new(node1);

    if let PushStatus::HalfMethod(chain) = node_stack.push_next() {
      match chain {
        NodeChain::Unique(half_method) => assert_eq!(vec![Arc::clone(node1)], half_method),
        NodeChain::Duplicates => panic!("Unexpected duplicates in test"),
      }
    }
  }

  #[test]
  fn can_get_duplicate_node_chain() {
    let nodes: StrongNodeVector = set_up_node_vector(3);
    build_node_graph(&nodes);

    let mut node_stack = NodeStack::new(&nodes[1]);
    node_stack.push_next(); // [3, 1, 2]
    node_stack.push_next(); // [1, 3, 2]

    if let PushStatus::HalfMethod(chain) = node_stack.push_next() {
      if let NodeChain::Unique(_) = chain {
        panic!("Unexpected unique in test")
      }
    } else {
      panic!("Expected a half method")
    }
  }

  #[test]
  fn can_check_is_empty() {
    let node = create_mock_permutation_node();
    let mut node_stack = NodeStack::new(&node);

    assert!(!node_stack.is_empty());

    node_stack.pop();

    assert!(node_stack.is_empty());
  }
}
