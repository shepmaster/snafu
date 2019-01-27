pub struct Context<E, C> {
    pub error: E,
    pub context: C,
}

pub trait ResultExt<T, E> {
    fn context<C>(self, context: C) -> Result<T, Context<E, C>>;
}

impl<T, E> ResultExt<T, E> for std::result::Result<T, E> {
    fn context<C>(self, context: C) -> Result<T, Context<E, C>> {
        self.map_err(|error| Context { error, context })
    }
}
