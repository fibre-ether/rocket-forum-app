#[macro_use]
extern crate rocket;

mod helpers;

use dotenv::dotenv;
use helpers::{create_blog_sql, retrieve_blogs_sql, vote_on_blog_sql, BlogIn, UserVoteAction};
use rocket::{
    http::Method,
    serde::json::{json, Json, Value},
    State,
};
use rocket_cors::{AllowedHeaders, AllowedOrigins};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{env, vec};

#[get("/")]
fn index() -> String {
    String::from("Hello world")
}

#[get("/retrieve")]
async fn retrieve_blogs(pool: &State<Pool<Postgres>>) -> Value {
    let blogs = retrieve_blogs_sql(pool).await;
    json!(blogs)
}

#[post("/submit", data = "<blog>")]
async fn submit_blog(blog: Json<BlogIn>, pool: &State<Pool<Postgres>>) -> String {
    create_blog_sql(blog, pool).await;
    "post added".to_string()
}

#[post("/vote", data = "<user_post_action>")]
async fn vote_on_blog(
    user_post_action: Json<UserVoteAction>,
    pool: &State<Pool<Postgres>>,
) -> &str {
    vote_on_blog_sql(user_post_action, pool).await
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    let allowed_origins = AllowedOrigins::All;

    // You can also deserialize this
    let cors = rocket_cors::CorsOptions {
        allowed_origins,
        allowed_methods: vec![Method::Get, Method::Post]
            .into_iter()
            .map(From::from)
            .collect(),
        allowed_headers: AllowedHeaders::All,
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Error while setting cors");

    // let default_post = BlogOut {
    //     added_at: Utc::now(),
    //     author: String::from("new author"),
    //     title: String::from("new title"),
    //     description: String::from("new description"),
    //     votes: 0,
    // };

    let db_url = env::var("POSTGRES_CONNECTION_STRING").expect("cannot find connection string");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .expect("cannot connect to db");

    rocket::build()
        .mount("/", routes![index])
        .mount("/blog", routes![submit_blog, retrieve_blogs, vote_on_blog])
        // .manage::<BlogOut>(default_post)
        .manage::<Pool<Postgres>>(pool)
        .attach(cors)
}
