use atomik::{Atomic, Ordering};

#[test]
fn should_check_methods_work() {
    static NUM: Atomic::<u8> = Atomic::new(0);

    assert_eq!(NUM.load(Ordering::Relaxed), 0);
    NUM.store(1, Ordering::Relaxed);
    assert_eq!(NUM.load(Ordering::Relaxed), 1);
    assert_eq!(NUM.swap(5, Ordering::Relaxed), 1);
    assert_eq!(NUM.load(Ordering::Relaxed), 5);

    assert_eq!(NUM.compare_exchange(5, 10, Ordering::Acquire, Ordering::Relaxed), Ok(5));
    assert_eq!(NUM.load(Ordering::Relaxed), 10);
    assert_eq!(NUM.compare_exchange(9, 20, Ordering::Acquire, Ordering::Relaxed), Err(10));
    assert_eq!(NUM.compare_exchange(10, 20, Ordering::Acquire, Ordering::Relaxed), Ok(10));
    assert_eq!(NUM.load(Ordering::Relaxed), 20);
    assert_eq!(NUM.compare_exchange(5, 10, Ordering::Acquire, Ordering::Relaxed), Err(20));
}
