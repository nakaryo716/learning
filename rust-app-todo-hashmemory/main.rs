use axum::{
    http::StatusCode,
    response::IntoResponse,
    Json,
    Router,
    routing::{get,post,patch,delete},
    extract::{Extension, Path},

};
use std::{net::SocketAddr, sync::{RwLockWriteGuard, RwLockReadGuard}};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    env
};
use thiserror::Error;
use anyhow::Context;
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() {
    let repository = TodoRepositoryForMemory::new();
    let app = create_app(repository);

    let addr = SocketAddr::from(([0,0,0,0],3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}

fn create_app<T: TodoRepository>(repository: T) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/todos", post(create_todo::<T>).get(all_todo::<T>))
        .route("/todos/:id", get(find_todo::<T>).patch(update_todo::<T>).delete(delete_todo::<T>))
        .layer(Extension(Arc::new(repository)))
}


// ハンドラー
async fn index() -> &'static str {
    "Hello Axum Server!"
}

async fn create_todo<T: TodoRepository>(
    Json(payload): Json<CreateTodo>,
    Extension(repository): Extension<Arc<T>>
    ) -> impl IntoResponse {
        let todo = repository.create(payload);

        (StatusCode::CREATED, Json(todo))
}

async fn find_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository.find(id).ok_or(StatusCode::NOT_FOUND)?;
    Ok((StatusCode::OK, Json(todo)))
    
}

async fn all_todo<T: TodoRepository>(
    Extension(repository): Extension<Arc<T>>
) -> impl IntoResponse {
    let todo = repository.all();

    (StatusCode::OK, Json(todo))
}

async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTodo>,
    Extension(repository): Extension<Arc<T>>
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository.update(id, payload).or(Err(StatusCode::NOT_FOUND))?;
    Ok((StatusCode::OK, Json(todo)))
}

async fn delete_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>
) -> Result<impl IntoResponse, StatusCode> {
    repository.delete(id).or(Err(StatusCode::NOT_FOUND))?;

    Ok(())
}


// リポジトリ
#[derive(Debug, Error)]
enum RepositoryError {
    #[error("NotFound, id is {0}")]
    NotFound(i32),
}

pub trait TodoRepository: Clone + std::marker::Sync + std::marker::Send + 'static {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> Option<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo, ) -> anyhow::Result<Todo>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct CreateTodo {
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

impl Todo {
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

type TodoDatas = HashMap<i32, Todo>;

#[derive(Debug, Clone)]
pub struct TodoRepositoryForMemory {
    store: Arc<RwLock<TodoDatas>>,
}

impl TodoRepositoryForMemory {
    pub fn new() -> Self {
        TodoRepositoryForMemory {
            store: Arc::default(), 
        }
    }

    fn write_store_ref(&self) -> RwLockWriteGuard<TodoDatas> {
        self.store.write().unwrap()
    }

    fn read_store_ref(&self) -> RwLockReadGuard<TodoDatas> {
        self.store.read().unwrap()
    }
}

impl TodoRepository for TodoRepositoryForMemory {
    fn create(&self, payload: CreateTodo) -> Todo {
        let mut store = self.write_store_ref();

        let id = store.len() as i32;

        let todo = Todo::new(id, payload.text);

        store.insert(id, todo.clone());
        todo
    }

    fn find(&self, id: i32) -> Option<Todo> {
        let store = self.read_store_ref();
        let todo = store.get(&id).map(|todo| todo.clone());
        todo
    }

    fn all(&self) -> Vec<Todo> {
        let store = self.read_store_ref();
        let mut v = Vec::new();
        for elment in store.values(){
            let value = elment.clone();
            v.push(value);
        }
        v
    }

    fn update(&self, id: i32, payload: UpdateTodo, ) -> anyhow::Result<Todo> {
        let mut store = self.write_store_ref();

        let todo = store
            .get(&id)
            .ok_or(RepositoryError::NotFound(id))
            .unwrap().clone();

        let text = if payload.text.is_none(){
            todo.text
        } else{
            payload.text.unwrap()
        };

        let completed = if payload.completed.is_none() {
            todo.completed
        } else {
            payload.completed.unwrap()
        };

        let todo = Todo {
            id,
            text,
            completed,
        };

        store.insert(id, todo.clone());
        Ok(todo)

    }

    fn delete(&self, id: i32) -> anyhow::Result<()> {
        let mut store = self.write_store_ref();

        store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
        Ok(())
    }
    }