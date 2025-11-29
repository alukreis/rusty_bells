mod channel_helper;
mod comparison_index_store;
mod comparison_status;
mod method_matcher;
mod thread_and_cup;

use std::{
  cmp::min,
  collections::HashMap,
  mem::swap,
  sync::{
    Arc,
    mpsc::{Receiver, Sender, channel},
  },
  thread,
};

use crate::permutation_graph::{
  method_iterator::{
    channel_helper::{recv_or_error, send_or_error},
    comparison_index_store::{ComparisonIndexStore, ComparisonIndexStoreBuilder},
    comparison_status::ComparisonStatus,
    method_matcher::MethodMatcher,
    thread_and_cup::ThreadAndCup,
  },
  permutation_node::StrongNodeVector,
  utility::are_unique,
};

pub type MethodIterator = Box<dyn Iterator<Item = StrongNodeVector>>;

const MAX_THREADS: usize = 4;

struct FullMethodIterator {
  shared_index: usize,
  status_channel: (Sender<ComparisonStatus>, Receiver<ComparisonStatus>),
  thread_and_cups: HashMap<usize, ThreadAndCup>,
  last_method_reverse: Option<StrongNodeVector>,
}

struct OneMethodIterator {
  done: bool,
  half_methods: Vec<StrongNodeVector>,
}

struct ZeroMethodIterator {
  done: bool,
}

pub fn new(half_methods: Vec<StrongNodeVector>) -> MethodIterator {
  if half_methods.is_empty() {
    Box::new(ZeroMethodIterator::new())
  } else if half_methods.len() == 1 {
    Box::new(OneMethodIterator::new(half_methods))
  } else {
    Box::new(FullMethodIterator::new(half_methods))
  }
}

impl FullMethodIterator {
  fn new(half_methods: Vec<StrongNodeVector>) -> FullMethodIterator {
    Self::validate_half_method_input(&half_methods);
    let (shared_index, thread_and_cups, status_channel) =
      Self::build_threads_start_index_and_status_receiver(half_methods);

    FullMethodIterator {
      shared_index,
      status_channel,
      thread_and_cups,
      last_method_reverse: None,
    }
  }

  fn validate_half_method_input(half_methods: &[StrongNodeVector]) {
    if half_methods.len() < 2 {
      panic!("Full method iterator expects greater than one half method");
    }
  }

  fn build_threads_start_index_and_status_receiver(
    half_methods: Vec<StrongNodeVector>,
  ) -> (
    usize,
    HashMap<usize, ThreadAndCup>,
    (Sender<ComparisonStatus>, Receiver<ComparisonStatus>),
  ) {
    let thread_total = min(half_methods.len() - 1, MAX_THREADS);
    let arc_half_methods = Arc::new(half_methods);
    let (status_sender, status_receiver) = channel();
    let mut hash_map = HashMap::with_capacity(thread_total);

    for thread_num in 0..thread_total {
      let status_sender = status_sender.clone();
      hash_map.insert(
        thread_num,
        Self::spawn_comparison_thread(thread_num, status_sender, Arc::clone(&arc_half_methods)),
      );
    }

    (thread_total + 1, hash_map, (status_sender, status_receiver))
  }

  fn get_reverse_method(method: &StrongNodeVector) -> StrongNodeVector {
    method.iter().rev().map(Arc::clone).collect()
  }

  fn consume_reverse_method(&mut self) -> Option<StrongNodeVector> {
    let mut method = None;
    swap(&mut self.last_method_reverse, &mut method);

    method
  }

  fn pair_matching_half_methods(
    half_methods: &[StrongNodeVector],
    index_store: &mut ComparisonIndexStore,
  ) -> Option<StrongNodeVector> {
    let matcher = MethodMatcher::new(half_methods, index_store.get_current_index());

    while index_store.is_running_comparison() {
      let full_method_match =
        matcher.get_full_method_from_end_node_match(index_store.get_comparison_index());

      index_store.increment_indexes();

      if let Some(method) = full_method_match {
        if are_unique(&method[..method.len() - 1]) {
          return Some(method);
        }
      }
    }

    None
  }

  fn spawn_comparison_thread(
    thread_num: usize,
    status_sender: Sender<ComparisonStatus>,
    half_methods: Arc<Vec<StrongNodeVector>>,
  ) -> ThreadAndCup {
    let (index_sender, index_receiver) = channel();

    let handle = thread::Builder::new()
      .name(thread_num.to_string())
      .spawn(move || {
        Self::run_comparisons(thread_num, half_methods, status_sender, index_receiver);
      })
      .expect("Thread should be valid");

    ThreadAndCup::new(handle, index_sender)
  }

