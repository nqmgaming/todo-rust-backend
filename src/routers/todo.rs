use crate::db::data_trait::todo_data_trait::TodoData;
use crate::db::database::Database;
use crate::error::AppError;
use crate::models::todo::{
    CreateTodoRequest, GetTodoURL, TodoQueryParams, TodoResponse, TodoResponseList,
    UpdateTodoRequest, UpdateTodoURL,
};
use crate::services::cache_service::CacheService;
use crate::swagger::{
    ApiResponseDeleteTodoResponse, ApiResponseEmpty, ApiResponseTodoResponse,
    ApiResponseTodoResponseList,
};
use actix_web::http::StatusCode;
use actix_web::web::{Data, Json, Path, Query};
use actix_web::{delete, get, patch, post, HttpMessage, HttpRequest};
use utoipa::IntoParams;

const CACHE_TTL: u64 = 300; // 5 minutes

pub fn todo_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(get_todos);
    cfg.service(get_todo);
    cfg.service(create_todo);
    cfg.service(update_todo);
    cfg.service(delete_todo);
}

// Implement IntoParams for TodoQueryParams
impl IntoParams for TodoQueryParams {
    fn into_params(
        parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
    ) -> Vec<utoipa::openapi::path::Parameter> {
        let parameter_in =
            parameter_in_provider().unwrap_or(utoipa::openapi::path::ParameterIn::Query);

        vec![
            // Pagination params
            {
                let mut param = utoipa::openapi::path::Parameter::new("page");
                param.description = Some("Số trang hiện tại".to_string());
                param.schema = Some(utoipa::openapi::RefOr::T(
                    utoipa::openapi::schema::Schema::Object(
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::schema::Type::Integer)
                            .examples(Some(vec![serde_json::json!(1)]))
                            .build(),
                    ),
                ));
                param.parameter_in = parameter_in.clone();
                param
            },
            {
                let mut param = utoipa::openapi::path::Parameter::new("page_size");
                param.description = Some("Số lượng todo trên mỗi trang".to_string());
                param.schema = Some(utoipa::openapi::RefOr::T(
                    utoipa::openapi::schema::Schema::Object(
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::schema::Type::Integer)
                            .examples(Some(vec![serde_json::json!(10)]))
                            .build(),
                    ),
                ));
                param.parameter_in = parameter_in.clone();
                param
            },
            // Filter params
            {
                let mut param = utoipa::openapi::path::Parameter::new("search");
                param.description = Some("Tìm kiếm theo tiêu đề hoặc mô tả".to_string());
                param.schema = Some(utoipa::openapi::RefOr::T(
                    utoipa::openapi::schema::Schema::Object(
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::schema::Type::String)
                            .examples(Some(vec![serde_json::json!("Rust")]))
                            .build(),
                    ),
                ));
                param.parameter_in = parameter_in.clone();
                param
            },
            {
                let mut param = utoipa::openapi::path::Parameter::new("is_completed");
                param.description = Some("Lọc theo trạng thái hoàn thành".to_string());
                param.schema = Some(utoipa::openapi::RefOr::T(
                    utoipa::openapi::schema::Schema::Object(
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::schema::Type::Boolean)
                            .examples(Some(vec![serde_json::json!(false)]))
                            .build(),
                    ),
                ));
                param.parameter_in = parameter_in.clone();
                param
            },
            {
                let mut param = utoipa::openapi::path::Parameter::new("sort_by");
                param.description = Some("Sắp xếp theo trường".to_string());
                param.schema = Some(utoipa::openapi::RefOr::T(
                    utoipa::openapi::schema::Schema::Object(
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::schema::Type::String)
                            .examples(Some(vec![serde_json::json!("created_at")]))
                            .build(),
                    ),
                ));
                param.parameter_in = parameter_in.clone();
                param
            },
            {
                let mut param = utoipa::openapi::path::Parameter::new("sort_order");
                param.description = Some("Thứ tự sắp xếp (asc hoặc desc)".to_string());
                param.schema = Some(utoipa::openapi::RefOr::T(
                    utoipa::openapi::schema::Schema::Object(
                        utoipa::openapi::schema::ObjectBuilder::new()
                            .schema_type(utoipa::openapi::schema::Type::String)
                            .examples(Some(vec![serde_json::json!("desc")]))
                            .build(),
                    ),
                ));
                param.parameter_in = parameter_in.clone();
                param
            },
        ]
    }
}

