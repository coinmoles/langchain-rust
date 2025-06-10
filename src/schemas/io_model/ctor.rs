pub use macros::Ctor;

use crate::schemas::ChainInput;

mod sealed {
    pub trait Sealed {}
}

pub trait Ctor: Send + Sync + 'static {
    type Target<'any>: Send + Sync + 'any;
}

pub trait InputCtor: sealed::Sealed + Send + Sync + 'static {
    type Target<'any>: ChainInput;
}

pub trait OutputCtor: sealed::Sealed + Send + Sync + 'static {
    type Target<'any>: Send + Sync + 'any;
}

impl<T: Ctor> sealed::Sealed for T {}

impl<T> InputCtor for T
where
    T: Ctor + sealed::Sealed,
    for<'any> T::Target<'any>: ChainInput,
{
    type Target<'any> = T::Target<'any>;
}

impl<T> OutputCtor for T
where
    T: Ctor + sealed::Sealed,
    for<'any> T::Target<'any>: Send + Sync + 'any,
{
    type Target<'any> = T::Target<'any>;
}

pub struct StringCtor;

impl Ctor for StringCtor {
    type Target<'any> = String;
}
