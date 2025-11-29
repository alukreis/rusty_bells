mod change_validation;

use std::sync::{Arc, OnceLock, Weak};

use permutations::Permutation;

use change_validation::is_valid_change;

pub type WeakNodeVector = Vec<Weak<PermutationNode>>;
pub type StrongNodeVector = Vec<Arc<PermutationNode>>;

#[derive(Debug)]
pub struct PermutationNode {
  is_rounds: bool,
  permutation: Vec<u8>,
  valid_permutations: OnceLock<WeakNodeVector>,
}

impl PermutationNode {
  pub fn new(permutation: Permutation) -> PermutationNode {
    let identity = Permutation::identity(permutation.len());
    let range: Vec<u8> = (1u8..(permutation.len() as u8) + 1u8).collect();
    PermutationNode {
      is_rounds: permutation.eq(&identity),
      permutation: permutation.permute(&range),
      valid_permutations: OnceLock::new(),
    }
  }

  fn is_rounds(&self) -> bool {
    self.is_rounds
  }

  pub fn get_permutation(&self) -> &Vec<u8> {
    &self.permutation
  }

  pub fn extract_valid_permutations(&self, permutations: &[Arc<PermutationNode>]) {
    self
      .valid_permutations
      .set(self.collect_valid_permutations(permutations))
      .expect("Valid permutations must only be set once");
  }

  fn collect_valid_permutations(&self, permutations: &[Arc<PermutationNode>]) -> WeakNodeVector {
    permutations
      .iter()
      .filter(|perm| self.is_valid_permutation(perm))
      .map(Arc::downgrade)
      .collect()
  }

  fn is_valid_permutation(&self, permutation: &Arc<PermutationNode>) -> bool {
    !permutation.is_rounds()
      && permutation.permutation != self.permutation
      && is_valid_change(self.get_permutation(), permutation.get_permutation())
  }

  pub fn get_valid_permutations(&self) -> &WeakNodeVector {
    self
      .valid_permutations
      .get()
      .expect("Valid permutations should be initialised before use")
  }
}

impl PartialEq for PermutationNode {
  fn eq(&self, other: &Self) -> bool {
    self.permutation == other.permutation
  }
}

#[cfg(test)]
mod test {
  use crate::permutation_graph::utility::test::{build_node_graph, get_valid_permutation};

  use super::*;

  use permutations::Permutations;

  #[test]
  fn can_get_permutation() {
    let node = PermutationNode::new(Permutation::identity(3));
    assert_eq!(vec![1, 2, 3], *node.get_permutation());
  }

  #[test]
  fn initialises_rounds_node_with_flag() {
    let permutation = Permutation::identity(3);
    let node = PermutationNode::new(permutation);

    assert!(node.is_rounds());
  }

  #[test]
  fn initialises_non_rounds_node_with_flag() {
    let identity_permutation = Permutation::identity(3);
    let permutation = Permutations::new(3).get(1).unwrap();
    assert_ne!(permutation, identity_permutation);

    let node = PermutationNode::new(permutation);
    assert!(!node.is_rounds());
  }

  #[test]
  fn can_set_and_get_valid_permutations() {
    let permutations = Permutations::new(2);

    let nodes: StrongNodeVector = permutations
      .iter()
      .map(|perm| Arc::new(PermutationNode::new(perm)))
      .collect();
    build_node_graph(&nodes);

    let valid_perms = nodes[0].get_valid_permutations();
    let first_first_valid = get_valid_permutation(&valid_perms, 0);

    assert_eq!(nodes[0].get_valid_permutations().len(), 1);
    assert_eq!(first_first_valid.get_permutation(), &vec![2, 1]);
    assert!(nodes[1].get_valid_permutations().is_empty());
  }

  #[test]
  #[should_panic(expected = "Valid permutations must only be set once")]
  fn valid_permutations_panic_if_setting_a_second_time() {
    let permutations = Permutations::new(2);

    let nodes: StrongNodeVector = permutations
      .iter()
      .map(|perm| Arc::new(PermutationNode::new(perm)))
      .collect();
    build_node_graph(&nodes);
    build_node_graph(&nodes);
  }

