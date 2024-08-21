use actix_multipart::Multipart;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use csv::ReaderBuilder;
use futures_util::StreamExt as _;
use serde::Serialize;
use std::io::Cursor;
// use futures::StreamExt;
use serde::Deserialize;
use std::io::{BufReader, BufWriter};
use tokio_postgres::{Client, Error, NoTls}; 
#[derive(Debug, Deserialize, Serialize)] 
struct Record {
    Email: String, 
    Username: String,
    Points: i32,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/api")]
async fn api(mut payload: Multipart) -> actix_web::Result<HttpResponse> {
    // Iterate over the multipart stream
    let mut results: Vec<Record> = Vec::new();
    while let Some(item) = payload.next().await {
        let mut field = item?;
        // Field in turn is stream of *Bytes* object
        let mut data = web::BytesMut::new();
        while let Some(chunk) = field.next().await {
            let chunk = chunk.unwrap();
            data.extend_from_slice(&chunk);
        }
        // Parse the CSV content
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(Cursor::new(&data));
        for result in reader.deserialize() {
            let record: Record = result.unwrap();
            results.push(record);
        }
    }
    // Connect to the database
    let (client, connection) = tokio_postgres::connect(
        "host=127.0.0.1 user=username dbname=postgres password=password",
        NoTls,
    )
    .await
    .unwrap();

    // Spawn the connection to run in the background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    for record in &results {
        // Check if the email exists in the RANKINGS table
        let row = client
            .query_opt(
                "SELECT points FROM RANKINGS WHERE email = $1",
                &[&record.Email],
            )
            .await
            .unwrap();

        if let Some(row) = row {
            // Email exists, update the points
            let current_points: i32 = row.get(0);
            let new_points = current_points + record.Points;
            client
                .execute(
                    "UPDATE RANKINGS SET points = $1 WHERE email = $2",
                    &[&new_points, &record.Email],
                )
                .await
                .unwrap();
            println!("Updated points for {}", record.Email);
        } else {
            // Email does not exist, insert a new record
            client
                .execute(
                    "INSERT INTO RANKINGS (email, username, points) VALUES ($1, $2, $3)",
                    &[&record.Email, &record.Username, &record.Points],
                )
                .await
                .unwrap();
            println!("Inserted new record for {}", record.Email);
        }
    }
    Ok(HttpResponse::Ok().json(results))
}

#[get("/dashboard")]
async fn dashboard(query: web::Query<PagingParams>) -> impl Responder {
    let page: i64 = query.page.unwrap_or(1).try_into().unwrap();
    let per_page: i64 = query.per_page.unwrap_or(10).try_into().unwrap();

    let (client, connection) = tokio_postgres::connect(
        "host=127.0.0.1 user=username dbname=postgres password=password",
        NoTls,
    )
    .await
    .unwrap();

    // Spawn the connection to run in the background
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Query to fetch items and total count from the database
    let offset: i64 = (page - 1) * per_page;
    let rows = client
        .query(
            "SELECT * FROM RANKINGS ORDER BY Points DESC LIMIT $1 OFFSET $2 ",
            &[&per_page, &offset],
        )
        .await
        .unwrap();
    let total_items_row = client
        .query_one("SELECT COUNT(*) FROM rankings", &[])
        .await
        .unwrap();

    let total_items: i64 = total_items_row.get(0);
    let items: Vec<Record> = rows
        .iter()
        .map(|row| Record {
            Email: row.get(0),
            Username: row.get(1),
            Points: row.get(2),
        })
        .collect();

    let html = render_html(&items, page, per_page, total_items);
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[derive(Deserialize)]
struct PagingParams {
    page: Option<usize>,
    per_page: Option<usize>,
}

fn render_html(items: &[Record], page: i64, per_page: i64, total_items: i64) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html>");
    html.push_str("<html lang=\"en\"><head><meta charset=\"UTF-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">");
    html.push_str("<title>Rankings Dashboard</title>");
    html.push_str(
        "<style>
        body { font-family: Arial, sans-serif; padding: 20px; }
        h1 { color: #333; }
        ol { padding: 0; }
        li { padding: 5px 0; }
        .pagination { margin-top: 20px; }
        .pagination button { padding: 5px 10px; margin-right: 5px; }
        .disabled { color: #aaa; cursor: not-allowed; }
    </style>",
    );
    html.push_str("</head><body>");
    html.push_str("<h1>Rankings Dashboard</h1><ol>");

    for item in items {
        html.push_str(&format!(
            "<li>{}. {} - {}</li>",
            item.Email, item.Username, item.Points
        ));
    }

    html.push_str("</ol><div class=\"pagination\">");

    if page > 1 {
        html.push_str(&format!(
            "<a href=\"/dashboard?page={}&per_page={}\"><button>Previous</button></a>",
            page - 1,
            per_page
        ));
    } else {
        html.push_str("<button class=\"disabled\">Previous</button>");
    }

    if page * per_page < total_items {
        html.push_str(&format!(
            "<a href=\"/dashboard?page={}&per_page={}\"><button>Next</button></a>",
            page + 1,
            per_page
        ));
    } else {
        html.push_str("<button class=\"disabled\">Next</button>");
    }

    html.push_str("</div></body></html>");
    html
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(dashboard).service(api))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
