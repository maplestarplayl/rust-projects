use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize)]
pub struct Article {
    pub title: String,
    pub link: String,
}