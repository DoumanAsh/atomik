use core::mem::transmute_copy;

//Not sure if Result<FROM, FROM> has the same layout as Result<TO, TO> so be a loser and transmute each variant
unsafe fn map_result<FROM, TO>(val: Result<FROM, FROM>) -> Result<TO, TO> {
    match val {
        Ok(val) => Ok(transmute_copy(&val)),
        Err(val) => Err(transmute_copy(&val)),
    }
}

macro_rules! impl_atomic_base {
    ($($ty:ident($atomic:ident)),*) => {$(
        pub mod $ty {
            #[cfg(feature = "critical-section-polyfill")]
            use ::atomic_polyfill::$atomic;
            #[cfg(not(feature = "critical-section-polyfill"))]
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
        }
    )*};
}

#[cfg(any(feature = "critical-section-polyfill", target_has_atomic = "8"))]
impl_atomic_base!(bool(AtomicBool), u8(AtomicU8), i8(AtomicI8));

#[cfg(any(feature = "critical-section-polyfill", target_has_atomic = "16"))]
impl_atomic_base!(u16(AtomicU16), i16(AtomicI16));
#[cfg(any(feature = "critical-section-polyfill", target_has_atomic = "32"))]
impl_atomic_base!(u32(AtomicU32), i32(AtomicI32));
#[cfg(any(feature = "critical-section-polyfill", target_has_atomic = "64"))]
impl_atomic_base!(u64(AtomicU64), i64(AtomicI64));