  #[test]
  fn can_set_and_get_valid_permutations_length_three() {
    let permutations = Permutations::new(3);

    let nodes: StrongNodeVector = permutations
      .iter()
      .map(|perm| Arc::new(PermutationNode::new(perm)))
      .collect();
    build_node_graph(&nodes);

    assert_eq!(nodes.len(), 6);

    /*
       1: [1, 2, 3] -> [1, 3, 2],[2, 1, 3]
       2: [1, 3, 2] -> [3, 1, 2]
       3: [2, 1, 3] -> [2, 3, 1]
       4: [2, 3, 1] -> [2, 1, 3],[3, 2, 1]
       5: [3, 1, 2] -> [1, 3, 2],[3, 2, 1]
       6: [3, 2, 1] -> [2, 3, 1],[3, 1, 2]
    */
    let valid_permutations1 = nodes[0].get_valid_permutations();
    let valid_permutations2 = nodes[1].get_valid_permutations();
    let valid_permutations3 = nodes[2].get_valid_permutations();
    let valid_permutations4 = nodes[3].get_valid_permutations();
    let valid_permutations5 = nodes[4].get_valid_permutations();
    let valid_permutations6 = nodes[5].get_valid_permutations();

    assert_eq!(valid_permutations1.len(), 2);
    assert_eq!(
      get_valid_permutation(&valid_permutations1, 0).get_permutation(),
      &vec![1, 3, 2]
    );
    assert_eq!(
      get_valid_permutation(&valid_permutations1, 1).get_permutation(),
      &vec![2, 1, 3]
    );

    assert_eq!(valid_permutations2.len(), 1);
    assert_eq!(
      get_valid_permutation(&valid_permutations2, 0).get_permutation(),
      &vec![3, 1, 2]
    );

    assert_eq!(valid_permutations3.len(), 1);
    assert_eq!(
      get_valid_permutation(&valid_permutations3, 0).get_permutation(),
      &vec![2, 3, 1]
    );

    assert_eq!(valid_permutations4.len(), 2);
    assert_eq!(
      get_valid_permutation(&valid_permutations4, 0).get_permutation(),
      &vec![2, 1, 3]
    );
    assert_eq!(
      get_valid_permutation(&valid_permutations4, 1).get_permutation(),
      &vec![3, 2, 1]
    );

    assert_eq!(valid_permutations5.len(), 2);
    assert_eq!(
      get_valid_permutation(&valid_permutations5, 0).get_permutation(),
      &vec![1, 3, 2]
    );
    assert_eq!(
      get_valid_permutation(&valid_permutations5, 1).get_permutation(),
      &vec![3, 2, 1]
    );

    assert_eq!(valid_permutations6.len(), 2);
    assert_eq!(
      get_valid_permutation(&valid_permutations6, 0).get_permutation(),
      &vec![2, 3, 1]
    );
    assert_eq!(
      get_valid_permutation(&valid_permutations6, 1).get_permutation(),
      &vec![3, 1, 2]
    );
  }

  #[test]
  fn does_not_leak_memory() {
    let permutations = Permutations::new(2);

    let node1 = Arc::new(PermutationNode::new(
      permutations.get(0).expect("Should be two permutations"),
    ));
    let node2 = Arc::new(PermutationNode::new(
      permutations.get(1).expect("Should be two permutations"),
    ));

    node1
      .valid_permutations
      .set(vec![Arc::downgrade(&node2), Arc::downgrade(&node1)])
      .expect("This should be the first set");

    let weak1 = Arc::downgrade(&node1);
    let weak2 = Arc::downgrade(&node2);

    drop(node1);
    drop(node2);

    assert_eq!(weak1.upgrade(), None);
    assert_eq!(weak2.upgrade(), None);
  }
}
