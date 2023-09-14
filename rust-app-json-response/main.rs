use axum::{
    http::StatusCode, 
    Router,
    routing::{get, post},
    response::{Html, IntoResponse},
    Json
};
use std::net::SocketAddr;
use serde::{Serialize, Deserialize};

#[tokio::main]
async fn main() {
    let app = create_app();

    let addr = SocketAddr::from(([0,0,0,0],3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


fn create_app() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/user", post(creat_user))
    // ハンドラは非同期関数でないといけない
}

async fn index() -> Html<&'static str> {
    Html("
        <h1>Hello axum</h1>
        <p>This is the index page for test</P>
    ")
}

// IntoResponse 型は　(StatusCode, T)
async fn creat_user(Json(payload): Json<CreateUser>) -> impl IntoResponse {
    let user = User {
        id: payload.username.len() as u64,
        username: payload.username,
    };

    (StatusCode::CREATED, Json(user))

}

#[derive(Debug, Serialize)]
pub struct User {
    id: u64,
    username: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    username: String,
}