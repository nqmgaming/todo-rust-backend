use crate::db::data_trait::todo_data_trait::TodoData;
use crate::db::database::Database;
use crate::error::AppError;
use crate::models::app::{
    ApiResponseDeleteTodoResponse, ApiResponseTodoResponse, ApiResponseTodoResponseList,
};
use crate::models::todo::{
    CreateTodoRequest, GetTodoURL, TodoQueryParams, TodoResponse, TodoResponseList,
    UpdateTodoRequest, UpdateTodoURL,
};
use crate::services::cache_service::CacheService;
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, patch, post, HttpMessage, HttpRequest};

const CACHE_TTL: u64 = 300; // 5 minutes

pub fn todo_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_todos);
    cfg.service(get_todo);
    cfg.service(create_todo);
    cfg.service(update_todo);
    cfg.service(delete_todo);
}

#[get("")]
async fn get_todos(
    req: HttpRequest,
    db: Data<Database>,
    query_params: Query<TodoQueryParams>,
) -> Result<Json<ApiResponseTodoResponseList>, AppError> {
    let extensions = req.extensions();
    let user_id = extensions
        .get::<String>()
        .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "User ID not found in request"))?;

    // Clone query_params before consuming it
    let query_params_inner = query_params.into_inner();
    let cache_key = format!(
        "todos:user:{}:list:{}",
        user_id,
        query_params_inner.to_string()
    );

    // Try to get from cache first
    if let Ok(Some(cached_data)) = db
        .redis_client
        .get_cached::<TodoResponseList>(&cache_key)
        .await
    {
        return Ok(Json(ApiResponseTodoResponseList {
            success: true,
            message: "Todos retrieved successfully".to_string(),
            data: Some(cached_data),
        }));
    }

    // If not in cache, get from database
    let todos = Database::get_all_todos(
        &db,
        user_id.to_string(),
        query_params_inner.pagination,
        query_params_inner.filter,
    )
    .await?;

    // Store in cache
    if let Ok(_) = db
        .redis_client
        .set_cached(&cache_key, &todos, CACHE_TTL)
        .await
    {
        log::info!("Successfully cached todos list for user {}", user_id);
    }

    Ok(Json(ApiResponseTodoResponseList {
        success: true,
        message: "Todos retrieved successfully".to_string(),
        data: Some(todos),
    }))
}

#[get("/{uuid}")]
async fn get_todo(
    get_todo_url: Path<GetTodoURL>,
    req: HttpRequest,
    db: Data<Database>,
) -> Result<Json<ApiResponseTodoResponse>, AppError> {
    let extensions = req.extensions();
    let user_id = extensions
        .get::<String>()
        .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "User ID not found in request"))?;

    let cache_key = format!("todos:user:{}:item:{}", user_id, get_todo_url.uuid);

    // Try to get from cache first
    if let Ok(Some(cached_data)) = db.redis_client.get_cached::<TodoResponse>(&cache_key).await {
        return Ok(Json(ApiResponseTodoResponse {
            success: true,
            message: "Todo retrieved successfully".to_string(),
            data: Some(cached_data),
        }));
    }

    // If not in cache, get from database
    let todo = Database::get_one_todo(&db, get_todo_url.uuid.clone()).await?;

    // Store in cache
    if let Ok(_) = db
        .redis_client
        .set_cached(&cache_key, &todo, CACHE_TTL)
        .await
    {
        log::info!("Successfully cached todo for user {}", user_id);
    }

    Ok(Json(ApiResponseTodoResponse {
        success: true,
        message: "Todo retrieved successfully".to_string(),
        data: Some(todo),
    }))
}

#[post("")]
async fn create_todo(
    body: Json<CreateTodoRequest>,
    req: HttpRequest,
    db: Data<Database>,
) -> Result<Json<ApiResponseTodoResponse>, AppError> {
    let extensions = req.extensions();
    let user_id = extensions
        .get::<String>()
        .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "User ID not found in request"))?;

    let todo = Database::add_todo(&db, user_id.to_string(), body.into_inner()).await?;

    // Invalidate user's todos list cache
    let cache_pattern = format!("todos:user:{}:*", user_id);
    if let Err(e) = db
        .redis_client
        .delete_cached_by_pattern(&cache_pattern)
        .await
    {
        log::error!(
            "Failed to invalidate todos cache for user {}: {:?}",
            user_id,
            e
        );
    } else {
        log::info!("Successfully invalidated todos cache for user {}", user_id);
    }

    Ok(Json(ApiResponseTodoResponse {
        success: true,
        message: "Todo created successfully".to_string(),
        data: Some(todo),
    }))
}

#[patch("/{uuid}")]
async fn update_todo(
    update_todo_url: Path<UpdateTodoURL>,
    body: Json<UpdateTodoRequest>,
    req: HttpRequest,
    db: Data<Database>,
) -> Result<Json<ApiResponseTodoResponse>, AppError> {
    let extensions = req.extensions();
    let user_id = extensions
        .get::<String>()
        .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "User ID not found in request"))?;

    let todo = Database::update_todo(
        &db,
        update_todo_url.uuid.clone(),
        body.title.clone(),
        body.description.clone(),
        body.is_completed,
    )
    .await?;

    let todo_response = TodoResponse::from(todo);

    // Invalidate both specific todo and list caches for the user
    let cache_pattern = format!("todos:user:{}:*", user_id);
    if let Err(e) = db
        .redis_client
        .delete_cached_by_pattern(&cache_pattern)
        .await
    {
        log::error!(
            "Failed to invalidate todos cache for user {}: {:?}",
            user_id,
            e
        );
    } else {
        log::info!("Successfully invalidated todos cache for user {}", user_id);
    }

    Ok(Json(ApiResponseTodoResponse {
        success: true,
        message: "Todo updated successfully".to_string(),
        data: Some(todo_response),
    }))
}

#[delete("/{uuid}")]
async fn delete_todo(
    todo_url: Path<GetTodoURL>,
    req: HttpRequest,
    db: Data<Database>,
) -> Result<Json<ApiResponseDeleteTodoResponse>, AppError> {
    let extensions = req.extensions();
    let user_id = extensions
        .get::<String>()
        .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "User ID not found in request"))?;

    let response = Database::delete_todo(&db, todo_url.uuid.clone()).await?;

    // Clear cache for the user
    let cache_pattern = format!("todos:user:{}:*", user_id);
    if let Err(e) = db
        .redis_client
        .delete_cached_by_pattern(&cache_pattern)
        .await
    {
        log::error!(
            "Failed to invalidate todos cache for user {}: {:?}",
            user_id,
            e
        );
    } else {
        log::info!("Successfully invalidated todos cache for user {}", user_id);
    }

    Ok(Json(ApiResponseDeleteTodoResponse {
        success: true,
        message: "Todo deleted successfully".to_string(),
        data: Some(response),
    }))
}
