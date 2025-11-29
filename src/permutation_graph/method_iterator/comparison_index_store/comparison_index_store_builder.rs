use std::sync::mpsc::{Receiver, Sender};

use crate::permutation_graph::method_iterator::{
  comparison_index_store::ComparisonIndexStore, comparison_status::ComparisonStatus,
};

pub struct ComparisonIndexStoreBuilder {
  last_index: Option<usize>,
  start_index: Option<usize>,
  status_sender: Option<Sender<ComparisonStatus>>,
  index_receiver: Option<Receiver<usize>>,
}

impl ComparisonIndexStoreBuilder {
  pub fn new() -> ComparisonIndexStoreBuilder {
    ComparisonIndexStoreBuilder {
      last_index: None,
      start_index: None,
      status_sender: None,
      index_receiver: None,
    }
  }

  pub fn last_index(mut self, index: usize) -> Self {
    self.last_index = Some(index);
    self
  }

  pub fn start_index(mut self, index: usize) -> Self {
    self.start_index = Some(index);
    self
  }

  pub fn status_sender(mut self, sender: Sender<ComparisonStatus>) -> Self {
    self.status_sender = Some(sender);
    self
  }

  pub fn index_receiver(mut self, receiver: Receiver<usize>) -> Self {
    self.index_receiver = Some(receiver);
    self
  }

  pub fn build(self) -> ComparisonIndexStore {
    ComparisonIndexStore::new(
      self.last_index.expect("Must provide field 'last_index'"),
      self.start_index.expect("Must provide field 'start_index'"),
      self
        .status_sender
        .expect("Must provide field 'status_sender'"),
      self
        .index_receiver
        .expect("Must provide field 'index_receiver'"),
    )
  }
}

#[cfg(test)]
mod test {
  use std::sync::mpsc;

  use super::*;

  #[test]
  fn can_create_new() {
    ComparisonIndexStoreBuilder::new();
  }

  #[test]
  fn can_build_a_comparison_index_store() {
    let (index_sender, index_receiver) = mpsc::channel();
    let (status_sender, status_receiver) = mpsc::channel();
    let builder = ComparisonIndexStoreBuilder::new();
    let test_last_index = 5;
    let test_start_index = 1;

    let index_store = builder
      .last_index(test_last_index)
      .start_index(test_start_index)
      .status_sender(status_sender)
      .index_receiver(index_receiver)
      .build();

    assert_eq!(index_store.last_index, test_last_index);
    assert_eq!(index_store.current_index, test_start_index);
    assert_eq!(index_store.comparison_index, test_start_index + 1);

    let sent_index = 8;
    index_sender.send(sent_index).unwrap();
    assert_eq!(index_store.index_receiver.recv().unwrap(), sent_index);

    let send_num = 4;
    index_store
      .status_sender
      .send(ComparisonStatus::NextIndex(send_num))
      .unwrap();
    assert_eq!(
      status_receiver.recv().unwrap(),
      ComparisonStatus::NextIndex(send_num)
    );
  }

  #[test]
  #[should_panic(expected = "Must provide field 'last_index'")]
  fn panics_if_last_index_not_filled() {
    let (_, index_receiver) = mpsc::channel();
    let (status_sender, _) = mpsc::channel();
    let builder = ComparisonIndexStoreBuilder::new();

    builder
      .start_index(1)
      .status_sender(status_sender)
      .index_receiver(index_receiver)
      .build();
  }

  #[test]
  #[should_panic(expected = "Must provide field 'start_index'")]
  fn panics_if_start_index_not_filled() {
    let (_, index_receiver) = mpsc::channel();
    let (status_sender, _) = mpsc::channel();
    let builder = ComparisonIndexStoreBuilder::new();

    builder
      .last_index(5)
      .status_sender(status_sender)
      .index_receiver(index_receiver)
      .build();
  }

  #[test]
  #[should_panic(expected = "Must provide field 'status_sender'")]
  fn panics_if_status_sender_not_filled() {
    let (_, index_receiver) = mpsc::channel();
    let builder = ComparisonIndexStoreBuilder::new();

    builder
      .last_index(5)
      .start_index(2)
      .index_receiver(index_receiver)
      .build();
  }

  #[test]
  #[should_panic(expected = "Must provide field 'index_receiver'")]
  fn panics_if_index_receiver_not_filled() {
    let (status_sender, _) = mpsc::channel();
    let builder = ComparisonIndexStoreBuilder::new();

    builder
      .last_index(5)
      .start_index(2)
      .status_sender(status_sender)
      .build();
  }
}
