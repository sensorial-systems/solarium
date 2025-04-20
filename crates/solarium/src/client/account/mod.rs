pub struct Account<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<T> Default for Account<T> {
    fn default() -> Self {
        let phantom = Default::default();
        Self { phantom }
    }
}

impl<T> Account<T> {
    pub fn new() -> Self {
        Default::default()
    }
}
