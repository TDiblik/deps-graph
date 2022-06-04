use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use redis_graph::*;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(moto_gp_get)
            .service(moto_gp_post)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
