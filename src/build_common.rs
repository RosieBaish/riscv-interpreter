pub fn tokenise(line: &str) -> Vec<String> {
  let splits: Vec<String> = line
    .trim()
    .split_inclusive(&[' ', '\t', ',', '(', ')'])
    .map(|s| s.to_string())
    .collect();
  let mut new_splits: Vec<String> = Vec::new();
  for split in splits {
    let mut new_split = split.clone();
    new_split = new_split.trim().to_string();
    if new_split.is_empty() {
      continue;
    }
    if new_split.ends_with(&[',', '(', ')']) {
      let len = new_split.len();
      let (first, last) = new_split.split_at(len - 1);
      new_splits.push(first.to_string());
      new_splits.push(last.to_string());
    } else {
      new_splits.push(new_split.to_string());
    }
  }
  new_splits
}
