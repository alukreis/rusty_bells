use crate::permutation_graph::permutation_node::StrongNodeVector;

mod perms;
mod permutation_graph;

pub use permutation_graph::PermutationGraph;

pub fn print_method(index: usize, method: &StrongNodeVector) {
  println!("{index}");
  for node in method.iter() {
    println!("{:?},", node.get_permutation());
  }
}
