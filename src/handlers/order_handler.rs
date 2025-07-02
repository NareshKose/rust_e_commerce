use crate::models::order::{OrderRequest, OrderProduct};
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};
use sqlx::MySqlPool;
use uuid::Uuid;
use serde_json::json;
use rust_decimal::Decimal;
use crate::models::auth::Claims;
use crate::utils::kafka::send_to_kafka;

use std::fs::OpenOptions;
use std::io::Write;
use chrono::Utc;

use crate::models::order::StatusChangeRequest;
  


pub async fn place_order(
    pool: web::Data<MySqlPool>,
    order: web::Json<OrderRequest>,
    req: HttpRequest,
) -> impl Responder {
   

    let order = order.into_inner();
    let order_id = Uuid::new_v4();
    let mut total_amount = Decimal::new(0, 2);
    let mut products_for_storage = Vec::new();

   
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>();
    if let Some(claims) = claims {
        if claims.role != "customer" {
            return HttpResponse::Forbidden().body("Only customers can place orders");
        }
    } else {
        return HttpResponse::Unauthorized().body("Unauthorized access");
    }

    let mut tx = match pool.begin().await {
        Ok(t) => t,
        Err(_) => return HttpResponse::InternalServerError().json(json!({ "error": "Failed to begin transaction" })),
    };


   
    for item in &order.products {
        let result = sqlx::query!(
            "SELECT name, price, stock, status FROM products WHERE id = ?",
            item.product_id
        )
        .fetch_optional(&mut *tx)
        .await;

       let product = match result {
            Ok(Some(prod)) => prod,
            _ => {
                return HttpResponse::BadRequest().json(
                    json!({ "error": format!("Product with ID {} not found", item.product_id) }),
                );
            }
        };
        
        if product.status.as_deref() != Some("listed") {
            return HttpResponse::BadRequest().json(
                json!({ "error": format!("Product '{}' is unavailable", product.name) }),
            );
        }
        
        if item.quantity > product.stock {
            return HttpResponse::BadRequest().json(
                json!({ "error": format!("Only {} units available for '{}'", product.stock, product.name) }),
            );
        }
        
        let item_total = product.price * Decimal::from(item.quantity);
        total_amount += item_total;
        
        products_for_storage.push(OrderProduct {
            order_id: order_id.to_string(),
            product_id: item.product_id.clone(),
            product_name: product.name.clone(),
            quantity: item.quantity,
            total_price : item_total
        });
    }

    println!("{:?}", &products_for_storage);

  
    let products_json = match serde_json::to_string(&products_for_storage) {
        Ok(json) => json,
        Err(_) => {
            return HttpResponse::InternalServerError().json(
                json!({ "error": "Failed to serialize order products" }),
            );
        }
    };

    let insert_result = sqlx::query!(
        "INSERT INTO orders (id, user_id, total_amount, status, shipping_address)
         VALUES (?, ?, ?, ?, ?)",
        order_id.to_string(),
        order.user_id,
        total_amount,
        "pending",
        order.shipping_address
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = insert_result {
        println!("Insert order error: {:?}", e);
        return HttpResponse::InternalServerError().json(json!({ "error": "Failed to insert order" }));
    }

 
    for item in &products_for_storage {
        let item_id = Uuid::new_v4();

        let insert_item_result = sqlx::query!(
            "INSERT INTO order_items (id, order_id, product_id, product_name, quantity, created_at)
             VALUES (?, ?, ?, ?, ?, NOW())",
            item_id.to_string(),
            item.order_id,
            item.product_id,
            item.product_name,
            item.quantity
        )
        .execute(&mut *tx)
        .await;

        if let Err(e) = insert_item_result {
            println!("Failed to insert order item: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to store product '{}' in order_items", item.product_name)
            }));
        }

        
        let stock_update_result = sqlx::query!(
            "UPDATE products SET stock = stock - ? WHERE id = ?",
            item.quantity,
            item.product_id
        )
        .execute(&mut *tx)
        .await;

        if let Err(e) = stock_update_result {
            println!("Failed to update stock: {:?}", e);
            return HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to update stock for '{}'", item.product_name)
            }));
        }




let stock_check = sqlx::query!(
    "SELECT stock FROM products WHERE id = ?",
    item.product_id
)
.fetch_one(&mut *tx)
.await;

if let Ok(stock_row) = stock_check {

      if stock_row.stock <= 3 {
        let alert = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "product_id": item.product_id,
            "product_name": item.product_name,
            "remaining_stock": stock_row.stock,
            "message": "Product is out of stock. Please restock."
        });

        if let Err(e) = send_to_kafka("out-of-stock-topic", alert.to_string()).await {
            println!("Failed to send out-of-stock Kafka alert: {:?}", e);
        } else {
            println!("Out-of-stock Kafka alert sent for '{}'", item.product_name);

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("logs/out_of_stock_log.jsonl")
                .expect("Unable to open or create log file");

            if let Err(e) = writeln!(file, "{}", alert.to_string()) {
                eprintln!("Failed to write alert to log file: {:?}", e);
            }
        }
    }
}


        
    }

    if let Err(e) = tx.commit().await {
        println!("Transaction commit failed: {:?}", e);
        return HttpResponse::InternalServerError().json(json!({ "error": "Failed to commit transaction" }));
    }

    if let Err(err) = send_to_kafka("order-topic", products_json.clone()).await {
        println!("Kafka error: {:?}", err);
    }

    HttpResponse::Ok().json(json!({
        "message": "Order placed successfully",
        "order_id": order_id,
        "total_amount": total_amount,
        "products": products_for_storage
    }))
}


