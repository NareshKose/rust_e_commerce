
# Rust E-Commerce API Documentation

## Base URL

```
http://localhost:8080
```

---

##  Auth Routes

###  Login

- **Endpoint:** `POST /login`
- **Request Body:**
```json
{
  "email": "user_1@gmail.com",
  "password": "user_1@123"
}
```

---

###  Sign Up

- **Endpoint:** `POST /signup`
- **Request Body:**
```json
{
  "username": "user_2",
  "email": "user_2@gmail.com",
  "password": "user_2@123"
}
```

---

###  Logout

- **Endpoint:** `POST /logout`

---

##  Product Routes

### Add Product

- **Endpoint:** `POST /products`
- **Request Body:**
```json
{
  "name": "hp 15s",
  "stock": 0,
  "price": 33,
  "category": "electronics"
}
```

---

###  Update Product

- **Endpoint:** `PUT /products/{product_id}`
- **Request Body:**
```json
{
  "name": "Mankcdgede's Dryer",
  "stock": 10,
  "price": 33,
  "category": "electronics"
}
```

---

###  Update Product Status

- **Endpoint:** `PUT /products/{product_id}/status`
- **Request Body:**
```json
{
  "status": "unlisted"
}
```

---

###  Get Product by Category

- **Endpoint:** `GET /products/category/{category}`

---

###  Get Product by ID

- **Endpoint:** `GET /products/{product_id}`

---

###  Get All Admin Products

- **Endpoint:** `GET /products`

---

##  Order Routes

###  Place Order

- **Endpoint:** `POST /orders/place`
- **Request Body:**
```json
{
  "user_id": "5bc3baa6-9db4-459c-aad3-6b591f300e2c",
  "shipping_address": "123, MG Road, Mumbai, India",
  "products": [
    {
      "product_id": "7d59b10f-8878-429f-96e1-4445878c5b78",
      "quantity": 2
    }
  ]
}
```

- **Success Response Example:**
```json
{
  "message": "Order placed successfully",
  "order_id": "e2f703cf-ec10-47f9-863f-d28b3622d3d3",
  "products": [
    {
      "order_id": "e2f703cf-ec10-47f9-863f-d28b3622d3d3",
      "product_id": "7d59b10f-8878-429f-96e1-4445878c5b78",
      "product_name": "Mankcdgede's Dryer",
      "quantity": 2,
      "total_price": "66.00"
    }
  ],
  "total_amount": "66.00"
}
```

---

###  Update Order Status

- **Endpoint:** `PATCH /orders/{order_id}/status`
- **Request Body:**
```json
{
  "new_status": "delivered"
}
```

---

###  Cancel Order

- **Endpoint:** `POST /orders/{order_id}/cancel`

---
#  Report API Documentation

## Base URL

```
/reports/{report_type}
```

Where `{report_type}` can be:

* `daily`
* `weekly`
* `monthly`
* `yearly`

---

## 🔹 Endpoint

### GET `/reports/{report_type}`

Fetches product sales summary report for the given report type using materialized views from ClickHouse.

### Path Parameters

| Parameter     | Type   | Description                                                        | Required |
| ------------- | ------ | ------------------------------------------------------------------ | -------- |
| `report_type` | String | Type of report to fetch: `daily`, `weekly`, `monthly`, or `yearly` | Yes    |

---

## Successful Response

**HTTP 200 OK**

```json
[
  {
    "period": "2025-06-01",
    "product_name": "Hair Dryer",
    "total_units_sold": 10,
    "total_sales_amount": 330.0
  },
  {
    "period": "2025-06-01",
    "product_name": "Shampoo",
    "total_units_sold": 5,
    "total_sales_amount": 100.0
  }
]
```

### Response Fields

| Field                | Type    | Description                                        |
| -------------------- | ------- | -------------------------------------------------- |
| `period`             | String  | Aggregated time period (`YYYY-MM-DD` or `YYYY-MM`) |
| `product_name`       | String  | Name of the product                                |
| `total_units_sold`   | Integer | Total number of units sold in the period           |
| `total_sales_amount` | Float   | Total revenue from the product in that period      |

---

##  Error Responses

### 400 Bad Request

```text
Invalid report type
```

Occurs if the path value is not one of: `daily`, `weekly`, `monthly`, or `yearly`.

---

### 500 Internal Server Error

```text
Failed to fetch report
```

Occurs if there is a query failure or unexpected internal issue.

---

##  Authentication (Optional)

If protected with JWT:

* **Header:** `Authorization: Bearer <token>`
* Or set JWT in cookie as: `auth_token=<your_token>`

---

##  Example Requests

##  Backend Reference

### Actix Handler

```rust
pub async fn fetch_report_by_type(
    client: web::Data<Client>,
    path: web::Path<ReportPath>,
) -> impl Responder
```

### Route Setup

```rust
web::scope("/reports")
    .route("/{report_type}", web::get().to(report_handler::fetch_report_by_type))
```

---
