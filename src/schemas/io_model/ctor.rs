pub use macros::Ctor;

use crate::schemas::ChainInput;

mod sealed {
    pub trait Sealed {}
}

pub trait Ctor: Send + Sync + 'static {
    type Target<'a>: Send + Sync + 'a;
}

pub trait InputCtor: sealed::Sealed + Send + Sync + 'static {
    type Target<'a>: ChainInput;
}

pub trait OutputCtor: sealed::Sealed + Send + Sync + 'static {
    type Target<'a>: Send + Sync + 'a;
}

impl<T: Ctor> sealed::Sealed for T {}

impl<T> InputCtor for T
where
    T: Ctor + sealed::Sealed,
    for<'a> T::Target<'a>: ChainInput,
{
    type Target<'a> = T::Target<'a>;
}

impl<T> OutputCtor for T
where
    T: Ctor + sealed::Sealed,
    for<'a> T::Target<'a>: Send + Sync + 'a,
{
    type Target<'a> = T::Target<'a>;
}

pub struct StringCtor;

impl Ctor for StringCtor {
    type Target<'a> = String;
}

impl Ctor for () {
    type Target<'a> = ();
}
