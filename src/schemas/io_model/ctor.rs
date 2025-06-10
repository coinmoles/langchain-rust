pub use macros::Ctor;

use crate::schemas::ChainInput;

pub trait Ctor: Send + Sync + 'static {
    type Target<'a>: Send + Sync + 'a;
}

pub trait InputCtor: Send + Sync + 'static {
    type Target<'a>: ChainInput;
}

pub trait OutputCtor: Send + Sync + 'static {
    type Target<'a>: Send + Sync + 'a;
}

impl<T: Ctor> InputCtor for T
where
    for<'a> T::Target<'a>: ChainInput,
{
    type Target<'a> = T::Target<'a>;
}

impl<T: Ctor> OutputCtor for T
where
    for<'a> T::Target<'a>: Send + Sync + 'a,
{
    type Target<'a> = T::Target<'a>;
}

pub struct StringCtor;

impl Ctor for StringCtor {
    type Target<'a> = String;
}
