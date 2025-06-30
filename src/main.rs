use axum::{
    extract::{Path, State},
    routing::{get, post, delete},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{sync::{Arc, Mutex}, collections::HashMap, net::SocketAddr};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    id: String,
    task: String,
    done: bool,
}

// App state with shared in-memory storage
type Db = Arc<Mutex<HashMap<String, Todo>>>;

#[tokio::main]
async fn main() {
    let db: Db = Arc::new(Mutex::new(HashMap::new()));

    let app = Router::new()
        .route("/todos", get(list_todos).post(add_todo))
        .route("/todos/:id", delete(delete_todo))
        .with_state(db.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://{}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn list_todos(State(db): State<Db>) -> Json<Vec<Todo>> {
    let db = db.lock().unwrap();
    let todos = db.values().cloned().collect();
    Json(todos)
}

#[derive(Debug, Deserialize)]
struct CreateTodo {
    task: String,
}

async fn add_todo(
    State(db): State<Db>,
    Json(payload): Json<CreateTodo>,
) -> Json<Todo> {
    let id = Uuid::new_v4().to_string();
    let todo = Todo {
        id: id.clone(),
        task: payload.task,
        done: false,
    };

    db.lock().unwrap().insert(id.clone(), todo.clone());
    Json(todo)
}

async fn delete_todo(
    Path(id): Path<String>,
    State(db): State<Db>,
) -> Json<String> {
    let mut db = db.lock().unwrap();
    if db.remove(&id).is_some() {
        Json(format!("Deleted todo {}", id))
    } else {
        Json("Todo not found".to_string())
    }
}