/// Lấy danh sách tất cả các todo của người dùng hiện tại
///
/// Endpoint này trả về danh sách tất cả các todo của người dùng hiện tại.
/// Hỗ trợ phân trang và lọc theo các tiêu chí khác nhau.
#[utoipa::path(
    get,
    path = "/api/v1/todos",
    tag = "todos",
    params(
        TodoQueryParams
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Danh sách todo được trả về thành công", body = ApiResponseTodoResponseList),
        (status = 401, description = "Không được xác thực", body = ApiResponseEmpty),
        (status = 500, description = "Lỗi máy chủ", body = ApiResponseEmpty)
    )
)]
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

/// Lấy thông tin chi tiết của một todo
///
/// Endpoint này trả về thông tin chi tiết của một todo dựa trên UUID.
#[utoipa::path(
    get,
    path = "/api/v1/todos/{uuid}",
    tag = "todos",
    params(
        ("uuid" = String, Path, description = "UUID của todo cần lấy thông tin")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Thông tin todo được trả về thành công", body = ApiResponseTodoResponse),
        (status = 401, description = "Không được xác thực", body = ApiResponseEmpty),
        (status = 404, description = "Không tìm thấy todo", body = ApiResponseEmpty),
        (status = 500, description = "Lỗi máy chủ", body = ApiResponseEmpty)
    )
)]
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

/// Tạo một todo mới
///
/// Endpoint này cho phép tạo một todo mới với tiêu đề và mô tả.
#[utoipa::path(
    post,
    path = "/api/v1/todos",
    tag = "todos",
    request_body = CreateTodoRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Todo được tạo thành công", body = ApiResponseTodoResponse),
        (status = 401, description = "Không được xác thực", body = ApiResponseEmpty),
        (status = 500, description = "Lỗi máy chủ", body = ApiResponseEmpty)
    )
)]
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

/// Cập nhật thông tin của một todo
///
/// Endpoint này cho phép cập nhật tiêu đề, mô tả và trạng thái hoàn thành của một todo.
#[utoipa::path(
    patch,
    path = "/api/v1/todos/{uuid}",
    tag = "todos",
    params(
        ("uuid" = String, Path, description = "UUID của todo cần cập nhật")
    ),
    request_body = UpdateTodoRequest,
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Todo được cập nhật thành công", body = ApiResponseTodoResponse),
        (status = 401, description = "Không được xác thực", body = ApiResponseEmpty),
        (status = 404, description = "Không tìm thấy todo", body = ApiResponseEmpty),
        (status = 500, description = "Lỗi máy chủ", body = ApiResponseEmpty)
    )
)]
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

/// Xóa một todo
///
/// Endpoint này cho phép xóa một todo dựa trên UUID.
#[utoipa::path(
    delete,
    path = "/api/v1/todos/{uuid}",
    tag = "todos",
    params(
        ("uuid" = String, Path, description = "UUID của todo cần xóa")
    ),
    security(
        ("bearer_auth" = [])
    ),
    responses(
        (status = 200, description = "Todo được xóa thành công", body = ApiResponseDeleteTodoResponse),
        (status = 401, description = "Không được xác thực", body = ApiResponseEmpty),
        (status = 404, description = "Không tìm thấy todo", body = ApiResponseEmpty),
        (status = 500, description = "Lỗi máy chủ", body = ApiResponseEmpty)
    )
)]
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
