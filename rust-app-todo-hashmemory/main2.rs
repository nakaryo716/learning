use anyhow::Context;
use axum::{
    http::StatusCode,
    Router,Json,
    routing::{get, post, patch, delete},
    response::{IntoResponse, Html},
    extract::{Extension, Path},
};
use serde::{Serialize, Deserialize};
use std::{
    net::SocketAddr,
    collections::HashMap,
    env,
    sync::{Arc, RwLock, RwLockWriteGuard, RwLockReadGuard},
};
use thiserror::Error;

#[tokio::main]
async fn main() {
    let repository = TodoRepositoryForMemory::new();
    let app = create_app(repository);

    let addr = SocketAddr::from(([0,0,0,0],3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}

fn create_app<T: TodoRepository>(repository: T) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/todos", post(create_todo::<T>).get(all_todo::<T>))
        .route("/todos/:id", get(find_todo::<T>).patch(update_todo::<T>).delete(delete_todo::<T>))
        .layer(Extension(Arc::new(repository)))
}










// handlers
async fn index() -> Html<&'static str> {
    Html("
        <h1>Hello axum server!</h1>
        <p>This is home page</p>
    ")
}
async fn create_todo<T: TodoRepository>(
    Json(payload): Json<CreateTodo>,
    Extension(reposiotry): Extension<Arc<T>>,
) -> impl IntoResponse {
    let todo = reposiotry.create(payload);

    (StatusCode::CREATED, Json(todo))
}

async fn find_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(reposiotry): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = reposiotry.find(id).or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(todo)))
}

async fn all_todo<T: TodoRepository>(
    Extension(reposiotry): Extension<Arc<T>>,
) -> impl IntoResponse {
    let todos = reposiotry.all();

    (StatusCode::OK, Json(todos))
}

async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Json(payload): Json<UpdateTodo>,
    Extension(reposiotry): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = reposiotry.update(id, payload).or(Err(StatusCode::NOT_FOUND))?;

    Ok((StatusCode::OK, Json(todo)))
}

async fn delete_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(reposiotry): Extension<Arc<T>>
) -> Result<impl IntoResponse, StatusCode> {
    reposiotry.delete(id).or(Err(StatusCode::NOT_FOUND))?;

    Ok(())
}










// repositorys

#[derive(Debug,Error)]
enum RepositoryError {
    #[error("NotFound id is {0}")]
    NotFound(i32),
}
pub trait TodoRepository: Clone + std::marker::Sync + std::marker::Send + 'static {
    fn create(&self, payload: CreateTodo) -> Todo;
    fn find(&self, id: i32) -> anyhow::Result<Todo>;
    fn all(&self) -> Vec<Todo>;
    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo>;
    fn delete(&self, id: i32) -> anyhow::Result<()>;
}


#[derive(Debug, Serialize, Deserialize, Clone,)]
pub struct Todo {
    id: i32,
    text: String,
    completed: bool,
}

impl Todo{
    pub fn new(id: i32, text: String) -> Self {
        Self {
            id,
            text,
            completed: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateTodo {
    text: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateTodo {
    text: Option<String>,
    completed: Option<bool>,
}

type TodoDatas = HashMap<i32,Todo>;

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

    fn write_ref(&self) -> RwLockWriteGuard<TodoDatas>{
        self.store.write().unwrap()
    }

    fn read_ref(&self) -> RwLockReadGuard<TodoDatas> {
        self.store.read().unwrap()
    }
}

impl TodoRepository for TodoRepositoryForMemory {
    fn create(&self, payload: CreateTodo) -> Todo {
        let mut store = self.write_ref();

        let id = store.len() as i32;
        let todo = Todo::new(id, payload.text);

        store.insert(id, todo.clone());
        todo
    }

    fn find(&self, id: i32) -> anyhow::Result<Todo> {
        let store = self.read_ref();

        let todo = store.get(&id).map(|todo| todo.clone()).ok_or(RepositoryError::NotFound(id))?;  
        Ok(todo)
    }

    fn all(&self) -> Vec<Todo> {
        let store = self.read_ref();
        
        let mut todos = Vec::new();
        
        for element in store.values(){
            let push_element = element.clone();
            todos.push(push_element);
        }

        todos
    }

    fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<Todo> {
        let mut store = self.write_ref();

        let todo = store
            .get(&id)
            .map(|todo| todo.clone())
            .ok_or(RepositoryError::NotFound(id))?;

        let text = if payload.text.is_none() {
            todo.text
        } else {
            payload.text.unwrap()
        };

        let completed = if payload.completed.is_none() {
            todo.completed
        }else {
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
        let mut store = self.write_ref();
        store
            .remove(&id)
            .ok_or(RepositoryError::NotFound(id))?;

        Ok(())
    }
}