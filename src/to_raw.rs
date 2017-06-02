//! this is for information hiding


pub trait ToRaw<T> {
    #[inline]
    fn raw(&self) -> *mut T;
}

pub trait FromRaw<T> {
    #[inline]
    unsafe fn from_ll(*mut T) -> Self;
}
