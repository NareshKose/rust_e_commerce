use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize , sqlx::FromRow)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub stock: i32,
    pub category: Option<String>,
    pub status: Option<String>
}


#[derive(Serialize, Deserialize, Debug, Validate , sqlx::FromRow)]
pub struct NewProduct {
    #[validate(length(min = 1, message = "Name cannot be empty"))]
    pub name: String,

    pub description: Option<String>,

    // #[validate(range(min = 0.0, message = "Price must be >= 0"))]
    pub price: Decimal,

    #[validate(range(min = 0, message = "Stock must be >= 0"))]
    pub stock: i32,

    pub category: Option<String>,
}
#[derive(Debug, Deserialize , sqlx::FromRow)]
pub struct UpdateStatus {
    pub status: String, // "listed" or "unlisted"
}


#[derive(Debug, Deserialize)]
pub struct UpdateProduct {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<Decimal>,
    pub category: Option<String>,
    pub status: Option<String>,
}