#[macro_export]
macro_rules! tools_vec {
    ($($tool:expr),* $(,)?) => {
        vec![$(::std::boxed::Box::new($tool) as ::std::boxed::Box<dyn $crate::tools::ToolInternal>),*]
    };
}
