use actix_web::{web, HttpResponse, HttpRequest, Error, cookie::Cookie};
use sqlx::MySqlPool;
use uuid::Uuid;
use crate::models::auth::{SignupData, LoginData};
use crate::utils::hash::{hash_password, verify_password};
use crate::utils::jwt::create_jwt;
use cookie::{time::Duration};
use validator::validate_email;

pub async fn signup_user(
    pool: web::Data<MySqlPool>,
    data: web::Json<SignupData>,
) -> Result<HttpResponse, Error> {
    let user = data.into_inner();

     let username = user.username.trim();
    let email = user.email.trim();
    let password = user.password.trim();

    if username.is_empty() {
        return Ok(HttpResponse::BadRequest().body("Username cannot be empty"));
    }

        if username.len() > 5 {
        return Ok(HttpResponse::BadRequest().body("Username is very short"));
    }
    if email.is_empty() || !validate_email(email) {
        return Ok(HttpResponse::BadRequest().body("Invalid email address"));
    }

    if password.len() < 8 || password.len() > 32 {
        return Ok(HttpResponse::BadRequest().body("Password must be between 8 and 32 characters"));
    }

    let hashed_pwd = hash_password(password)
        .map_err(|_| HttpResponse::InternalServerError().body("Password hashing failed")).unwrap();

    let user_id = Uuid::new_v4().to_string();


    let result = sqlx::query!(
        "INSERT INTO users (id, username, email, password) VALUES (?, ?, ?, ?)",
       user_id,
        username,
        email,
        hashed_pwd,
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => Ok(HttpResponse::Ok().body("User registered successfully")),
        Err(e) => {
            eprintln!("DB error: {:?}", e);
            Ok(HttpResponse::InternalServerError().body("User registration failed please check you data provided "))
        }
    }
}

pub async fn login_user(
    pool: web::Data<MySqlPool>,
    data: web::Json<LoginData>,
) -> Result<HttpResponse, Error> {
    let login = data.into_inner();

    let row = sqlx::query!(
        "SELECT id, email, password, role FROM users WHERE email = ?",
        login.email.trim(),
    )
    .fetch_optional(pool.get_ref())
    .await
    .map_err(|_| HttpResponse::InternalServerError().body("Query failed")).unwrap();

    if let Some(user) = row {
        let is_valid = verify_password(&login.password, &user.password)
            .map_err(|_| HttpResponse::InternalServerError().body("Verification failed")).unwrap();

        if is_valid {
            let token = create_jwt(&user.id, &user.email, &user.role.unwrap())
                .map_err(|_| HttpResponse::InternalServerError().body("Token creation failed")).unwrap();

            
                       let cookie = Cookie::build("auth_token", token.clone())
                            .http_only(true)
                            .secure(true) 
                            .path("/")
                            .same_site(actix_web::cookie::SameSite::Lax)
                            .finish();
            return Ok(HttpResponse::Ok().cookie(cookie).json(token));
        } else {
            return Ok(HttpResponse::Unauthorized().body("Invalid credentials"));
        }
    }


    Ok(HttpResponse::Unauthorized().body("User not found"))
}




pub async fn logout_user(_req: HttpRequest) -> Result<HttpResponse, Error> {
     let expired_cookie = Cookie::build("auth_token", "")
        .path("/")
        .http_only(true)
        .max_age(cookie::time::Duration::seconds(0)) 
        .finish();

    Ok(HttpResponse::Ok().body("Logged out successfully"))
}


