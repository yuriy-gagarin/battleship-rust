// Take first item that satisfies the predicate
pub fn take_one<T, F: FnMut(&T) -> bool>(
  iter: &mut dyn Iterator<Item = T>,
  mut pred: F,
) -> Result<T, &str> {
  iter
    .skip_while(|m| !pred(m))
    .next()
    .ok_or("no items in iterator")
}
