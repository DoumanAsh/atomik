//! Generic atomic
//!
//!This Atomic allows only types whose size and alignment is compatible with `u8`, `u16`, `u32`, `u64`.
//!
//!With exception of `fetch_*` methods, all atomic methods are implemented for generic `T`
//!
//!`fetch_*` makes sense only to integers, hence they are implemented as specialized methods.

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

use core::mem;
use core::cell::UnsafeCell;
use core::sync::atomic;
pub use core::sync::atomic::Ordering;

mod ops;

#[repr(transparent)]
///Generic atomic which allows any `T` to be used as lock-free atomic integer.
///
///This atomic allows only types whose size and alignment is compatible with `u8`, `u16`, `u32`, `u64`.
///
///With exception of `fetch_*` methods, all atomic methods are implemented for generic `T`
///
///`fetch_*` makes sense only to integers, hence they are implemented as specialized methods.
pub struct Atomic<T> {
    inner: UnsafeCell<T>
}

unsafe impl<T: Send> Sync for Atomic<T> {}
impl<T: Copy + core::panic::RefUnwindSafe> core::panic::RefUnwindSafe for Atomic<T> {}

impl<T: Default> Default for Atomic<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

macro_rules! match_size_arm {
    ($SIZE:expr => $fn:ident on $T:ident) => {
        match $SIZE {
            #[cfg(target_has_atomic = "8")]
            1 if mem::align_of::<$T>() >= mem::align_of::<u8>() => ops::u8::$fn,
            #[cfg(target_has_atomic = "16")]
            2 if mem::align_of::<$T>() >= mem::align_of::<u16>() => ops::u16::$fn,
            #[cfg(target_has_atomic = "32")]
            4 if mem::align_of::<$T>() >= mem::align_of::<u32>() => ops::u32::$fn,
            #[cfg(target_has_atomic = "64")]
            8 if mem::align_of::<$T>() >= mem::align_of::<u64>() => ops::u64::$fn,
            _ => unimplemented!(),
        }
    };
}

impl<T> Atomic<T> {
    //For this to affect compilation, this constant must be used
    //hence we slap assert in `new` which is only way to legit create atomic;
    const TYPE_SIZE: usize = {
        let size = mem::size_of::<T>();
        assert!(size > 0);
        size
    };

    const LOAD: fn(*mut T, Ordering) -> T = {
        match_size_arm!(Self::TYPE_SIZE => atomic_load on T)
    };
    const STORE: fn(*mut T, T, Ordering) = {
        match_size_arm!(Self::TYPE_SIZE => atomic_store on T)
    };
    const SWAP: fn(*mut T, T, Ordering) -> T = {
        match_size_arm!(Self::TYPE_SIZE => atomic_swap on T)
    };
    const CMP_EX: fn(*mut T, T, T, Ordering, Ordering) -> Result<T, T> = {
        match_size_arm!(Self::TYPE_SIZE => atomic_compare_exchange on T)
    };
    const CMP_EX_WEAK: fn(*mut T, T, T, Ordering, Ordering) -> Result<T, T> = {
        match_size_arm!(Self::TYPE_SIZE => atomic_compare_exchange_weak on T)
    };

    #[inline]
    ///Creates a new instance
    pub const fn new(value: T) -> Atomic<T> {
        debug_assert!(Self::TYPE_SIZE <= mem::size_of::<u64>());

        Atomic {
            inner: UnsafeCell::new(value),
        }
    }
}

impl<T: Copy> Atomic<T> {
    #[inline]
    fn inner_ptr(&self) -> *mut T {
        self.inner.get() as *mut T
    }

