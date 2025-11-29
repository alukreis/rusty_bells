mod comparison_index_store_builder;

use std::sync::mpsc::{Receiver, Sender};

pub use comparison_index_store_builder::ComparisonIndexStoreBuilder;

use crate::permutation_graph::method_iterator::{
  channel_helper::{recv_or_error, send_or_error},
  comparison_status::ComparisonStatus,
};

pub struct ComparisonIndexStore {
  last_index: usize,
  start_index: usize,
  current_index: usize,
  status_sender: Sender<ComparisonStatus>,
  index_receiver: Receiver<usize>,
  comparison_index: usize,
}

impl ComparisonIndexStore {
  pub fn new(
    last_index: usize,
    start_index: usize,
    status_sender: Sender<ComparisonStatus>,
    index_receiver: Receiver<usize>,
  ) -> ComparisonIndexStore {
    ComparisonIndexStore {
      last_index,
      start_index,
      current_index: start_index,
      status_sender,
      index_receiver,
      comparison_index: Self::get_start_comparison_index(last_index, start_index),
    }
  }

  fn get_start_comparison_index(last_index: usize, start_index: usize) -> usize {
    if last_index == 0 && start_index == 0 {
      0
    } else {
      start_index + 1
    }
  }

  pub fn increment_indexes(&mut self) {
    let ComparisonIndexStore {
      last_index,
      start_index,
      current_index,
      status_sender,
      index_receiver,
      comparison_index,
    } = self;

    if *comparison_index == *last_index + 1 {
      send_or_error(status_sender, ComparisonStatus::NextIndex(*start_index));
      *current_index = recv_or_error(index_receiver);
      *comparison_index = *current_index + 1;
    } else {
      *comparison_index += 1;
    };
  }

  pub fn get_current_index(&self) -> usize {
    self.current_index
  }

  pub fn get_comparison_index(&self) -> usize {
    self.comparison_index
  }

  pub fn is_running(&self) -> bool {
    self.current_index < self.last_index
  }

  pub fn is_running_comparison(&self) -> bool {
    self.comparison_index <= self.last_index
  }
}

#[cfg(test)]
mod test {
  use std::sync::mpsc::{self, Sender};

  use comparison_index_store_builder::ComparisonIndexStoreBuilder;

  use super::*;

  struct BuilderWithChannels {
    builder: ComparisonIndexStoreBuilder,
    index_sender: Sender<usize>,
    status_receiver: Receiver<ComparisonStatus>,
  }

  #[test]
  fn can_create_new() {
    let (_, test_receiver) = mpsc::channel();
    let (test_sender, _) = mpsc::channel();
    ComparisonIndexStore::new(5, 0, test_sender, test_receiver);
  }

  #[test]
  fn can_get_indexes() {
    let index_store = create_builder_with_channels()
      .builder
      .last_index(5)
      .start_index(0)
      .build();

    assert_eq!(index_store.get_current_index(), 0);
    assert_eq!(index_store.get_comparison_index(), 1);
  }

  #[test]
  fn can_start_from_further_along() {
    let index_store = create_builder_with_channels()
      .builder
      .last_index(5)
      .start_index(2)
      .build();

    assert_eq!(index_store.get_current_index(), 2);
    assert_eq!(index_store.get_comparison_index(), 3);
  }

  #[test]
  fn can_increment_indexes() {
    let BuilderWithChannels {
      builder,
      index_sender,
      status_receiver,
    } = create_builder_with_channels();

    let mut index_store = builder.last_index(3).start_index(0).build();

    assert_indexes(&index_store, 0, 1);

    index_store.increment_indexes();
    assert_indexes(&index_store, 0, 2);

    index_store.increment_indexes();
    assert_indexes(&index_store, 0, 3);

    index_store.increment_indexes();
    assert_indexes(&index_store, 0, 4);

    index_sender.send(2).unwrap();
    index_store.increment_indexes();

    assert_eq!(
      status_receiver.recv().unwrap(),
      ComparisonStatus::NextIndex(0)
    );
    assert_indexes(&index_store, 2, 3);

    index_store.increment_indexes();
    assert_indexes(&index_store, 2, 4);

    index_sender.send(3).unwrap();
    index_store.increment_indexes();

    assert_eq!(
      status_receiver.recv().unwrap(),
      ComparisonStatus::NextIndex(0)
    );
    assert_indexes(&index_store, 3, 4);
  }

  fn assert_indexes(index_store: &ComparisonIndexStore, current: usize, cmp: usize) {
    assert_eq!(index_store.get_current_index(), current);
    assert_eq!(index_store.get_comparison_index(), cmp);
  }

  #[test]
  fn can_check_if_still_running_overall() {
    let BuilderWithChannels {
      builder,
      index_sender,
      status_receiver,
    } = create_builder_with_channels();

    let mut index_store = builder.last_index(2).start_index(0).build();

    assert!(index_store.is_running());

    index_store.increment_indexes();
    assert!(index_store.is_running());

    index_store.increment_indexes();
    assert!(index_store.is_running());

    index_sender.send(1).unwrap();
    index_store.increment_indexes();

    assert_eq!(
      status_receiver.recv().unwrap(),
      ComparisonStatus::NextIndex(0)
    );
    assert!(index_store.is_running());

    index_store.increment_indexes();
    assert!(index_store.is_running());

    index_sender.send(2).unwrap();
    index_store.increment_indexes();

    assert_eq!(
      status_receiver.recv().unwrap(),
      ComparisonStatus::NextIndex(0)
    );
    assert!(!index_store.is_running());
  }

  #[test]
  fn can_check_if_current_comparison_still_running() {
    let BuilderWithChannels {
      builder,
      index_sender,
      status_receiver,
    } = create_builder_with_channels();

    let mut index_store = builder.last_index(2).start_index(0).build();

    assert!(index_store.is_running_comparison());

    index_store.increment_indexes();
    assert!(index_store.is_running_comparison());

    index_store.increment_indexes();
    assert!(!index_store.is_running_comparison());

    index_sender.send(1).unwrap();
    index_store.increment_indexes();

    assert_eq!(
      status_receiver.recv().unwrap(),
      ComparisonStatus::NextIndex(0)
    );
    assert!(index_store.is_running_comparison());
  }

  #[test]
  fn does_not_run_comparison_if_starting_index_is_greater_than_or_equal_to_last_index() {
    let builder1 = create_builder_with_channels().builder;
    let builder2 = create_builder_with_channels().builder;
    let builder3 = create_builder_with_channels().builder;
    let index_store1 = builder1.last_index(0).start_index(1).build();
    let index_store2 = builder2.last_index(5).start_index(9).build();
    let index_store3 = builder3.last_index(4).start_index(4).build();

    assert!(!index_store1.is_running());
    assert!(!index_store1.is_running_comparison());

    assert!(!index_store2.is_running());
    assert!(!index_store2.is_running_comparison());

    assert!(!index_store3.is_running());
    assert!(!index_store3.is_running_comparison());
  }

  fn create_builder_with_channels() -> BuilderWithChannels {
    let (index_sender, index_receiver) = mpsc::channel();
    let (status_sender, status_receiver) = mpsc::channel();
    let builder = ComparisonIndexStoreBuilder::new()
      .last_index(0)
      .start_index(0)
      .status_sender(status_sender)
      .index_receiver(index_receiver);

    BuilderWithChannels {
      builder,
      index_sender,
      status_receiver,
    }
  }
}
