pub struct Client<C> {
    pub stream: C,
}

impl<C> Client<C> {
    pub fn new(stream: C) -> Self {
        Self { stream }
    }
}