  fn run_comparisons(
    thread_num: usize,
    half_methods: Arc<Vec<StrongNodeVector>>,
    status_sender: Sender<ComparisonStatus>,
    index_receiver: Receiver<usize>,
  ) {
    let mut index_store = ComparisonIndexStoreBuilder::new()
      .last_index(half_methods.len() - 1)
      .start_index(thread_num)
      .status_sender(status_sender.clone())
      .index_receiver(index_receiver)
      .build();

    while index_store.is_running() {
      if let Some(method) = Self::pair_matching_half_methods(&half_methods, &mut index_store) {
        let reverse_method = Self::get_reverse_method(&method);
        send_or_error(
          &status_sender,
          ComparisonStatus::Methods(method, reverse_method),
        );
      }

      index_store.increment_indexes();
    }

    send_or_error(&status_sender, ComparisonStatus::End(thread_num));
  }

  fn are_comparisons_running(thread_and_cups: &HashMap<usize, ThreadAndCup>) -> bool {
    thread_and_cups
      .values()
      .any(|thread_and_cup| thread_and_cup.is_thread_running())
  }
}

impl OneMethodIterator {
  fn new(half_methods: Vec<StrongNodeVector>) -> OneMethodIterator {
    Self::validate_half_method_input(&half_methods);
    OneMethodIterator {
      done: false,
      half_methods,
    }
  }

  fn validate_half_method_input(half_methods: &[StrongNodeVector]) {
    if half_methods.len() != 1 {
      panic!("One method iterator expects single half method");
    } else if half_methods[0].len() > 2 {
      panic!("One method iterator expects a half method with max length of two");
    }
  }
}

impl ZeroMethodIterator {
  fn new() -> ZeroMethodIterator {
    ZeroMethodIterator { done: false }
  }
}

impl Iterator for FullMethodIterator {
  type Item = StrongNodeVector;

  fn next(&mut self) -> Option<Self::Item> {
    let Self {
      shared_index,
      status_channel: (_, status_receiver),
      thread_and_cups,
      last_method_reverse,
    } = self;

    if last_method_reverse.is_some() {
      return self.consume_reverse_method();
    }

    while Self::are_comparisons_running(thread_and_cups) {
      match recv_or_error(status_receiver) {
        ComparisonStatus::End(thread_num) => {
          let thread_and_cup = thread_and_cups
            .remove(&thread_num)
            .expect("Key is taken directly from the hash map");
          thread_and_cup.join_thread().unwrap();
        }
        ComparisonStatus::Methods(method, reversed) => {
          *last_method_reverse = Some(reversed);
          return Some(method);
        }
        ComparisonStatus::NextIndex(thread_num) => {
          thread_and_cups
            .get(&thread_num)
            .expect("All running threads should have an entry.")
            .send_to_thread(*shared_index);

          *shared_index += 1;
        }
      }
    }

    None
  }
}

impl Iterator for OneMethodIterator {
  type Item = StrongNodeVector;

  fn next(&mut self) -> Option<Self::Item> {
    let Self { done, half_methods } = self;

    if *done {
      None
    } else {
      *done = true;
      let half_method = &half_methods[0];

      if half_method.len() == 1 {
        Some(half_method.clone())
      } else {
        let rounds_node = &half_method[0];
        let change_node = &half_method[1];

        Some(vec![
          Arc::clone(rounds_node),
          Arc::clone(change_node),
          Arc::clone(rounds_node),
        ])
      }
    }
  }
}

impl Iterator for ZeroMethodIterator {
  type Item = StrongNodeVector;

  fn next(&mut self) -> Option<Self::Item> {
    if self.done {
      None
    } else {
      self.done = true;
      Some(vec![])
    }
  }
}

#[cfg(test)]
mod test {
  use std::sync::Arc;

  use crate::permutation_graph::utility::test::set_up_node_vector;

  use super::*;

  #[test]
  fn can_create_a_zero_method_iterator() {
    ZeroMethodIterator::new();
  }

  #[test]
  fn can_create_a_one_method_iterator() {
    let mock_nodes = set_up_node_vector(2);
    let methods = vec![mock_nodes];
    OneMethodIterator::new(methods);
  }

  #[test]
  fn can_create_a_full_method_iterator() {
    let mock_nodes = set_up_node_vector(2);
    let methods = vec![mock_nodes.clone(), mock_nodes];
    FullMethodIterator::new(methods);
  }

  #[test]
  #[should_panic(expected = "One method iterator expects single half method")]
  fn one_method_iterator_panics_if_not_one_method() {
    let mock_nodes = set_up_node_vector(2);
    let methods = vec![mock_nodes.clone(), mock_nodes];
    OneMethodIterator::new(methods);
  }

  #[test]
  #[should_panic(expected = "One method iterator expects a half method with max length of two")]
  fn one_method_iterator_panics_if_half_method_longer_than_two() {
    let mock_nodes = set_up_node_vector(2);
    let node1 = &mock_nodes[0];
    let node2 = &mock_nodes[1];

    let methods = vec![vec![
      Arc::clone(node1),
      Arc::clone(node2),
      Arc::clone(node1),
    ]];
    OneMethodIterator::new(methods);
  }

  #[test]
  #[should_panic(expected = "Full method iterator expects greater than one half method")]
  fn full_method_iterator_panics_if_less_than_two_half_methods() {
    let mock_nodes = set_up_node_vector(2);
    let methods = vec![mock_nodes];
    FullMethodIterator::new(methods);
  }