    ///Returns a mutable reference to the underlying type.
    ///
    ///This is safe because the mutable reference guarantees that no other threads are concurrently accessing the atomic data.
    #[inline]
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.inner_ptr() }
    }

    ///Consumes the atomic and returns the contained value.
    ///
    ///This is safe because passing `self` by value guarantees that no other threads are concurrently accessing the atomic data.
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner.into_inner()
    }

    ///Loads a value from the atomic integer.
    ///
    ///load takes an Ordering argument which describes the memory ordering of this operation.
    ///Possible values are SeqCst, Acquire and Relaxed.
    ///
    ///## Panics
    ///
    ///Panics if `order` is `Release` or `AcqRel`.
    #[inline]
    pub fn load(&self, order: Ordering) -> T {
        Self::LOAD(self.inner_ptr(), order)
    }

    ///Stores a value into the atomic integer.
    ///
    ///store takes an Ordering argument which describes the memory ordering of this operation.
    ///Possible values are `SeqCst`, `Release` and `Relaxed`.
    ///
    ///## Panics
    ///
    ///Panics if `order` is `Acquire` or `AcqRel`.
    pub fn store(&self, val: T, order: Ordering) {
        Self::STORE(self.inner_ptr(), val, order)
    }

    #[inline]
    ///Stores a value into the atomic integer, returning the previous value.
    ///
    ///`swap` takes an `Ordering` argument which describes the memory ordering of this operation.
    ///All ordering modes are possible.
    ///Note that using `Acquire` makes the store part of this operation `Relaxed`, and using `Release` makes the load part `Relaxed`.
    pub fn swap(&self, val: T, order: Ordering) -> T {
        Self::SWAP(self.inner_ptr(), val, order)
    }

    ///Stores a value into the atomic if the current value is the same as the `current` value.
    ///
    ///The return value is a result indicating whether the new value was written and containing the previous value. On success this value is guaranteed to be equal to current.
    ///
    ///`compare_exchange` takes two `Ordering` arguments to describe the memory ordering of this operation. `success` describes the required ordering for the read-modify-write operation that takes place if the comparison with `current` succeeds. `failure` describes the required ordering for the load operation that takes place when the comparison fails. Using `Acquire` as success ordering makes the store part of this operation `Relaxed`, and using `Release` makes the successful load `Relaxed`. The `failure` ordering can only be `SeqCst`, `Acquire` or `Relaxed`.
    #[inline]
    pub fn compare_exchange(&self, current: T, new: T, success: Ordering, failure: Ordering) -> Result<T, T> {
        Self::CMP_EX(self.inner_ptr(), current, new, success, failure)
    }

    #[inline]
    ///Stores a value into the atomic if the current value is the same as the `current` value.
    ///
    ///Unlike `compare_exchange`, this function is allowed to spuriously fail even when the comparison succeeds, which can result in more efficient code on some platforms. The return value is a result indicating whether the new value was written and containing the previous value.
    ///
    ///`compare_exchange_weak` takes two `Ordering` arguments to describe the memory ordering of this operation. `success` describes the required ordering for the read-modify-write operation that takes place if the comparison with `current` succeeds. `failure` describes the required ordering for the load operation that takes place when the comparison fails. Using `Acquire` as success ordering makes the store part of this operation `Relaxed`, and using `Release` makes the successful load `Relaxed`. The failure ordering can only be `SeqCst`, `Acquire` or `Relaxed`.
    pub fn compare_exchange_weak(&self, current: T, new: T, success: Ordering, failure: Ordering) -> Result<T, T> {
        Self::CMP_EX_WEAK(self.inner_ptr(), current, new, success, failure)
    }

    #[inline]

    ///Fetches the value, and applies a function to it that returns an optional new value.
    ///Returns a `Result` of `Ok(previous_value)` if the function returned `Some(_)`, else `Err(previous_value)`.
    ///
    ///Note: This may call the function multiple times if the value has been changed from other threads in the meantime, as long as the function returns Some(_), but the function will have been applied only once to the stored value.
    ///
    ///`fetch_update` takes two `Ordering` arguments to describe the memory ordering of this operation.
    ///The first describes the required ordering for when the operation finally succeeds while the second describes the required ordering for loads.
    ///These correspond to the success and failure orderings of `compare_exchange` respectively.
    ///
    ///Using `Acquire` as success ordering makes the store part of this operation `Relaxed`, and using `Release` makes the final successful load `Relaxed`.
    ///The (failed) load ordering can only be `SeqCst`, `Acquire` or `Relaxed`.
    pub fn fetch_update<F: FnMut(T) -> Option<T>>(&self, set_order: Ordering, fetch_order: Ordering, mut cb: F) -> Result<T, T> {
        let mut prev = self.load(fetch_order);
        while let Some(next) = cb(prev) {
            match self.compare_exchange_weak(prev, next, set_order, fetch_order) {
                res @ Ok(_) => return res,
                Err(next_prev) => prev = next_prev,
            }
        }
        Err(prev)
    }
}

