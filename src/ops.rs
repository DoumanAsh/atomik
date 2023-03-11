use core::mem::transmute_copy;

//Not sure if Result<FROM, FROM> has the same layout as Result<TO, TO> so be a loser and transmute each variant
unsafe fn map_result<FROM, TO>(val: Result<FROM, FROM>) -> Result<TO, TO> {
    match val {
        Ok(val) => Ok(transmute_copy(&val)),
        Err(val) => Err(transmute_copy(&val)),
    }
}

macro_rules! impl_atomic_load {
    ($ty:ident($atomic:ident)) => {
        pub mod $ty {
            use ::core::sync::atomic::$atomic;
            //transmute() doesn't work with generics, until it is fixed, use transmute_copy
            use ::core::mem::transmute_copy;

            pub fn atomic_load<T>(dst: *mut T, order: $crate::Ordering) -> T {
                unsafe {
                    transmute_copy(&(*(dst as *const $atomic)).load(order))
                }
            }

            pub fn atomic_store<T>(dst: *mut T, val: T, order: $crate::Ordering) {
                unsafe {
                    (*(dst as *const $atomic)).store(transmute_copy(&val), order)
                }
            }

            pub fn atomic_swap<T>(dst: *mut T, val: T, order: $crate::Ordering) -> T {
                unsafe {
                    transmute_copy(&(*(dst as *const $atomic)).swap(transmute_copy(&val), order))
                }
            }

            pub fn atomic_compare_exchange<T>(dst: *mut T, current: T, new: T, ok: $crate::Ordering, err: $crate::Ordering) -> Result<T, T> {
                unsafe {
                    $crate::ops::map_result::<$ty, T>(
                        transmute_copy(&(*(dst as *const $atomic)).compare_exchange(
                            transmute_copy(&current), transmute_copy(&new), ok, err)
                        )
                    )
                }
            }

            pub fn atomic_compare_exchange_weak<T>(dst: *mut T, current: T, new: T, ok: $crate::Ordering, err: $crate::Ordering) -> Result<T, T> {
                unsafe {
                    $crate::ops::map_result::<$ty, T>(
                        transmute_copy(&(*(dst as *const $atomic)).compare_exchange_weak(
                            transmute_copy(&current), transmute_copy(&new), ok, err)
                        )
                    )
                }
            }
        }
    };
}

impl_atomic_load!(u8(AtomicU8));
