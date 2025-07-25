pub struct ExecutorOptions {
    pub max_iterations: Option<usize>,
    pub max_consecutive_fails: Option<usize>,
}

impl ExecutorOptions {
    pub fn new(max_iterations: Option<usize>, max_consecutive_fails: Option<usize>) -> Self {
        Self {
            max_iterations,
            max_consecutive_fails,
        }
    }

    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = Some(max_iterations);
        self
    }

    pub fn without_max_iterations(mut self) -> Self {
        self.max_iterations = None;
        self
    }

    pub fn with_max_consecutive_fails(mut self, max_consecutive_fails: usize) -> Self {
        self.max_consecutive_fails = Some(max_consecutive_fails);
        self
    }

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
