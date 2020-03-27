//! This is for information hiding.

pub(crate) trait ToRaw<T> {
    fn raw(&self) -> *mut T;
}

pub(crate) trait FromRaw<T> {
    unsafe fn from_ll(_: *mut T) -> Self;
}
