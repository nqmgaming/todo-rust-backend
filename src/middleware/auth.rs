use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use actix_web::{
    dev::{forward_ready, Service, ServiceResponse, Transform},
    Error as ActixError,
};
use futures_util::future::LocalBoxFuture;
use crate::db::database::Database;
use crate::db::data_trait::todo_data_trait::TodoData;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = credentials.token();
    
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_key".into());
    let key = DecodingKey::from_secret(secret.as_ref());

    match decode::<Claims>(token, &key, &Validation::default()) {
        Ok(claims) => {
            // Extract user_id from token and set it in request extensions
            let user_id = claims.claims.sub;
            req.extensions_mut().insert(user_id);
            Ok(req)
        },
        Err(_) => Err((ErrorUnauthorized("Invalid token"), req)),
    }
}

// Middleware để kiểm tra quyền sở hữu todo
pub struct TodoOwnershipChecker {
    db: actix_web::web::Data<Database>,
}

impl TodoOwnershipChecker {
    pub fn new(db: actix_web::web::Data<Database>) -> Self {
        TodoOwnershipChecker { db }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TodoOwnershipChecker
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static + Clone,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Transform = TodoOwnershipCheckerMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TodoOwnershipCheckerMiddleware {
            service,
            db: self.db.clone(),
        }))
    }
}

pub struct TodoOwnershipCheckerMiddleware<S> {
    service: S,
    db: actix_web::web::Data<Database>,
}

impl<S, B> Service<ServiceRequest> for TodoOwnershipCheckerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = ActixError> + 'static + Clone,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = ActixError;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let db = self.db.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Chỉ kiểm tra quyền sở hữu cho các yêu cầu PATCH, DELETE đến /todos/{uuid}
            let path = req.path();
            if (req.method() == actix_web::http::Method::PATCH || req.method() == actix_web::http::Method::DELETE) 
                && path.starts_with("/api/v1/todos/") && path.len() > 14 {
                
                let todo_id = path[14..].to_string();
                
                // Lấy user_id từ request extensions
                let extensions = req.extensions();
                let user_id = match extensions.get::<String>() {
                    Some(id) => id.clone(),
                    None => return Err(AppError::unauthorized("User ID not found in request").into()),
                };
                
                // Kiểm tra quyền sở hữu todo
                match Database::get_one_todo(&db, todo_id.clone()).await {
                    Ok(todo) => {
                        if todo.user_id != user_id {
                            return Err(AppError::unauthorized("You don't have permission to access this todo").into());
                        }
                    },
                    Err(_) => {
                        // Nếu todo không tồn tại, cho phép request tiếp tục
                        // Hàm xử lý sẽ trả về lỗi not found sau
                    }
                }
            }
            
            service.call(req).await
        })
    }
}
