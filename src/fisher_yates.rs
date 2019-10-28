
/// The in place Fisher-Yates shuffle.
/// Based on [Fisher–Yates shuffle].
///
// [Fisher–Yates shuffle]: https://en.wikipedia.org/wiki/Fisher–Yates_shuffle
///
/// # Examples
///
/// ```
/// use mohan::fisher_yates;
///
/// // Collect the numbers 0..5
/// let mut a = (0..5).collect::<Vec<_>>();
///
/// // Permute the values in place with Fisher-Yates
/// fisher_yates(&mut a);
/// ```
pub fn fisher_yates<T>(arr: &mut [T]) {
    use rand::Rng;
    let n = arr.len();
    let mut rng = crate::mohan_rand();

    for i in 0..n {
        // Swap i with a random point after it
        let j = rng.gen_range(0, n - i);
        arr.swap(i, i + j);
    }
}


#[test]
fn test_in_place_fisher_yates() {
    let mut a = (0..10).collect::<Vec<_>>();
    fisher_yates(&mut a);
    for val in 0..10 {
        assert!(a.contains(&val));
    }
}

#[test]
fn test_vector_shuffle() {
    let a = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut b = a.clone();
    fisher_yates(&mut b);
    assert!(a != b);
}
