#[cfg(test)]
mod test {
  use permutations::Permutations;

  #[test]
  fn produces_all_permutations() {
    let perms = Permutations::new(3);
    let base_perm_vec = vec![0, 1, 2];
    let all_perms = [
      [0, 1, 2],
      [0, 2, 1],
      [1, 0, 2],
      [1, 2, 0],
      [2, 0, 1],
      [2, 1, 0],
    ];
    let mut checked_perms = Vec::<[i32; 3]>::new();

    for &perm in all_perms.iter() {
      let is_match = perms
        .iter()
        .any(|inner| inner.permute(&base_perm_vec).eq(&perm));

      assert!(is_match);

      if is_match {
        assert!(!checked_perms.contains(&perm));
        checked_perms.push(perm);
      }
    }
  }

  #[test]
  fn implements_equals_on_each_permutation() {
    let perms = Permutations::new(3);

    for perm in perms {
      assert_eq!(perm, perm);
    }
  }
}
