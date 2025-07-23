#[macro_export]
macro_rules! tools_vec {
    ($($tool:expr),* $(,)?) => {
        vec![$(Box::new($tool) as Box<dyn Tool>),*]
    };
}
