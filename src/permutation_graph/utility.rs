use std::{iter::Product, ops::Add};

pub fn are_unique<'a, T: ?Sized, I>(nodes: &'a T) -> bool
where
  &'a T: IntoIterator<Item = &'a I>,
  I: PartialEq + 'a,
{
  let mut is_valid = true;
  for (index_out, permutation_out) in nodes.into_iter().enumerate() {
    for (index_in, permutation_in) in nodes.into_iter().enumerate() {
      if index_out != index_in {
        is_valid = is_valid && permutation_out != permutation_in;
      }
      if !is_valid {
        break;
      }
    }

    if !is_valid {
      break;
    }
  }

  is_valid
}

pub fn factorial<T>(n: T) -> T
where
  T: Product<usize> + PartialEq<usize> + Add<usize, Output = usize>,
{
  if n == 0 { n } else { (1..n + 1).product() }
}

#[cfg(test)]
pub mod test {
  use std::sync::Arc;

  use permutations::{Permutation, Permutations};

  use crate::permutation_graph::permutation_node::{
    PermutationNode, StrongNodeVector, WeakNodeVector,
  };

  use super::*;

  pub fn set_up_node_vector(n: usize) -> StrongNodeVector {
    Permutations::new(n)
      .iter()
      .map(|perm| Arc::new(PermutationNode::new(perm)))
      .collect()
  }

  pub fn create_mock_permutation_node() -> Arc<PermutationNode> {
    Arc::new(PermutationNode::new(Permutation::identity(1)))
  }

  pub fn get_valid_permutation(
    valid_permutations: &WeakNodeVector,
    index: usize,
  ) -> Arc<PermutationNode> {
    Arc::clone(
      &valid_permutations[index]
        .upgrade()
        .expect("Node should exist"),
    )
  }

  pub fn build_node_graph(nodes: &StrongNodeVector) {
    for node in nodes.iter() {
      node.extract_valid_permutations(&nodes);
    }
  }

  #[test]
  fn can_check_if_permutations_vector_has_unique_members() {
    let nodes = set_up_node_vector(3);
    assert!(are_unique(&nodes));
  }

  #[test]
  fn can_check_if_permutations_vector_has_duplicate_members() {
    let mut nodes = set_up_node_vector(3);
    nodes.push(Arc::new(PermutationNode::new(Permutation::identity(3))));
    assert!(!are_unique(&nodes));
  }

  #[test]
  fn can_check_a_single_length_vector_is_unique() {
    let nodes = vec![Arc::new(PermutationNode::new(Permutation::identity(3)))];
    assert!(are_unique(&nodes));
  }

  #[test]
  fn can_check_a_zero_length_vector_is_unique() {
    let empty: Vec<u8> = vec![];
    assert!(are_unique(&empty));
  }

  #[test]
  fn can_get_the_factorial_of_a_number() {
    assert_eq!(factorial(0), 0);
    assert_eq!(factorial(1), 1);
    assert_eq!(factorial(2), 2);
    assert_eq!(factorial(3), 6);
    assert_eq!(factorial(4), 24);
  }
}
