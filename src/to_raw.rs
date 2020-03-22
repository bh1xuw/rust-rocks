//! this is for information hiding

pub trait ToRaw<T> {
    fn raw(&self) -> *mut T;
}

pub trait FromRaw<T> {
    unsafe fn from_ll(_: *mut T) -> Self;
}
