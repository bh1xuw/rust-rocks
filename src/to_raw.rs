//! this is for information hiding


pub trait ToRaw<T> {
    fn raw(&self) -> *mut T;
}

pub trait FromRaw<T> {
    fn from_ll(*mut T) -> Self;
}
