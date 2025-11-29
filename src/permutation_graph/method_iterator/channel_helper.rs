use std::sync::mpsc::{Receiver, Sender};

pub fn send_or_error<T>(sender: &Sender<T>, msg: T) {
  sender.send(msg).unwrap_or_else(|error| {
    panic!("{}", error.to_string());
  });
}

pub fn recv_or_error<T>(receiver: &Receiver<T>) -> T {
  receiver.recv().unwrap_or_else(|error| {
    panic!("{}", error.to_string());
  })
}

#[cfg(test)]
mod test {
  use std::sync::mpsc::channel;

  use super::*;

  #[test]
  fn can_send_a_message() {
    let msg = 5;
    let (sender, receiver) = channel();
    send_or_error(&sender, msg);

    assert_eq!(receiver.recv().unwrap(), msg);
  }

  #[test]
  #[should_panic(expected = "sending on a closed channel")]
  fn sending_panics_with_the_error_message() {
    let (sender, receiver) = channel();
    drop(receiver);

    send_or_error(&sender, 8);
  }

  #[test]
  fn can_receive_a_message() {
    let msg = 7;
    let (sender, receiver) = channel();

    sender.send(msg).unwrap();
    assert_eq!(recv_or_error(&receiver), msg);
  }

  #[test]
  #[should_panic(expected = "receiving on a closed channel")]
  fn receiving_panics_with_the_error_message() {
    let (sender, receiver) = channel::<i32>();
    drop(sender);

    recv_or_error(&receiver);
  }
}
