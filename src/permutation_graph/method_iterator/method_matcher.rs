use std::sync::Arc;

use crate::permutation_graph::permutation_node::{PermutationNode, StrongNodeVector};

pub struct MethodMatcher<'a> {
  half_methods: &'a [StrongNodeVector],
  current_method: &'a StrongNodeVector,
  current_end_node: Arc<PermutationNode>,
}

impl<'a> MethodMatcher<'a> {
  pub fn new(half_methods: &'a [StrongNodeVector], current_index: usize) -> MethodMatcher<'a> {
    let current_method = &half_methods[current_index];
    MethodMatcher {
      half_methods,
      current_method,
      current_end_node: Arc::clone(current_method.last().unwrap()),
    }
  }

  pub fn get_full_method_from_end_node_match(
    &self,
    comparison_index: usize,
  ) -> Option<StrongNodeVector> {
    let Self {
      half_methods,
      current_method,
      current_end_node,
    } = self;

    let comparison_method = Self::get_method_at_index(half_methods, comparison_index);
    let comparison_end_node = Self::get_last_node(comparison_method);

    if current_end_node == comparison_end_node {
      Some(Self::build_full_method(current_method, comparison_method))
    } else {
      None
    }
  }

  fn get_method_at_index(half_methods: &[StrongNodeVector], index: usize) -> &StrongNodeVector {
    half_methods.get(index).expect("Index should be valid")
  }

  fn get_last_node(half_method: &StrongNodeVector) -> &Arc<PermutationNode> {
    half_method.last().expect("All methods should be populated")
  }

  fn build_full_method(method1: &StrongNodeVector, method2: &StrongNodeVector) -> StrongNodeVector {
    let mut full_method: StrongNodeVector = Vec::with_capacity((method1.len() * 2) - 1);

    for permutation in method1.iter() {
      full_method.push(Arc::clone(permutation));
    }

    // Miss out "joining" node since method1 has it at the end
    for permutation in method2[..method2.len() - 1].iter().rev() {
      full_method.push(Arc::clone(permutation));
    }

    full_method
  }
}

#[cfg(test)]
mod test {
  use crate::permutation_graph::{
    permutation_node::StrongNodeVector,
    utility::test::{create_mock_permutation_node, set_up_node_vector},
  };

  use super::*;

  #[test]
  fn can_create_a_method_matcher() {
    let half_methods: Vec<StrongNodeVector> = vec![vec![create_mock_permutation_node()]];
    let current_index: usize = 0;

    let matcher = MethodMatcher::new(&half_methods, current_index);

    assert_eq!(matcher.half_methods, &half_methods);
    assert_eq!(matcher.current_method, &half_methods[current_index]);
    assert_eq!(
      matcher.current_end_node,
      *half_methods[current_index].last().unwrap()
    );
  }

  #[test]
  fn can_get_full_method_from_end_node_match() {
    let nodes = set_up_node_vector(2);
    let node1 = &nodes[0];
    let node2 = &nodes[1];

    let half_method = vec![Arc::clone(node1), Arc::clone(node2)];
    let half_methods: Vec<StrongNodeVector> = vec![half_method];
    let current_index: usize = 0;

    let matcher = MethodMatcher::new(&half_methods, current_index);

    let expected_full_method = vec![Arc::clone(node1), Arc::clone(node2), Arc::clone(node1)];

    assert_eq!(
      matcher
        .get_full_method_from_end_node_match(current_index)
        .unwrap(),
      expected_full_method
    );
  }

  #[test]
  fn returns_none_match_if_ends_do_not_match() {
    let nodes = set_up_node_vector(2);
    let node1 = &nodes[0];
    let node2 = &nodes[1];

    let half_method1 = vec![Arc::clone(node1), Arc::clone(node2)];
    let half_method2 = vec![Arc::clone(node2), Arc::clone(node1)];
    let half_methods: Vec<StrongNodeVector> = vec![half_method1, half_method2];
    let current_index: usize = 0;

    let matcher = MethodMatcher::new(&half_methods, current_index);

    assert_eq!(matcher.get_full_method_from_end_node_match(1), None);
  }
}
