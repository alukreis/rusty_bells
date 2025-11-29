mod method_iterator;
mod node_stack;
mod utility;

pub mod permutation_node;

use std::{
  sync::{Arc, Weak},
  thread,
};

use permutations::Permutations;

use method_iterator::MethodIterator;
use permutation_node::PermutationNode;

use crate::permutation_graph::{
  node_stack::{NodeChain, NodeStack, PushStatus},
  permutation_node::StrongNodeVector,
};

pub struct PermutationGraph {
  nodes: Vec<Arc<PermutationNode>>,
}

impl PermutationGraph {
  pub fn new(bells_amount: usize) -> PermutationGraph {
    let nodes = if bells_amount == 0 {
      Vec::new()
    } else {
      Permutations::new(bells_amount)
        .iter()
        .map(|perm| Arc::new(PermutationNode::new(perm)))
        .collect()
    };

    PermutationGraph::build_node_graph(&nodes);

    PermutationGraph { nodes }
  }

  fn build_node_graph(nodes: &StrongNodeVector) {
    for node in nodes.iter() {
      node.extract_valid_permutations(nodes);
    }
  }

  pub fn get_valid_full_methods(&self) -> MethodIterator {
    if self.nodes.is_empty() {
      method_iterator::new(vec![])
    } else {
      let half_methods = self.get_valid_half_methods();
      method_iterator::new(half_methods)
    }
  }

  fn get_valid_half_methods(&self) -> Vec<StrongNodeVector> {
    let rounds_node = Arc::clone(&self.nodes[0]);

    if self.nodes.len() == 1 {
      return vec![vec![Arc::clone(&rounds_node)]];
    }

    let join_handles: Vec<thread::JoinHandle<Vec<StrongNodeVector>>> = rounds_node
      .get_valid_permutations()
      .iter()
      .map(|change| self.spawn_method_traversal_thread(change))
      .collect();

    join_handles
      .into_iter()
      .flat_map(|handle| handle.join().unwrap())
      .collect()
  }

  fn spawn_method_traversal_thread(
    &self,
    first_change: &Weak<PermutationNode>,
  ) -> thread::JoinHandle<Vec<StrongNodeVector>> {
    let rounds_node = Arc::clone(&self.nodes[0]);
    let change_clone = Weak::clone(first_change);

    thread::spawn(move || Self::get_node_half_methods(change_clone, rounds_node))
  }

  fn get_node_half_methods(
    node: Weak<PermutationNode>,
    rounds_node: Arc<PermutationNode>,
  ) -> Vec<StrongNodeVector> {
    let strong_node = node.upgrade().expect("Node should exist");

    let mut node_stack = NodeStack::new(&strong_node);

    let mut half_methods = Vec::new();
    while !node_stack.is_empty() {
      match node_stack.push_next() {
        PushStatus::End => node_stack.pop(),
        PushStatus::Next => (),
        PushStatus::HalfMethod(chain) => {
          if let NodeChain::Unique(mut half_method) = chain {
            let mut method_with_rounds = vec![Arc::clone(&rounds_node)];
            method_with_rounds.append(&mut half_method);
            half_methods.push(method_with_rounds);
          }
          node_stack.pop();
        }
      };
    }

    half_methods
  }
}

#[cfg(test)]
mod test {
  use crate::permutation_graph::utility::test::get_valid_permutation;

  use super::*;

  #[test]
  fn handles_no_bells() {
    let graph = PermutationGraph::new(0);
    assert!(graph.nodes.is_empty());
  }

  #[test]
  fn handles_one_bell() {
    let graph = PermutationGraph::new(1);
    assert_eq!(graph.nodes.len(), 1);
  }

  #[test]
  fn handles_many_bells() {
    let graph1 = PermutationGraph::new(5);
    let graph2 = PermutationGraph::new(6);

    assert_eq!(graph1.nodes.len(), 120);
    assert_eq!(graph2.nodes.len(), 720);
  }

  #[test]
  fn permutation_nodes_have_all_valid_changes() {
    let graph = PermutationGraph::new(2);

    let valid_permutations1 = graph.nodes[0].get_valid_permutations();

    assert_eq!(valid_permutations1.len(), 1);
    assert_eq!(
      get_valid_permutation(&valid_permutations1, 0).get_permutation(),
      &vec![2, 1]
    );
    assert!(
      get_valid_permutation(&valid_permutations1, 0)
        .get_valid_permutations()
        .is_empty()
    );
  }

  #[test]
  fn can_get_all_half_methods_without_repeats() {
    let graph = PermutationGraph::new(3);

    /*
        132 312 321
        213 231 321
    */
    let half_methods = graph.get_valid_half_methods();

    assert_eq!(half_methods.len(), 2);

    let expected_method1: Vec<&[u8; 3]> = vec![&[1, 2, 3], &[1, 3, 2], &[3, 1, 2], &[3, 2, 1]];
    let expected_method2 = vec![&[1, 2, 3], &[2, 1, 3], &[2, 3, 1], &[3, 2, 1]];

    assert!(half_methods.iter().any(|method| {
      method
        .iter()
        .enumerate()
        .all(|(index, node)| node.get_permutation() == &expected_method1[index])
    }));
    assert!(half_methods.iter().any(|method| {
      method
        .iter()
        .enumerate()
        .all(|(index, node)| node.get_permutation() == &expected_method2[index])
    }));
  }

