//! Generic atomic
//!
//!# Features
//!
//!- `std` Enables `std` only traits;

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[cfg(any(test, feature = "std"))]
extern crate std;

use core::mem;
use core::cell::UnsafeCell;
pub use core::sync::atomic::Ordering;

mod ops;

#[repr(transparent)]
///Generic Atomic which allows any `T` to be used as lock-free atomic integer.
///
///This type allows only types whose size and alignment is compatible with `u8`, `u16`, `u32`, `u64`.
///
///With exception of `fetch_*` methods, all atomic methods are implemented for generic `T`
///
///`fetch_*` makes sense only to integers, hence they are implemented as specialized methods.
pub struct Atomic<T> {
    inner: UnsafeCell<T>
}

unsafe impl<T: Send> Sync for Atomic<T> {}
#[cfg(feature = "std")]
impl<T: Copy + std::panic::RefUnwindSafe> std::panic::RefUnwindSafe for Atomic<T> {}

impl<T: Default> Default for Atomic<T> {
    #[inline(always)]
    fn default() -> Self {
        Self::new(Default::default())
    }
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
        match Self::TYPE_SIZE {
            1 if mem::align_of::<T>() >= 1 => ops::u8::atomic_load,
            _ => unimplemented!(),
        }
    };
    const STORE: fn(*mut T, T, Ordering) = {
        match Self::TYPE_SIZE {
            1 if mem::align_of::<T>() >= 1 => ops::u8::atomic_store,
            _ => unimplemented!(),
        }
    };
    const SWAP: fn(*mut T, T, Ordering) -> T = {
        match Self::TYPE_SIZE {
            1 if mem::align_of::<T>() >= 1 => ops::u8::atomic_swap,
            _ => unimplemented!(),
        }
    };
    const CMP_EX: fn(*mut T, T, T, Ordering, Ordering) -> Result<T, T> = {
        match Self::TYPE_SIZE {
            1 if mem::align_of::<T>() >= 1 => ops::u8::atomic_compare_exchange,
            _ => unimplemented!(),
        }
    };
    const CMP_EX_WEAK: fn(*mut T, T, T, Ordering, Ordering) -> Result<T, T> = {
        match Self::TYPE_SIZE {
            1 if mem::align_of::<T>() >= 1 => ops::u8::atomic_compare_exchange_weak,
            _ => unimplemented!(),
        }
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

//TODO: add `fetch_*` methods
