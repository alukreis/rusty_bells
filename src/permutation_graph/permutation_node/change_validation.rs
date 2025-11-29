pub fn is_valid_change(before: &[u8], after: &[u8]) -> bool {
  if before.len() != after.len() {
    panic!("changes must be the same length");
  }

  before.iter().enumerate().all(|(index, val)| {
    get_neighbouring_values_slice(index, after)
      .iter()
      .any(|comp_val| *comp_val == *val)
  })
}

fn get_neighbouring_values_slice(index: usize, change: &[u8]) -> &[u8] {
  let change_len = change.len();
  match index {
    0 if change_len == 1 => change,
    0 => &change[..index + 2],
    index if index + 1 == change_len => &change[index - 1..],
    _ => &change[index - 1..index + 2],
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn can_tell_a_valid_change() {
    let first = [0, 1, 2];
    let second = [0, 2, 1];
    let third = [2, 0, 1];
    let fourth = [2, 1, 0];
    assert!(is_valid_change(&first, &second));
    assert!(is_valid_change(&second, &third));
    assert!(is_valid_change(&third, &fourth));
  }

  #[test]
  fn can_tell_an_invalid_change() {
    let first = [0, 1, 2];
    let second = [0, 2, 1];
    let third = [2, 0, 1];
    let fourth = [2, 1, 0];
    assert!(!is_valid_change(&first, &third));
    assert!(!is_valid_change(&second, &fourth));
    assert!(!is_valid_change(&first, &fourth));
  }

  #[test]
  #[should_panic(expected = "changes must be the same length")]
  fn panics_if_mismatched_lengths() {
    is_valid_change(&[0, 1, 2], &[0, 2, 1, 3]);
  }

  #[test]
  fn handles_zero_length_changes() {
    assert!(is_valid_change(&[], &[]));
  }

  #[test]
  fn handles_length_one() {
    assert!(is_valid_change(&[0], &[0]));
  }

  #[test]
  fn handles_length_two() {
    assert!(is_valid_change(&[0, 1], &[0, 1]));
    assert!(is_valid_change(&[0, 1], &[1, 0]));
  }
}