  #[test]
  fn can_get_all_valid_methods() {
    let graph = PermutationGraph::new(3);
    let method_iterator = graph.get_valid_full_methods();
    let method1 = [
      [1, 2, 3],
      [1, 3, 2],
      [3, 1, 2],
      [3, 2, 1],
      [2, 3, 1],
      [2, 1, 3],
      [1, 2, 3],
    ];
    let method2: Vec<[u8; 3]> = method1.iter().rev().map(|slice| slice.clone()).collect();
    let valid_methods = collect_full_methods(method_iterator);

    assert_eq!(valid_methods.len(), 2);

    let valid_method1 = &valid_methods[0];
    let valid_method2 = &valid_methods[1];

    assert_eq!(valid_method1.len(), method1.len());
    assert_eq!(valid_method1.len(), valid_method2.len());

    assert!(valid_methods.iter().any(|method| {
      method
        .iter()
        .enumerate()
        .all(|(index, node)| node.get_permutation() == &method1[index])
    }));
    assert!(valid_methods.iter().any(|method| {
      method
        .iter()
        .enumerate()
        .all(|(index, node)| node.get_permutation() == &method2[index])
    }));
  }

  #[test]
  fn can_handle_getting_two_bell_methods() {
    let graph = PermutationGraph::new(2);
    let method_iterator = graph.get_valid_full_methods();

    let valid_methods = collect_full_methods(method_iterator);
    assert_eq!(valid_methods.len(), 1);

    let permutations: Vec<&Vec<u8>> = valid_methods[0]
      .iter()
      .map(|node| node.get_permutation())
      .collect();

    assert_eq!(permutations, vec![&[1, 2], &[2, 1], &[1, 2],]);
  }

  #[test]
  fn can_handle_getting_one_bell_method() {
    let graph = PermutationGraph::new(1);
    let method_iterator = graph.get_valid_full_methods();
    let valid_methods = collect_full_methods(method_iterator);

    assert_eq!(valid_methods.len(), 1);

    let permutations: Vec<&Vec<u8>> = valid_methods[0]
      .iter()
      .map(|node| node.get_permutation())
      .collect();

    assert_eq!(permutations, vec![&[1],]);
  }

  #[test]
  fn can_handle_getting_zero_bells() {
    let graph = PermutationGraph::new(0);
    let method_iterator = graph.get_valid_full_methods();
    let valid_methods = collect_full_methods(method_iterator);

    assert_eq!(valid_methods.len(), 1);

    let permutations: Vec<&Vec<u8>> = valid_methods[0]
      .iter()
      .map(|node| node.get_permutation())
      .collect();

    assert_eq!(permutations, Vec::<&Vec<u8>>::new());
  }

  #[test]
  fn can_get_a_node_half_method_job() {
    let graph = PermutationGraph::new(3);
    let rounds_node = &graph.nodes[0];
    let valid_changes = rounds_node.get_valid_permutations();
    let valid_change1 = &valid_changes[0];
    let valid_change2 = &valid_changes[1];

    let half_methods1 =
      PermutationGraph::get_node_half_methods(Weak::clone(valid_change1), Arc::clone(&rounds_node));
    let half_methods2 =
      PermutationGraph::get_node_half_methods(Weak::clone(valid_change2), Arc::clone(&rounds_node));

    assert_eq!(half_methods1.len(), 1);
    assert_eq!(half_methods2.len(), 1);

    let first_half = &half_methods1[0];
    let second_half = &half_methods2[0];

    assert_eq!(first_half.len(), 4);
    assert_eq!(second_half.len(), 4);

    assert_eq!(first_half[0].get_permutation(), &[1, 2, 3]);
    assert_eq!(first_half[1].get_permutation(), &[1, 3, 2]);
    assert_eq!(first_half[2].get_permutation(), &[3, 1, 2]);
    assert_eq!(first_half[3].get_permutation(), &[3, 2, 1]);

    assert_eq!(second_half[0].get_permutation(), &[1, 2, 3]);
    assert_eq!(second_half[1].get_permutation(), &[2, 1, 3]);
    assert_eq!(second_half[2].get_permutation(), &[2, 3, 1]);
    assert_eq!(second_half[3].get_permutation(), &[3, 2, 1]);
  }

  fn collect_full_methods(
    method_iterator: impl Iterator<Item = StrongNodeVector>,
  ) -> Vec<StrongNodeVector> {
    let mut valid_methods = Vec::new();
    for method in method_iterator {
      valid_methods.push(method);
    }

    valid_methods
  }
}
