use futures::Future;

use std::{error::Error, pin::Pin};

pub type StreamingFunc = dyn FnMut(&str) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send>>
    + Send;
