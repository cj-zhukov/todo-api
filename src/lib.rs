use std::sync::Arc;

// pub type Result<T> = core::result::Result<T, Error>;
// pub type Error = Box<dyn std::error::Error>;
use anyhow::Result;

pub mod config;

use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::routing::post;
use axum::Json;
use axum::{Router, routing::get};
use axum::response::IntoResponse;
use axum::extract::{Path, State};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use sqlx::PgPool;
use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

const MAX_ROWS: u32 = 10;
const TODO_TABLE_NAME: &str = "todos";

// pub type AppStateRef = Arc<AppState>;
// pub struct AppState {
//     pub db: PgPool,
// }

// impl AppState {
//     pub fn new(db: PgPool) -> AppStateRef {
//         Arc::new(Self { db })
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Todo {
    id: i64,
    body: String,
    completed: bool,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Deserialize, Clone)]
pub struct CreateTodo {
    body: String,
}

impl CreateTodo {
    pub fn body(&self) -> &str {
        self.body.as_ref()
    }
}

#[derive(Deserialize, Clone)]
pub struct UpdateTodo {
    body: String,
    completed: bool,
}

impl UpdateTodo {
    pub fn body(&self) -> &str {
        self.body.as_ref()
    }

    pub fn completed(&self) -> bool {
        self.completed
}
}

impl Todo {
    fn table_name() -> String {
        TODO_TABLE_NAME.to_string()
    }

    pub async fn list(pool: PgPool) -> Result<Vec<Todo>> {
        let sql = format!("select * from {} limit {};", Self::table_name(), MAX_ROWS);
        let query = sqlx::query_as::<_, Self>(&sql);
        let data = query.fetch_all(&pool).await?;

        Ok(data)
    }

    pub async fn read(pool: PgPool, id: i64) -> Result<Todo> {
        let sql = format!("select * from {} where id = ?", Self::table_name());
        let query = sqlx::query_as::<_, Self>(&sql);
        let data = query
            .bind(id)
            .fetch_one(&pool)
            .await?;

        Ok(data)
    }

    pub async fn query_table(pool: PgPool) -> Result<()> {
        let sql = format!("select * from {} limit {};", Self::table_name(), MAX_ROWS);
        let query = sqlx::query_as::<_, Self>(&sql);
        let data = query.fetch_all(&pool).await?;
        println!("{:?}", data);

        Ok(())
    }

    pub async fn query_table_to_json(pool: PgPool) -> Result<String> {
        let sql = format!("select * from {} limit {};", Self::table_name(), MAX_ROWS);
        let query = sqlx::query_as::<_, Self>(&sql);
        let data = query.fetch_all(&pool).await?;
        let res = serde_json::to_string(&data)?;
        
        Ok(res)
    }
}

impl Todo {
    pub async fn create(pool: PgPool, new_todo: CreateTodo) -> Result<Todo> {
        let sql = format!("insert into {} (body) values (?) returning *;", Self::table_name());
        let query = sqlx::query_as::<_, Self>(&sql);
        let data = query
            .bind(new_todo.body())
            .fetch_one(&pool)
            .await?;
        
        Ok(data)
    }

    pub async fn update(pool: PgPool, id: i64, update_todo: UpdateTodo) -> Result<Todo> {
        let sql = format!("
            update {} set body = ?, completed = ?,
            updated_at = datetime('now') where id = ? returning *;", Self::table_name());
        let query = sqlx::query_as::<_, Self>(&sql);
        let data = query
            .bind(update_todo.body())
            .bind(update_todo.completed())
            .bind(id)
            .fetch_one(&pool)
            .await?;
        
        Ok(data)
    }

    pub async fn delete(pool: PgPool, id: i64) -> Result<()> {
        let sql = format!("delete from {} where id = ?;", Self::table_name());
        let query = sqlx::query_as::<_, Self>(&sql);
        let _data = query
            .bind(id)
            .fetch_one(&pool)
            .await?;
        
        Ok(())
    }
}

pub async fn todo_list(State(pool): State<PgPool>) -> Result<impl IntoResponse, Json<Vec<Todo>>> {
    let data = Todo::list(pool).await.map(Json::from).unwrap();

    Ok(data)
}


pub async fn todo_read(State(pool): State<PgPool>, Path(id): Path<i64>) -> Result<Json<Todo>> {
    let res = Todo::read(pool, id).await.map(Json::from)?;

    Ok(res)
}

pub async fn todo_create(State(pool): State<PgPool>, Json(new_todo): Json<CreateTodo>) -> Result<Json<Todo>> {
    let res = Todo::create(pool, new_todo).await.map(Json::from)?;

    Ok(res)
}

pub async fn todo_update(State(pool): State<PgPool>, Path(id): Path<i64>, Json(updated_todo): Json<UpdateTodo>) -> Result<Json<Todo>> {
    let res = Todo::update(pool, id, updated_todo).await.map(Json::from)?;

    Ok(res)
}

pub async fn todo_delete(State(pool): State<PgPool>, Path(id): Path<i64>) -> Result<()> {
    Todo::delete(pool, id).await?;

    Ok(())
}

pub fn init_tracing() {
    let rust_log = std::env::var(EnvFilter::DEFAULT_ENV)
        .unwrap_or_else(|_| "sqlx=info,tower_http=debug,info".to_string());

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into()) 
                .parse_lossy(rust_log),
        )
        .init()
}

pub async fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/alive", get(|| async { "ok" }))
        .route("/ready", get(ping)) 
        .route("/todos", get(todo_list))
        .with_state(pool)
        .layer(CorsLayer::new().allow_methods(Any).allow_origin(Any))
        .layer(TraceLayer::new_for_http())
}

pub async fn ping() -> impl IntoResponse {
    let msg = "foo";

    let json_response = serde_json::json!({
        "status": "success",
        "message": msg
    });

    Json(json_response)
}