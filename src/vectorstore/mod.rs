#![allow(dead_code)]
// I have no idea how to remove dead codes here.

#[allow(clippy::module_inception)]
mod vectorstore;
pub use vectorstore::*;

mod options;
pub use options::*;

mod retriever;
pub use retriever::*;

#[cfg(feature = "postgres")]
pub mod pgvector;

#[cfg(feature = "sqlite-vss")]
pub mod sqlite_vss;

#[cfg(feature = "sqlite-vec")]
pub mod sqlite_vec;

#[cfg(feature = "surrealdb")]
pub mod surrealdb;

#[cfg(feature = "opensearch")]
pub mod opensearch;

#[cfg(feature = "qdrant")]
pub mod qdrant;
