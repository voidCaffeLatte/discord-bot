pub struct User {
    id: String,
}

impl User {
    pub fn new(
        id: String
    ) -> Self {
        Self {
            id
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
