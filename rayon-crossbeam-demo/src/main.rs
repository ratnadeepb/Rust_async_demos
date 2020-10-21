// use crossbeam;
use rayon;

fn main() {
    let arr = &[1, 25, -4, 10];
    let max = find_max(arr);
    assert_eq!(max, Some(25));
}

fn find_max(arr: &[i32]) -> Option<i32> {
    const THRESHOLD: usize = 2;

    if arr.len() < THRESHOLD {
        return arr.iter().cloned().max();
    }

    let mid = arr.len() / 2;
    let (left, right) = arr.split_at(mid);

    // scope is more flexible than join since tasks can be created in a loop without recursing
    // however, the scope tasks are allocated on the heap whereas join can use stack memory

    // rayon threads
    // let (max_l, max_r) = rayon::join(|| find_max(left), || find_max(right));
    // max_l.max(max_r);

    // rayon scope
    let mut max_l: Option<i32> = None;
    let mut max_r: Option<i32> = None;
    rayon::scope(|s| {
        s.spawn(|_| max_l = find_max(left));
        s.spawn(|_| max_r = find_max(right));
    });
    max_l.max(max_r)

    // crossbeam scope
    // crossbeam::scope(|s| {
    //     let thread1 = s.spawn(|_| find_max(left));
    //     let thread2 = s.spawn(|_| find_max(right));

    //     let max_l = thread1.join().unwrap()?;
    //     let max_r = thread2.join().unwrap()?;

    //     Some(max_l.max(max_r))
    // })
    // .unwrap()
}
