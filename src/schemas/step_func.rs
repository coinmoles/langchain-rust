use std::{error::Error, future::Future, pin::Pin};

use super::AgentStep;

pub type StepFunc = dyn FnMut(
        &AgentStep,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send>>
    + Send;
