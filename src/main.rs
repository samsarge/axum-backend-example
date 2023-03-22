use axum::{
    extract::{State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{Mutex};

use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

extern crate dotenv;

use dotenv::dotenv;

#[derive(Clone, Debug)]
struct AppState {
    users: Vec<DbUser>,
    pg_pool: Pool<Postgres>
}

#[derive(Clone, Serialize, Debug)]
struct DbUser {
    id: u64,
    username: String
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL env var missing");

    // call Pool::acquire to get a connection from the pool; when the connection
    // is dropped it will return to the pool so it can be reused.
    // You can also pass &Pool directly anywhere an Executor
    // is required; this will automatically checkout a connection for you.

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Cannot connect to database");

    // load users into memory on app start
    let shared_state = Arc::new(
        Mutex::new(
            AppState {
                users: vec![],
                pg_pool: pool
            }
        )
    );
    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user))
        .route("/users", get(all_users))
        .with_state(shared_state);
        
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> impl IntoResponse {
    let response = json!(
        {
            "message": "Mutable shared state success"
        }
    );
    (StatusCode::OK, Json(response))
}

async fn all_users(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.clone();
    let locked_state = state.lock().await;
    let users = locked_state.users.to_owned();

    (StatusCode::OK, Json(users))
}

async fn create_user(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<CreateUserRequest>,
) -> impl IntoResponse {
    // acquire the lock at the beginning and ensure it doesnt
    // get freed until the end of this function to keep it atomic
    let state = state.clone();
    let mut locked_state = state.lock().await;

    // let db = Pool::acquire(&locked_state.pg_pool).await.unwrap();
    // tokio::task::spawn(async move {
        // save user to db
    // });
    

    let count = locked_state.users.len() as u64;
    let users = &mut locked_state.users;
    let user = DbUser {
        id: count.to_owned(),
        username: payload.username,
    };
    let response = user.to_owned();
    users.push(user);

    (StatusCode::CREATED, Json(response))
}