pub async fn update_order_status(
    pool: web::Data<MySqlPool>,
    order_id: web::Path<String>,
    status_change: web::Json<StatusChangeRequest>,
    req: HttpRequest,
) -> impl Responder {
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>();

    
    if let Some(claims) = claims {
        if claims.role != "admin" {
            return HttpResponse::Forbidden().json(json!({
                "error": "Only admins can update order status"
            }));
        }
    } else {
        return HttpResponse::Unauthorized().json(json!({
            "error": "Unauthorized access"
        }));
    }

    let order_id = order_id.into_inner();
    let new_status = status_change.into_inner().new_status.to_lowercase();

   
    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Failed to start transaction"})),
    };

    
    let current_status = match sqlx::query!(
        "SELECT status FROM orders WHERE id = ?",
        order_id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(record)) => record.status,
        Ok(None) => return HttpResponse::NotFound().json(json!({"error": "Order not found"})),
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
    };

   
    if let Some(ref status) = current_status {
        if status == "delivered" || status == "cancelled" {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Cannot change status from {}", status)
            }));
        }
    } else {
        return HttpResponse::BadRequest().json(json!({
            "error": "Order has no status set"
        }));
    }

    if !["shipped", "delivered"].contains(&new_status.as_str()) {
        return HttpResponse::BadRequest().json(json!({
            "error": "Invalid status for admin update"
        }));
    }


    if let Err(_) = sqlx::query!(
        "UPDATE orders SET status = ? WHERE id = ?",
        new_status,
        order_id
    )
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to update order status"
        }));
    }


    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to commit transaction"
        }));
    }

    HttpResponse::Ok().json(json!({
        "message": format!("Order status updated to {}", new_status)
    }))
}


pub async fn cancel_order(
    pool: web::Data<MySqlPool>,
    order_id: web::Path<String>,
    req: HttpRequest,
) -> impl Responder {
    let extensions = req.extensions();
    let claims = extensions.get::<Claims>();

   
       let user_id = if let Some(claims) = claims {
        if claims.role != "user" {
            return HttpResponse::Forbidden().body("customer cancel route");
            
        }
        &claims.sub  
    } else {
        return HttpResponse::Unauthorized().body("anauthorized access");
    };

    let order_id = order_id.into_inner();

    let mut tx = match pool.begin().await {
        Ok(tx) => tx,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to start transaction"
        })),
    };

  
    let order = match sqlx::query!(
        "SELECT status, user_id FROM orders WHERE id = ?",
        order_id
    )
    .fetch_optional(&mut *tx)
    .await
    {
        Ok(Some(order)) => order,
        Ok(None) => return HttpResponse::NotFound().json(json!({"error": "Order not found"})),
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
    };

 
    if let Some(ref owner_id) = order.user_id {
        if owner_id != user_id {
            return HttpResponse::Forbidden().json(json!({
                "error": "You can only cancel your own orders"
            }));
        }
    } else {
        return HttpResponse::BadRequest().json(json!({
            "error": "Order has no associated user"
        }));
    }


    if let Some(ref status) = order.status {
        if status == "delivered" || status == "cancelled" {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Cannot cancel order with status {}", status)
            }));
        }
    } else {
        return HttpResponse::BadRequest().json(json!({
            "error": "Order has no status set"
        }));
    }


    let order_items = match sqlx::query!(
        "SELECT product_id, quantity FROM order_items WHERE order_id = ?",
        order_id
    )
    .fetch_all(&mut *tx)
    .await
    {
        Ok(items) => items,
        Err(_) => return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to fetch order items"
        })),
    };

    for item in &order_items {
        if let (Some(product_id), Some(quantity)) = (&item.product_id, item.quantity) {
            if let Err(_) = sqlx::query!(
                "UPDATE products SET stock = stock + ? WHERE id = ?",
                quantity,
                product_id
            )
            .execute(&mut *tx)
            .await
            {
                return HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to update product stock"
                }));
            }
        }
    }

    if let Err(_) = sqlx::query!(
        "UPDATE orders SET status = 'cancelled' WHERE id = ?",
        order_id
    )
    .execute(&mut *tx)
    .await
    {
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to cancel order"
        }));
    }

    if let Err(_) = tx.commit().await {
        return HttpResponse::InternalServerError().json(json!({
            "error": "Failed to complete cancellation"
        }));
    }

    HttpResponse::Ok().json(json!({
        "message": "Order cancelled successfully"
    }))
}




pub async fn get_order_details(
    pool: web::Data<MySqlPool>,
    order_id: web::Path<String>,
) -> impl Responder {

    let order_id = order_id.into_inner();

    let order = match sqlx::query!(
        "SELECT id, user_id, total_amount, status, created_at, shipping_address FROM orders WHERE id = ?",
        order_id
    )
    .fetch_optional(&**pool)
    .await
    {
        Ok(Some(order)) => order,
        Ok(None) => return HttpResponse::NotFound().json(json!({"error": "Order not found"})),
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Database error"})),
    };

   
    let order_items = match sqlx::query!(
        "SELECT id, product_id, product_name, quantity, created_at FROM order_items WHERE order_id = ?",
        order_id
    )
    .fetch_all(&**pool)
    .await
    {
        Ok(items) => items,
        Err(_) => return HttpResponse::InternalServerError().json(json!({"error": "Failed to fetch order items"})),
    };

 
    HttpResponse::Ok().json(json!({
        "order": {
            "id": order.id,
            "user_id": order.user_id,
            "total_amount": order.total_amount,
            "status": order.status,
            "created_at": order.created_at,
            "shipping_address": order.shipping_address
        },
        "items": order_items.iter().map(|item| json!({
            "id": item.id,
            "product_id": item.product_id,
            "product_name": item.product_name,
            "quantity": item.quantity,
            "created_at": item.created_at
        })).collect::<Vec<_>>()
    }))
}


