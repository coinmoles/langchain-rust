use std::{error::Error, future::Future, pin::Pin};

use super::AgentStep;

pub type OnStepFunc = dyn FnMut(
        &AgentStep,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send>>
    + Send
    + Sync;
