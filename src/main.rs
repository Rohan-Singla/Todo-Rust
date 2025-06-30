use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use uuid::Uuid;
use tracing_subscriber;

// MODELS
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Todo {
    id: Uuid,
    title: String,
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    title: String,
}

// APP STATE
type SharedState = Arc<Mutex<Vec<Todo>>>;

// HANDLERS
async fn get_todos(State(state): State<SharedState>) -> impl IntoResponse {
    let todos = state.lock().unwrap();
    Json(todos.clone())
}

async fn create_todo(
    State(state): State<SharedState>,
    Json(payload): Json<CreateTodo>,
) -> impl IntoResponse {
    let mut todos = state.lock().unwrap();
    let todo = Todo {
        id: Uuid::new_v4(),
        title: payload.title,
    };
    todos.push(todo.clone());
    (StatusCode::CREATED, Json(todo))
}

async fn delete_todo(State(state): State<SharedState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    let mut todos = state.lock().unwrap();
    let len_before = todos.len();
    todos.retain(|todo| todo.id != id);
    let deleted = len_before != todos.len();

    if deleted {
        (StatusCode::OK, "Deleted").into_response()
    } else {
        (StatusCode::NOT_FOUND, "Todo not found").into_response()
    }
}

// MAIN
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let state: SharedState = Arc::new(Mutex::new(Vec::new()));

    let app = Router::new()
        .route("/todos", get(get_todos).post(create_todo))
        .route("/todos/{id}", delete(delete_todo))
        .with_state(state);

    println!("Listening at port 3000");
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}