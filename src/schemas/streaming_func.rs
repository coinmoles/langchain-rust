use futures::Future;
use std::pin::Pin;

pub type StreamingFunc =
    dyn FnMut(&str) -> Pin<Box<dyn Future<Output = Result<(), ()>> + Send>> + Send;