  #[test]
  fn can_iterate_through_methods() {
    let nodes = set_up_node_vector(3);

    let node1 = &nodes[0]; // [1, 2, 3]
    let node2 = &nodes[1]; // [1, 3, 2]
    let node3 = &nodes[2]; // [2, 1, 3]
    let node4 = &nodes[3]; // [2, 3, 1]
    let node5 = &nodes[4]; // [3, 1, 2]
    let node6 = &nodes[5]; // [3, 2, 1]

    let half_method1 = vec![
      Arc::clone(node1),
      Arc::clone(node2),
      Arc::clone(node5),
      Arc::clone(node6),
    ];
    let half_method2 = vec![
      Arc::clone(node1),
      Arc::clone(node3),
      Arc::clone(node4),
      Arc::clone(node6),
    ];

    let mut iterator = new(vec![half_method1, half_method2]);
    let full_method1 = iterator.next().expect("There should be a method");
    let full_method2 = iterator.next().expect("There should be a method");
    let end = iterator.next();

    assert_eq!(
      full_method1,
      vec![
        Arc::clone(node1),
        Arc::clone(node2),
        Arc::clone(node5),
        Arc::clone(node6),
        Arc::clone(node4),
        Arc::clone(node3),
        Arc::clone(node1),
      ]
    );
    assert_eq!(
      full_method2,
      vec![
        Arc::clone(node1),
        Arc::clone(node3),
        Arc::clone(node4),
        Arc::clone(node6),
        Arc::clone(node5),
        Arc::clone(node2),
        Arc::clone(node1),
      ]
    );
    assert_eq!(end, None);
  }

  #[test]
  fn can_iterate_through_two_bell_methods() {
    let nodes = set_up_node_vector(2);
    let node1 = &nodes[0]; // [1, 2]
    let node2 = &nodes[1]; // [2, 1]
    let two_bell_half = vec![Arc::clone(node1), Arc::clone(node2)];
    let mut iterator = new(vec![two_bell_half]);

    let full_method = iterator.next().expect("There should be a method");
    let end = iterator.next();

    assert_eq!(
      full_method,
      vec![Arc::clone(node1), Arc::clone(node2), Arc::clone(node1),]
    );
    assert_eq!(end, None);
  }

  #[test]
  fn can_iterate_through_1_bell_methods() {
    let nodes = set_up_node_vector(1);
    let node1 = &nodes[0]; // [1]
    let one_bell_half = vec![Arc::clone(node1)];
    let mut iterator = new(vec![one_bell_half]);

    let full_method = iterator.next().expect("There should be a method");
    let end = iterator.next();

    assert_eq!(full_method, vec![Arc::clone(node1)]);
    assert_eq!(end, None);
  }

  #[test]
  fn can_iterate_through_zero_bell_methods() {
    let mut iterator = new(vec![]);

    let full_method = iterator.next().expect("There should be a method");
    let end = iterator.next();

    assert_eq!(full_method, vec![]);
    assert_eq!(end, None);
  }

  #[test]
  fn can_run_comparisons_over_half_methods() {
    let nodes = set_up_node_vector(3);

    let node1 = &nodes[0]; // [1, 2, 3]
    let node2 = &nodes[1]; // [1, 3, 2]
    let node3 = &nodes[2]; // [2, 1, 3]
    let node4 = &nodes[3]; // [2, 3, 1]
    let node5 = &nodes[4]; // [3, 1, 2]
    let node6 = &nodes[5]; // [3, 2, 1]

    let half_method1 = vec![
      Arc::clone(node1),
      Arc::clone(node2),
      Arc::clone(node5),
      Arc::clone(node6),
    ];
    let half_method2 = vec![
      Arc::clone(node1),
      Arc::clone(node3),
      Arc::clone(node4),
      Arc::clone(node6),
    ];

    let thread_num = 0;
    let half_methods = Arc::new(vec![half_method1, half_method2]);
    let (index_sender, index_receiver) = channel();
    let (status_sender, status_receiver) = channel();
    let full_method1 = vec![
      Arc::clone(node1),
      Arc::clone(node2),
      Arc::clone(node5),
      Arc::clone(node6),
      Arc::clone(node4),
      Arc::clone(node3),
      Arc::clone(node1),
    ];
    let full_method2 = vec![
      Arc::clone(node1),
      Arc::clone(node3),
      Arc::clone(node4),
      Arc::clone(node6),
      Arc::clone(node5),
      Arc::clone(node2),
      Arc::clone(node1),
    ];

    index_sender.send(1).unwrap();
    FullMethodIterator::run_comparisons(0, half_methods, status_sender.clone(), index_receiver);

    assert_eq!(
      status_receiver.recv().unwrap(),
      ComparisonStatus::Methods(full_method1, full_method2)
    );
    assert_eq!(
      status_receiver.recv().unwrap(),
      ComparisonStatus::NextIndex(thread_num)
    );
  }
}
