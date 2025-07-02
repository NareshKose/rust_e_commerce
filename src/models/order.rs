use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

#[derive(Debug, Deserialize)]
pub struct OrderRequest {
    pub user_id: String,
    pub shipping_address: String,
    pub products: Vec<ProductOrderRequest>, // 👈 change this
}


#[derive(Debug, Serialize, Deserialize)]
pub struct OrderProduct {
    pub order_id: String,
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub total_price : Decimal
}


#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemRecord {
    pub id: String,
    pub order_id: String,
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
}

#[derive(Debug, Deserialize)]
pub struct ProductOrderRequest {
    pub product_id: String,
    pub quantity: i32,
}

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Debug)]
enum OrderStatus {
    Pending,
    Shipped,
    Delivered,
    Cancelled,
}

// Map string to enum
impl From<&str> for OrderStatus {
    fn from(s: &str) -> Self {
        match s {
            "pending" => OrderStatus::Pending,
            "shipped" => OrderStatus::Shipped,
            "delivered" => OrderStatus::Delivered,
            "cancelled" => OrderStatus::Cancelled,
            _ => OrderStatus::Pending, // default fallback
        }
    }
}

#[derive(serde::Deserialize)]
pub struct StatusChangeRequest {
    pub new_status: String,
}