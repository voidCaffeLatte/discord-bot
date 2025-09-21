pub struct AIText {
    text: String,
    web_references: Option<Vec<WebReference>>,
}

impl AIText {
    pub fn new(
        text: String,
        web_references: Option<Vec<WebReference>>,
    ) -> Self {
        Self {
            text,
            web_references,
        }
    }

    pub fn text(&self) -> &str {
        self.text.as_str()
    }

    pub fn web_references(&self) -> Option<&Vec<WebReference>> {
        self.web_references.as_ref()
    }
}

pub struct WebReference {
    title: String,
    url: String,
}

impl WebReference {
    pub fn new(
        title: String,
        url: String,
    ) -> Self {
        Self {
            title,
            url,
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn url(&self) -> &str {
        self.url.as_str()
    }
}
