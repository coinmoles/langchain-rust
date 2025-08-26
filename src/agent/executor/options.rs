/// Options for the [`AgentExecutor`](crate::agent::AgentExecutor)
pub struct ExecutorOptions {
    /// Max iterations allowed.
    pub max_iterations: Option<usize>,
    /// Max number of consecutive failures allowed.
    pub max_consecutive_fails: Option<usize>,
}

impl ExecutorOptions {
    /// Constructs a new [`ExecutorOptions`]
    pub fn new(max_iterations: Option<usize>, max_consecutive_fails: Option<usize>) -> Self {
        Self {
            max_iterations,
            max_consecutive_fails,
        }
    }

    /// Sets the max iterations allowed.
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    /// Disables the max iteration check.
    pub fn without_max_iterations(mut self) -> Self {
        self.max_iterations = None;
        self
    }

    /// Sets the max number of consecutive failures allowed.
    pub fn with_max_consecutive_fails(mut self, max_consecutive_fails: usize) -> Self {
        self.max_consecutive_fails = Some(max_consecutive_fails);
        self
    }

    /// Disables the consecutive failures check.
    pub fn without_max_consecutive_fails(mut self) -> Self {
        self.max_consecutive_fails = None;
        self
    }
}

impl Default for ExecutorOptions {
    fn default() -> Self {
        Self {
            max_iterations: Some(10),
            max_consecutive_fails: Some(3),
        }
    }
}
