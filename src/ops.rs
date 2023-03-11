use core::mem::transmute_copy;

//Not sure if Result<FROM, FROM> has the same layout as Result<TO, TO> so be a loser and transmute each variant
unsafe fn map_result<FROM, TO>(val: Result<FROM, FROM>) -> Result<TO, TO> {
    match val {
        Ok(val) => Ok(transmute_copy(&val)),
        Err(val) => Err(transmute_copy(&val)),
    }
}

macro_rules! impl_atomic_base {
    ($ty:ident($atomic:ident)) => {
        pub mod $ty {
            use ::core::sync::atomic::$atomic;
            //transmute() doesn't work with generics, until it is fixed, use transmute_copy
            use ::core::mem::transmute_copy;

            #[allow(unused)]
            pub fn atomic_load<T>(dst: *mut T, order: $crate::Ordering) -> T {
                unsafe {
                    transmute_copy(&(*(dst as *const $atomic)).load(order))
                }
            }

            #[allow(unused)]
            pub fn atomic_store<T>(dst: *mut T, val: T, order: $crate::Ordering) {
                unsafe {
                    (*(dst as *const $atomic)).store(transmute_copy(&val), order)
                }
            }

            #[allow(unused)]
            pub fn atomic_swap<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                unsafe {
                    transmute_copy(&(*(dst as *const $atomic)).swap(transmute_copy(&val), order))
                }
            }

            #[allow(unused)]
            pub fn atomic_compare_exchange<T>(dst: *mut T, current: T, new: T, ok: $crate::Ordering, err: $crate::Ordering) -> Result<T, T> {
                unsafe {
                    $crate::ops::map_result::<$ty, T>(
                        transmute_copy(&(*(dst as *const $atomic)).compare_exchange(
                            transmute_copy(&current), transmute_copy(&new), ok, err)
                        )
                    )
                }
            }

            #[allow(unused)]
            pub fn atomic_compare_exchange_weak<T>(dst: *mut T, current: T, new: T, ok: $crate::Ordering, err: $crate::Ordering) -> Result<T, T> {
                unsafe {
                    $crate::ops::map_result::<$ty, T>(
                        transmute_copy(&(*(dst as *const $atomic)).compare_exchange_weak(
                            transmute_copy(&current), transmute_copy(&new), ok, err)
                        )
                    )
                }
            }

            pub fn atomic_fetch_and<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                unsafe {
                    transmute_copy(&(*(dst as *const $atomic)).fetch_and(transmute_copy(&val), order))
                }
            }

            pub fn atomic_fetch_or<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                unsafe {
                    transmute_copy(&(*(dst as *const $atomic)).fetch_or(transmute_copy(&val), order))
                }
            }

            pub fn atomic_fetch_xor<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                unsafe {
                    transmute_copy(&(*(dst as *const $atomic)).fetch_xor(transmute_copy(&val), order))
                }
            }

            pub fn atomic_fetch_nand<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                unsafe {
                    transmute_copy(&(*(dst as *const $atomic)).fetch_nand(transmute_copy(&val), order))
                }
            }
        }
    };
}

macro_rules! impl_atomic_math {
    ($($ty:ident($atomic:ident)),*) => {
        pub mod math {$(
            pub mod $ty {
                use ::core::sync::atomic::$atomic;
                //transmute() doesn't work with generics, until it is fixed, use transmute_copy
                use ::core::mem::transmute_copy;

                pub fn atomic_fetch_add<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                    unsafe {
                        transmute_copy(&(*(dst as *const $atomic)).fetch_add(transmute_copy(&val), order))
                    }
                }

                pub fn atomic_fetch_sub<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                    unsafe {
                        transmute_copy(&(*(dst as *const $atomic)).fetch_sub(transmute_copy(&val), order))
                    }
                }

                pub fn atomic_fetch_min<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                    unsafe {
                        transmute_copy(&(*(dst as *const $atomic)).fetch_min(transmute_copy(&val), order))
                    }
                }

                pub fn atomic_fetch_max<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                    unsafe {
                        transmute_copy(&(*(dst as *const $atomic)).fetch_max(transmute_copy(&val), order))
                    }
                }
            }
        )*}
    }
}

impl_atomic_base!(bool(AtomicBool));

impl_atomic_base!(u8(AtomicU8));
impl_atomic_base!(u16(AtomicU16));
impl_atomic_base!(u32(AtomicU32));
impl_atomic_base!(u64(AtomicU64));
impl_atomic_base!(usize(AtomicUsize));

impl_atomic_base!(i8(AtomicI8));
impl_atomic_base!(i16(AtomicI16));
impl_atomic_base!(i32(AtomicI32));
impl_atomic_base!(i64(AtomicI64));
impl_atomic_base!(isize(AtomicIsize));

impl_atomic_math!(u8(AtomicU8), u16(AtomicU16), u32(AtomicU32), u64(AtomicU64), usize(AtomicUsize), i8(AtomicI8), i16(AtomicI16), i32(AtomicI32), i64(AtomicI64), isize(AtomicIsize));
