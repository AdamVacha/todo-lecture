use axum::{http::StatusCode, routing::get};
use std::env;

use axum::{
    extract::{self, State},
    response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// Simple program to track todo items
#[derive(Debug, Serialize, Deserialize)]
struct Todo {
    title: String,
    msg: String,
}

impl Todo {
    async fn add(conn: SqlitePool, todo: Todo) -> Result<(), ()> {
        let result = sqlx::query!(
            "insert into todos (title, msg) values ($1, $2)",
            todo.title,
            todo.msg
        )
        .execute(&conn)
        .await;
        if result.unwrap().rows_affected() != 1 {
            return Err(());
        }
        Ok(())
    }
    async fn list(conn: SqlitePool) -> Result<Vec<Todo>, ()> {
        let result = sqlx::query_as!(Todo, "select * from todos")
            .fetch_all(&conn)
            .await;
        if let Err(_) = result {
            println!("error reading from todos");
            return Err(());
        }
        Ok(result.unwrap())
    }
    async fn update(conn: SqlitePool, todo: Todo) -> Result<(), ()> {
        let result = sqlx::query!(
            "update todos set title = $1, msg = $2 where title = $1",
            todo.title,
            todo.msg
        )
        .execute(&conn)
        .await;
        if result.unwrap().rows_affected() != 1 {
            return Err(());
        }
        Ok(())
    }
    async fn delete(conn: SqlitePool, title: &String) -> Result<(), ()> {
        let result = sqlx::query!("delete from todos where title = $1", title)
            .execute(&conn)
            .await;
        if result.unwrap().rows_affected() != 1 {
            return Err(());
        }
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv::dotenv().ok();

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("DATABASE_URL missing"))
        .await;

    let Ok(pool) = pool else {
        eprintln!("cannot connect to db");
        return;
    };

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migrations failed to run");

    // build our application with a route
    let app = Router::new()
        .route("/health", get(health))
        .route(
            "/todos",
            get(get_todos)
                .put(put_todos)
                .patch(patch_todos)
                .delete(delete_todos),
        )
        .with_state(pool);
    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
// basic handler that responds with a static string
async fn health() -> impl IntoResponse {
    StatusCode::OK
}

async fn get_todos(State(pool): State<SqlitePool>) -> impl IntoResponse {
    let todos = Todo::list(pool).await.unwrap();
    Json(todos)
}

async fn put_todos(
    State(pool): State<SqlitePool>,
    extract::Json(todo): extract::Json<Todo>,
) -> impl IntoResponse {
    Todo::add(pool, todo).await.unwrap()
}

async fn patch_todos(
    State(pool): State<SqlitePool>,
    extract::Json(todo): extract::Json<Todo>,
) -> impl IntoResponse {
    Todo::update(pool, todo).await.unwrap()
}

async fn delete_todos(
    State(pool): State<SqlitePool>,
    extract::Json(todo): extract::Json<Todo>,
) -> impl IntoResponse {
    Todo::delete(pool, &todo.title).await.unwrap()
}