macro_rules! impl_common_spec {
    ($($ty:ident($atomic:ident)),*) => {$(
        impl Atomic<$ty> {
            /// Bitwise and with the current value, returning the previous value.
            #[inline]
            pub fn fetch_and(&self, val: $ty, order: Ordering) -> $ty {
                unsafe {
                    (*(self.inner_ptr() as *const atomic::$atomic)).fetch_and(val, order)
                }
            }
            /// Bitwise nand with the current value.
            #[inline]
            pub fn fetch_nand(&self, val: $ty, order: Ordering) -> $ty {
                unsafe {
                    (*(self.inner_ptr() as *const atomic::$atomic)).fetch_nand(val, order)
                }
            }

            /// Bitwise or with the current value, returning the previous value.
            #[inline]
            pub fn fetch_or(&self, val: $ty, order: Ordering) -> $ty {
                unsafe {
                    (*(self.inner_ptr() as *const atomic::$atomic)).fetch_or(val, order)
                }
            }

            /// Bitwise xor with the current value, returning the previous value.
            #[inline]
            pub fn fetch_xor(&self, val: $ty, order: Ordering) -> $ty {
                unsafe {
                    (*(self.inner_ptr() as *const atomic::$atomic)).fetch_xor(val, order)
                }
            }
        }
    )*};
}

macro_rules! impl_math_spec {
    ($($ty:ident($atomic:ident)),*) => {$(
        impl Atomic<$ty> {
            #[inline]
            /// Minimum with the current value.
            pub fn fetch_min(&self, val: $ty, order: Ordering) -> $ty {
                unsafe {
                    (*(self.inner_ptr() as *const atomic::$atomic)).fetch_min(val, order)
                }
            }

            #[inline]
            /// Maximum with the current value.
            pub fn fetch_max(&self, val: $ty, order: Ordering) -> $ty {
                unsafe {
                    (*(self.inner_ptr() as *const atomic::$atomic)).fetch_max(val, order)
                }
            }

            #[inline]
            /// Adds to the current value, returning the previous value.
            pub fn fetch_add(&self, val: $ty, order: Ordering) -> $ty {
                unsafe {
                    (*(self.inner_ptr() as *const atomic::$atomic)).fetch_add(val, order)
                }
            }

            /// Subtract from the current value, returning the previous value.
            #[inline]
            pub fn fetch_sub(&self, val: $ty, order: Ordering) -> $ty {
                unsafe {
                    (*(self.inner_ptr() as *const atomic::$atomic)).fetch_sub(val, order)
                }
            }
        }
    )*};
}

#[cfg(target_has_atomic = "8")]
impl_common_spec!(i8(AtomicI8), u8(AtomicU8), bool(AtomicBool));
#[cfg(target_has_atomic = "16")]
impl_common_spec!(i16(AtomicI16), u16(AtomicU16));
#[cfg(target_has_atomic = "32")]
impl_common_spec!(i32(AtomicI32), u32(AtomicU32));
#[cfg(target_has_atomic = "64")]
impl_common_spec!(i64(AtomicI64), u64(AtomicU64));

#[cfg(all(target_has_atomic = "64", target_pointer_width = "64"))]
impl_common_spec!(isize(AtomicIsize), usize(AtomicUsize));
#[cfg(all(target_has_atomic = "32", target_pointer_width = "32"))]
impl_common_spec!(isize(AtomicIsize), usize(AtomicUsize));
#[cfg(all(target_has_atomic = "16", target_pointer_width = "16"))]
impl_common_spec!(isize(AtomicIsize), usize(AtomicUsize));
#[cfg(all(target_has_atomic = "8", target_pointer_width = "8"))]
impl_common_spec!(isize(AtomicIsize), usize(AtomicUsize));

#[cfg(target_has_atomic = "8")]
impl_math_spec!(i8(AtomicI8), u8(AtomicU8));
#[cfg(target_has_atomic = "16")]
impl_math_spec!(i16(AtomicI16), u16(AtomicU16));
#[cfg(target_has_atomic = "32")]
impl_math_spec!(i32(AtomicI32), u32(AtomicU32));
#[cfg(target_has_atomic = "64")]
impl_math_spec!(i64(AtomicI64), u64(AtomicU64));

#[cfg(all(target_has_atomic = "64", target_pointer_width = "64"))]
impl_math_spec!(isize(AtomicIsize), usize(AtomicUsize));
#[cfg(all(target_has_atomic = "32", target_pointer_width = "32"))]
impl_math_spec!(isize(AtomicIsize), usize(AtomicUsize));
#[cfg(all(target_has_atomic = "16", target_pointer_width = "16"))]
impl_math_spec!(isize(AtomicIsize), usize(AtomicUsize));
#[cfg(all(target_has_atomic = "8", target_pointer_width = "8"))]
impl_math_spec!(isize(AtomicIsize), usize(AtomicUsize));
