use crate::embedding::Embedder;
use crate::schemas::BuilderError;
use crate::vectorstore::opensearch::Store;
use opensearch::OpenSearch;
use std::sync::Arc;

pub struct StoreBuilder {
    client: Option<OpenSearch>,
    embedder: Option<Arc<dyn Embedder>>,
    k: i32,
    index: Option<String>,
    vector_field: String,
    content_field: String,
}

impl StoreBuilder {
    // Returns a new StoreBuilder instance with default values for each option
    pub fn new() -> Self {
        StoreBuilder {
            client: None,
            embedder: None,
            k: 2,
            index: None,
            vector_field: "vector_field".to_string(),
            content_field: "page_content".to_string(),
        }
    }

    pub fn client(mut self, client: OpenSearch) -> Self {
        self.client = Some(client);
        self
    }

    pub fn embedder<E: Embedder + 'static>(mut self, embedder: E) -> Self {
        self.embedder = Some(Arc::new(embedder));
        self
    }

    pub fn k(mut self, k: i32) -> Self {
        self.k = k;
        self
    }

    pub fn index(mut self, index: &str) -> Self {
        self.index = Some(index.to_string());
        self
    }

    pub fn vector_field(mut self, vector_field: &str) -> Self {
        self.vector_field = vector_field.to_string();
        self
    }

    pub fn content_field(mut self, content_field: &str) -> Self {
        self.content_field = content_field.to_string();
        self
    }

    // Finalize the builder and construct the Store object
    pub async fn build(self) -> Result<Store, BuilderError> {
        let client = self.client.ok_or(BuilderError::MissingField("client"))?;
        let embedder = self
            .embedder
            .ok_or(BuilderError::MissingField("embedder"))?;
        let index = self.index.ok_or(BuilderError::MissingField("index"))?;

        Ok(Store {
            client,
            embedder,
            k: self.k,
            index,
            vector_field: self.vector_field,
            content_field: self.content_field,
        })
    }
}

impl Default for StoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}
