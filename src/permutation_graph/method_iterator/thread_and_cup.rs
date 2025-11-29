use std::{
  sync::mpsc::Sender,
  thread::{JoinHandle, Result},
};

use crate::permutation_graph::method_iterator::channel_helper::send_or_error;

pub struct ThreadAndCup {
  cup: Sender<usize>,
  thread: JoinHandle<()>,
}

impl ThreadAndCup {
  pub fn new(thread: JoinHandle<()>, cup: Sender<usize>) -> ThreadAndCup {
    ThreadAndCup { cup, thread }
  }

  pub fn join_thread(self) -> Result<()> {
    self.thread.join()
  }

  pub fn send_to_thread(&self, value: usize) {
    send_or_error(&self.cup, value);
  }

  pub fn is_thread_running(&self) -> bool {
    !self.thread.is_finished()
  }
}

#[cfg(test)]
mod test {
  use std::{
    sync::mpsc::channel,
    thread::{self, sleep},
    time::Duration,
  };

  use super::*;

  #[test]
  fn can_create_new() {
    let (sender, _) = channel();
    let thread = thread::spawn(|| ());
    let thread_and_cup = ThreadAndCup::new(thread, sender);

    thread_and_cup.thread.join().unwrap();
  }

  #[test]
  fn can_join_thread() {
    let (sender, _) = channel();
    let thread = thread::spawn(|| ());
    let thread_and_cup = ThreadAndCup::new(thread, sender);

    // It's a bad test, but because everything gets moved out after
    // this, we need to just rely on the compiler and other tests
    thread_and_cup.join_thread().unwrap();
  }

  #[test]
  fn can_communicate_with_the_thread() {
    let (input_sender, input_receiver) = channel();
    let (test_sender, test_receiver) = channel();
    let value = 42;
    let thread = thread::spawn(move || {
      test_sender.send(input_receiver.recv().unwrap()).unwrap();
    });
    let thread_and_cup = ThreadAndCup::new(thread, input_sender);

    thread_and_cup.send_to_thread(value);

    assert_eq!(test_receiver.recv().unwrap(), value);
    thread_and_cup.join_thread().unwrap();
  }

  #[test]
  fn can_check_if_thread_is_still_running() {
    let (sender, _) = channel();
    let thread = thread::spawn(|| {
      sleep(Duration::from_millis(300));
    });
    let thread_and_cup = ThreadAndCup::new(thread, sender);

    assert!(thread_and_cup.is_thread_running());

    thread_and_cup.join_thread().unwrap();
  }
}
