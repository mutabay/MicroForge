mod db;
mod models;

use axum::{
    extract::Path,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use db::DB_POOL;
use models::{NewUser, User};
use sqlx::Row;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    db::init().await;

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user).delete(delete_user))
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 8001));
    tracing::info!("🚀 user_service listening at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> &'static str {
    "User Service – v1"
}

async fn health() -> &'static str {
    "OK"
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "List all users", body = [User])
    )
)]
async fn list_users() -> impl IntoResponse {
    let rows = sqlx::query("SELECT id, name, email FROM users")
        .fetch_all(&*DB_POOL)
        .await
        .unwrap();

    let users: Vec<User> = rows
        .into_iter()
        .map(|r| User {
            id: r.get::<String, _>("id").parse().unwrap(),
            name: r.get("name"),
            email: r.get("email"),
        })
        .collect();

    Json(users)
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found")
    )
)]
async fn get_user(Path(id): Path<Uuid>) -> impl IntoResponse {
    let user = sqlx::query("SELECT id, name, email FROM users WHERE id = ?")
        .bind(id.to_string())
        .fetch_optional(&*DB_POOL)
        .await
        .unwrap();

    match user {
        Some(row) => Json(User {
            id,
            name: row.get("name"),
            email: row.get("email"),
        })
        .into_response(),
        None => axum::http::StatusCode::NOT_FOUND.into_response(),
    }
}

#[utoipa::path(
    post,
    path = "/users",
    request_body = NewUser,
    responses(
        (status = 201, description = "User created", body = User)
    )
)]
async fn create_user(Json(payload): Json<NewUser>) -> impl IntoResponse {
    let user = User {
        id: Uuid::new_v4(),
        name: payload.name,
        email: payload.email,
    };

    sqlx::query("INSERT INTO users (id, name, email) VALUES (?, ?, ?)")
        .bind(user.id.to_string())
        .bind(&user.name)
        .bind(&user.email)
        .execute(&*DB_POOL)
        .await
        .unwrap();

    (axum::http::StatusCode::CREATED, Json(user))
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = NewUser,
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User updated", body = User),
        (status = 404, description = "User not found")
    )
)]
async fn update_user(Path(id): Path<Uuid>, Json(payload): Json<NewUser>) -> impl IntoResponse {
    let result = sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
        .bind(&payload.name)
        .bind(&payload.email)
        .bind(id.to_string())
        .execute(&*DB_POOL)
        .await
        .unwrap();

    if result.rows_affected() == 1 {
        Json(User {
            id,
            name: payload.name,
            email: payload.email,
        })
        .into_response()
    } else {
        axum::http::StatusCode::NOT_FOUND.into_response()
    }
}

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found")
    )
)]
async fn delete_user(Path(id): Path<Uuid>) -> impl IntoResponse {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id.to_string())
        .execute(&*DB_POOL)
        .await
        .unwrap();

    if result.rows_affected() == 1 {
        axum::http::StatusCode::NO_CONTENT
    } else {
        axum::http::StatusCode::NOT_FOUND
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list_users,
        get_user,
        create_user,
        update_user,
        delete_user
    ),
    components(schemas(User, NewUser)),
    tags(
        (name = "Users", description = "User CRUD endpoints")
    )
)]
pub struct ApiDoc;