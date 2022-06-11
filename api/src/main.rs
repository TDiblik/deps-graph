#[macro_use]
extern crate lazy_static;
extern crate dotenv;

use actix_files::Files;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use redis_graph::*;

mod constants;

/* TODO:
    -- Mock third party APIs for development
    -- Make automatic program that uploads data from third party APIs dumps into redis
        making it unnecessary to scrape, using scraping as a last resort
        examples:
        https://crates.io/data-access
        https://docs.npmjs.com/policies/crawlers
*/

#[post("/moto_gp/create")]
async fn moto_gp_post() -> impl Responder {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_connection().unwrap();

    let _: GraphResultSet = con
        .graph_query(
            "my_graph",
            "CREATE (:Rider {name:'Valentino Rossi'})-[:rides]->(:Team {name:'Yamaha'})",
        )
        .unwrap();

    HttpResponse::Ok().body("Since you're seeing this, everything went well")
}

#[get("/moto_gp/get")]
async fn moto_gp_get() -> impl Responder {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let mut con = client.get_async_connection().await.unwrap();

    let results: GraphResultSet = con
        .graph_ro_query(
            "my_graph",
            "MATCH (rider:Rider)-[:rides]->(:Team {name:'Yamaha'}) RETURN rider",
        )
        .await
        .unwrap();

    println!("{:?}", results);

    HttpResponse::Ok().body("Results are in console")
}

mod models {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    pub struct ExampleArgs {
        pub name: String,
        pub version: String,
    }
}

use crate::models::ExampleArgs;

#[get("/package/example")]
async fn get_package_example(args: web::Query<ExampleArgs>) -> impl Responder {
    let client = reqwest::Client::new();
    let resp = client
        .get(format!(
            "https://crates.io/api/v1/crates/{name}/{version}/dependencies",
            name = args.name,
            version = args.version
        ))
        .header("User-Agent", &*constants::USER_AGENT_IDENTIFIER)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    println!("{:#?}", resp);

    HttpResponse::Ok().body("Results are in console")
}

static SERVER_IP: &str = "127.0.0.1";
static SERVER_PORT: u16 = 8080;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("----- Starting initiation sequence -----");

    // Load .env
    println!("- Load .env starting -");
    dotenv::dotenv().ok();
    println!("User-Agent: {:?}", &*constants::USER_AGENT_IDENTIFIER);
    println!("- Load .env done -");

    println!("----- Initiation sequence completed -----");

    println!(
        "----- Starting server at: {:?}:{:?} -----",
        SERVER_IP, SERVER_PORT
    );
    HttpServer::new(|| {
        App::new()
            // .service(hello)
            .service(moto_gp_get)
            .service(moto_gp_post)
            .service(get_package_example)
            // This must be always last
            .service(Files::new("/", "./react-output/").index_file("index.html"))
    })
    .bind((SERVER_IP, SERVER_PORT))?
    .run()
    .await
}
