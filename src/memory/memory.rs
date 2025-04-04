use crate::schemas::Message;

pub trait Memory: Send + Sync {
    fn messages(&self) -> Vec<Message>;

    fn add_message(&mut self, message: Message);

    fn clear(&mut self);

    fn to_string(&self) -> String;
}

impl<M> From<M> for Box<dyn Memory>
where
    M: Memory + 'static,
{
    fn from(memory: M) -> Self {
        Box::new(memory)
    }
}
