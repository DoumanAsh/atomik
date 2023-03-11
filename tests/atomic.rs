use atomik::{Atomic, Ordering};

macro_rules! impl_test_unsigned {
    ($ty:ident) => {
        static NUM: Atomic::<$ty> = Atomic::new(0);

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

        assert_eq!(NUM.fetch_sub(17, Ordering::Relaxed), 20);
        assert_eq!(NUM.fetch_add(7, Ordering::Relaxed), 3);
        assert_eq!(NUM.fetch_sub(10, Ordering::Relaxed), 10);
        assert_eq!(NUM.fetch_sub(1, Ordering::Relaxed), 0);
        assert_eq!(NUM.fetch_and(0, Ordering::Relaxed), $ty::max_value());

        assert_eq!(NUM.fetch_add(0b101101, Ordering::Relaxed), 0);
        assert_eq!(NUM.fetch_or(0b110011, Ordering::Relaxed), 0b101101);

        NUM.store(0b101101, Ordering::Relaxed);
        assert_eq!(NUM.fetch_xor(0b110011, Ordering::Relaxed), 0b101101);
        assert_eq!(NUM.load(Ordering::Relaxed), 0b011110);

        NUM.store(0x13, Ordering::Relaxed);
        assert_eq!(NUM.fetch_nand(0x31, Ordering::Relaxed), 0x13);
        assert_eq!(NUM.load(Ordering::Relaxed), !(0x13 & 0x31));

        NUM.store(0b101101, Ordering::Relaxed);
        assert_eq!(NUM.fetch_and(0b110011, Ordering::Relaxed), 0b101101);
        assert_eq!(NUM.load(Ordering::Relaxed), 0b100001);

        NUM.store(7, Ordering::Relaxed);
        assert_eq!(NUM.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| None), Err(7));
        assert_eq!(NUM.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(x + 1)), Ok(7));
        assert_eq!(NUM.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(x + 1)), Ok(8));
        assert_eq!(NUM.load(Ordering::SeqCst), 9);
    };
}

macro_rules! impl_test_signed {
    ($ty:ident) => {
        static NUM: Atomic::<$ty> = Atomic::new(0);

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

        assert_eq!(NUM.fetch_sub(17, Ordering::Relaxed), 20);
        assert_eq!(NUM.fetch_add(7, Ordering::Relaxed), 3);
        assert_eq!(NUM.fetch_sub(10, Ordering::Relaxed), 10);

        assert_eq!(NUM.fetch_add($ty::max_value(), Ordering::Relaxed), 0);
        assert_eq!(NUM.fetch_add(1, Ordering::Relaxed), $ty::max_value());
        assert_eq!(NUM.fetch_sub($ty::min_value(), Ordering::Relaxed), $ty::min_value());

        NUM.store(0b101101, Ordering::Relaxed);
        assert_eq!(NUM.fetch_or(0b110011, Ordering::Relaxed), 0b101101);

        NUM.store(0b101101, Ordering::Relaxed);
        assert_eq!(NUM.fetch_xor(0b110011, Ordering::Relaxed), 0b101101);
        assert_eq!(NUM.load(Ordering::Relaxed), 0b011110);

        NUM.store(0x13, Ordering::Relaxed);
        assert_eq!(NUM.fetch_nand(0x31, Ordering::Relaxed), 0x13);
        assert_eq!(NUM.load(Ordering::Relaxed), !(0x13 & 0x31));

        NUM.store(0b101101, Ordering::Relaxed);
        assert_eq!(NUM.fetch_and(0b110011, Ordering::Relaxed), 0b101101);
        assert_eq!(NUM.load(Ordering::Relaxed), 0b100001);

        NUM.store(7, Ordering::Relaxed);
        assert_eq!(NUM.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| None), Err(7));
        assert_eq!(NUM.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(x + 1)), Ok(7));
        assert_eq!(NUM.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| Some(x + 1)), Ok(8));
        assert_eq!(NUM.load(Ordering::SeqCst), 9);
    };
}

#[test]
fn should_check_methods_work_on_u8() {
    impl_test_unsigned!(u8);
}

#[test]
fn should_check_methods_work_on_u16() {
    impl_test_unsigned!(u16);
}

#[test]
fn should_check_methods_work_on_u32() {
    impl_test_unsigned!(u32);
}

#[test]
fn should_check_methods_work_on_u64() {
    impl_test_unsigned!(u64);
}

#[test]
fn should_check_methods_work_on_usize() {
    impl_test_unsigned!(usize);
}

#[test]
fn should_check_methods_work_on_i8() {
    impl_test_signed!(i8);
}

#[test]
fn should_check_methods_work_on_i16() {
    impl_test_signed!(i16);
}

#[test]
fn should_check_methods_work_on_i32() {
    impl_test_signed!(i32);
}

#[test]
fn should_check_methods_work_on_i64() {
    impl_test_signed!(i64);
}

#[test]
fn should_check_methods_work_on_isize() {
    impl_test_signed!(isize);
}
