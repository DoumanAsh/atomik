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
    (INT $SIZE:expr => $fn:ident on $T:ident) => {
        match $SIZE {
            #[cfg(target_has_atomic = "8")]
            1 if mem::align_of::<$T>() >= mem::align_of::<$T>() => ops::$T::$fn,
            #[cfg(target_has_atomic = "16")]
            2 if mem::align_of::<$T>() >= mem::align_of::<$T>() => ops::$T::$fn,
            #[cfg(target_has_atomic = "32")]
            4 if mem::align_of::<$T>() >= mem::align_of::<$T>() => ops::$T::$fn,
            #[cfg(target_has_atomic = "64")]
            8 if mem::align_of::<$T>() >= mem::align_of::<$T>() => ops::$T::$fn,
            _ => unimplemented!(),
        }
    };
    (::math => $SIZE:expr => $fn:ident on $T:ident) => {
        match $SIZE {
            #[cfg(target_has_atomic = "8")]
            1 if mem::align_of::<$T>() >= mem::align_of::<$T>() => ops::math::$T::$fn,
            #[cfg(target_has_atomic = "16")]
            2 if mem::align_of::<$T>() >= mem::align_of::<$T>() => ops::math::$T::$fn,
            #[cfg(target_has_atomic = "32")]
            4 if mem::align_of::<$T>() >= mem::align_of::<$T>() => ops::math::$T::$fn,
            #[cfg(target_has_atomic = "64")]
            8 if mem::align_of::<$T>() >= mem::align_of::<$T>() => ops::math::$T::$fn,
            _ => unimplemented!(),
        }
    };
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
}

macro_rules! impl_common_ops {
    ($($ty:ident),*) => ($(
        impl Atomic<$ty> {
            const FETCH_AND: fn(*mut $ty, $ty, Ordering) -> $ty = {
                match_size_arm!(INT Self::TYPE_SIZE => atomic_fetch_and on $ty)
            };
            const FETCH_NAND: fn(*mut $ty, $ty, Ordering) -> $ty = {
                match_size_arm!(INT Self::TYPE_SIZE => atomic_fetch_nand on $ty)
            };
            const FETCH_OR: fn(*mut $ty, $ty, Ordering) -> $ty = {
                match_size_arm!(INT Self::TYPE_SIZE => atomic_fetch_or on $ty)
            };
            const FETCH_XOR: fn(*mut $ty, $ty, Ordering) -> $ty = {
                match_size_arm!(INT Self::TYPE_SIZE => atomic_fetch_xor on $ty)
            };

            /// Add to the current value, returning the previous value.

            /// Bitwise and with the current value, returning the previous value.
            #[inline]
            pub fn fetch_and(&self, val: $ty, order: Ordering) -> $ty {
                Self::FETCH_AND(self.inner_ptr(), val, order)
            }
            /// Bitwise nand with the current value.
            #[inline]
            pub fn fetch_nand(&self, val: $ty, order: Ordering) -> $ty {
                Self::FETCH_NAND(self.inner_ptr(), val, order)
            }

            /// Bitwise or with the current value, returning the previous value.
            #[inline]
            pub fn fetch_or(&self, val: $ty, order: Ordering) -> $ty {
                Self::FETCH_OR(self.inner_ptr(), val, order)
            }

            /// Bitwise xor with the current value, returning the previous value.
            #[inline]
            pub fn fetch_xor(&self, val: $ty, order: Ordering) -> $ty {
                Self::FETCH_XOR(self.inner_ptr(), val, order)
            }
        }
    )*);
}

macro_rules! impl_math_ops {
    ($($ty:ident),*) => ($(
        impl Atomic<$ty> {
            const FETCH_ADD: fn(*mut $ty, $ty, Ordering) -> $ty = {
                match_size_arm!(::math => Self::TYPE_SIZE => atomic_fetch_add on $ty)
            };
            const FETCH_SUB: fn(*mut $ty, $ty, Ordering) -> $ty = {
                match_size_arm!(::math => Self::TYPE_SIZE => atomic_fetch_sub on $ty)
            };
            const FETCH_MIN: fn(*mut $ty, $ty, Ordering) -> $ty = {
                match_size_arm!(::math => Self::TYPE_SIZE => atomic_fetch_min on $ty)
            };
            const FETCH_MAX: fn(*mut $ty, $ty, Ordering) -> $ty = {
                match_size_arm!(::math => Self::TYPE_SIZE => atomic_fetch_max on $ty)
            };

            #[inline]
            /// Minimum with the current value.
            pub fn fetch_min(&self, val: $ty, order: Ordering) -> $ty {
                Self::FETCH_MIN(self.inner_ptr(), val, order)
            }

            #[inline]
            /// Maximum with the current value.
            pub fn fetch_max(&self, val: $ty, order: Ordering) -> $ty {
                Self::FETCH_MAX(self.inner_ptr(), val, order)
            }

            #[inline]
            /// Adds to the current value, returning the previous value.
            pub fn fetch_add(&self, val: $ty, order: Ordering) -> $ty {
                Self::FETCH_ADD(self.inner_ptr(), val, order)
            }

            /// Subtract from the current value, returning the previous value.
            #[inline]
            pub fn fetch_sub(&self, val: $ty, order: Ordering) -> $ty {
                Self::FETCH_SUB(self.inner_ptr(), val, order)
            }
        }
    )*);
}

impl_common_ops!(bool, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
impl_math_ops!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);
