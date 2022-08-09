#![allow(clippy::unused_self)]

use core::{marker::{PhantomData, PhantomPinned}, ops::Deref};

pub struct Markers<const SEND: bool, const SYNC: bool, const UNPIN: bool>(
    PhantomPinned,
    PhantomData<*const ()>,
);

impl<const SEND: bool, const SYNC: bool, const UNPIN: bool> Markers<SEND, SYNC, UNPIN> {
    pub const fn new() -> Self {
        Self(PhantomPinned, PhantomData)
    }
}

#[allow(clippy::undocumented_unsafe_blocks)]
const _: () = {
    unsafe impl<const SYNC: bool, const UNPIN: bool> Send  for Markers<true, SYNC, UNPIN> {}
    unsafe impl<const SEND: bool, const UNPIN: bool> Sync  for Markers<SEND, true, UNPIN> {}
    impl       <const SEND: bool, const SYNC:  bool> Unpin for Markers<SEND, SYNC, true>  {}
};

pub struct False; pub struct True;
pub trait Bool      { const VALUE: bool;         }
impl Bool for False { const VALUE: bool = false; }
impl Bool for True  { const VALUE: bool = true;  }

pub const fn to_bool<T: Bool, F: FnOnce() -> T>(_: &F) -> bool { T::VALUE }

pub struct Checker<T>(PhantomData<T>);
impl<T> Checker<T> {
    pub const fn new(_: &T) -> Self { Self(PhantomData) }
}
impl<T: Send>  Checker<T> { pub fn check_send(self: &&Self)  -> True { True } }
impl<T: Sync>  Checker<T> { pub fn check_sync(self: &&Self)  -> True { True } }
impl<T: Unpin> Checker<T> { pub fn check_unpin(self: &&Self) -> True { True } }

pub struct CheckerFalse;
impl CheckerFalse {
    pub fn check_send(&self)  -> False { False }
    pub fn check_sync(&self)  -> False { False }
    pub fn check_unpin(&self) -> False { False }
}

impl<T> Deref for Checker<T> {
    type Target = CheckerFalse;

    fn deref(&self) -> &CheckerFalse { &CheckerFalse }
}

#[doc(hidden)]
#[macro_export]
macro_rules! markers {
    ($func:expr) => {
        $crate::markers::Markers<
            { $crate::markers::to_bool(&|| (&$crate::markers::Checker::new(&$func)).check_send())  },
            { $crate::markers::to_bool(&|| (&$crate::markers::Checker::new(&$func)).check_sync())  },
            { $crate::markers::to_bool(&|| (&$crate::markers::Checker::new(&$func)).check_unpin()) },
        >
    };
}
