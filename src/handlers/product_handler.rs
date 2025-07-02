use actix_web::{web, HttpResponse, Responder ,  HttpRequest , HttpMessage};
use uuid::Uuid;
use sqlx::MySqlPool;
use validator::Validate;
use crate::models::product::UpdateProduct;
use crate::models::auth::Claims;
use crate::models::product::{NewProduct, Product, UpdateStatus};

pub async fn add_product(
    pool: web::Data<MySqlPool>,
    data: web::Json<NewProduct>,
    req: HttpRequest,
) -> impl Responder {
    

     let extensions = req.extensions();
    let claims = extensions.get::<Claims>();

     if let Some(claims) = claims {
        if claims.role != "admin" {
            return HttpResponse::Forbidden().body("Only customers can place orders");
        }
    } else {
        return HttpResponse::Unauthorized().body("Unauthorized access");
    }

    if let Err(errors) = data.validate() {
        return HttpResponse::BadRequest().json(errors);
    }
    let price_str = data.price.to_string();

    let new_id = Uuid::new_v4();

    let result = sqlx::query(
        r#"
        INSERT INTO products (id, name, description, price, stock, category)
        VALUES (?, ?, ?, ?, ?, ?)
        "#
    )
    .bind(new_id.to_string())
    .bind(&data.name.trim())
    .bind(&data.description)
    .bind(price_str.trim())
    .bind(data.stock)
    .bind(&data.category)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "id": new_id  , "message" : "prodcut is added"})),
        Err(e) => {
            eprintln!("Failed to insert product: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}



pub async fn get_products(pool: web::Data<MySqlPool> , 
     ) -> impl Responder {

    let result = sqlx::query_as::<_, Product>("SELECT * FROM products")
        .fetch_all(pool.get_ref())
        .await;

   
    match result {
        Ok(products) => HttpResponse::Ok().json(products),
 Err(e) => {
        eprintln!("Database error: {}", e); 
        HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
    }    }
}


pub async fn get_product_by_id(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,

) -> impl Responder {

    
    let id = path.into_inner();
    println!("{}", id);

    let product = sqlx::query_as::<_, Product>(
        "SELECT * FROM products WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await;

    match product {
        Ok(Some(product)) => HttpResponse::Ok().json(product),
        Ok(None) => HttpResponse::NotFound().body("Product not found"),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn get_products_by_category(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> impl Responder {
     
    let category = path.into_inner();

    let products = sqlx::query_as::<_, Product>(
       "SELECT id, name, description, price, stock, category, status FROM products WHERE category = ?"
    )
    .bind(category)
    .fetch_all(pool.get_ref())
    .await;

    match products {
        Ok(products) => HttpResponse::Ok().json(products),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn update_product_status(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
    status: web::Json<UpdateStatus>,
    req: HttpRequest,

) -> impl Responder {

     let extensions = req.extensions();
    let claims = extensions.get::<Claims>();

if let Some(claims) = claims {
        if claims.role != "admin" {
            return HttpResponse::Forbidden().body("Only admin can update details");
        }
    } else {
        return HttpResponse::Unauthorized().body("Unauthorized access");
    }
    let id = path.into_inner();
    println!("{}", id);

    let status = status.into_inner().status;

    println!("dd");
    let result = sqlx::query("UPDATE products SET status = ? WHERE id = ?")
        .bind(status)
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Status updated successfully"),
   Err(e) => {
        eprintln!("Database error: {}", e); 
        HttpResponse::InternalServerError().body(format!("Internal Server Error: {}", e))
    }   }

    
}


pub async fn update_product(
    path: web::Path<Uuid>,
    pool: web::Data<MySqlPool>,
    item: web::Json<UpdateProduct>,
    req: HttpRequest,
    
) -> impl Responder {
      let extensions = req.extensions();
    let claims = extensions.get::<Claims>();

if let Some(claims) = claims {
        if claims.role != "admin" {
            return HttpResponse::Forbidden().body("Only admin can update details");
        }
    } else {
        return HttpResponse::Unauthorized().body("Unauthorized access");
    }
    let id = path.into_inner();
    println!("{}",id);
    let update = item.into_inner();

  
    let existing = sqlx::query!(
        "SELECT * FROM products WHERE id = ?",
        id.to_string()
    )
    .fetch_optional(pool.get_ref())
    .await;

    if let Ok(Some(prod)) = existing {
        
        let new_name = update.name.unwrap_or(prod.name);
      let new_desc = update.description.clone().or(prod.description.clone());
      let new_price = update.price.clone().or(Some(prod.price.clone()));
let new_category = update.category.clone().or(prod.category.clone());
let new_status = update.status.clone().or(prod.status.clone());

       
        let result = sqlx::query!(
            "UPDATE products SET name = ?, description = ?, price = ?, category = ?, status = ? WHERE id = ?",
            new_name,
            new_desc,
            new_price,
            new_category,
            new_status,
            id.to_string()
        )
        .execute(pool.get_ref())
        .await;

    
    println!("updated");
        match result {
            Ok(_) => HttpResponse::Ok().body("Product updated successfully"),
            Err(e) => {
                println!("Update error: {:?}", e);
                HttpResponse::InternalServerError().body("Failed to update product")
            }
        }
    } else {
        HttpResponse::NotFound().body("Product not found")
    }
}