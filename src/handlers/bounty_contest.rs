use std::io::Cursor;

use crate::{
    models::bounty_contest::BountyContestLeaderboard,
    serialize::{error::AppError, PaginationReq},
    services::bounty_contest::BountyContestSrv,
};

use actix_multipart::Multipart;
use actix_web::{web, HttpRequest, HttpResponse};
use csv::ReaderBuilder;
use futures_util::StreamExt as _;

const SECRET_KEY: &str = "Utx0-Global-Bounty-Contest";
async fn submit_points(
    mut payload: Multipart,
    bounty_contest_srv: web::Data<BountyContestSrv>,
    http_req: HttpRequest,
) -> Result<HttpResponse, AppError> {
    let secret = http_req
        .headers()
        .get("SECRET_KEY")
        .map(|x| x.to_str().unwrap())
        .unwrap();

    if secret == SECRET_KEY {
        let mut results: Vec<BountyContestLeaderboard> = Vec::new();
        while let Some(item) = payload.next().await {
            if let Ok(i) = item {
                let mut field = i;
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
                    let record: BountyContestLeaderboard = result.unwrap();
                    results.push(record);
                }
            }
        }

        match bounty_contest_srv.process_points(results.clone()).await {
            Ok(_) => Ok(HttpResponse::Ok().json(results)),
            Err(err) => Err(err),
        }
    } else {
        return Err(AppError::new(400).message("Missing secret key"));
    }
}

async fn request_dashboard(
    pagination: web::Query<PaginationReq>,
    bounty_contest_srv: web::Data<BountyContestSrv>,
) -> Result<HttpResponse, AppError> {
    let results = bounty_contest_srv
        .get_dashboard(pagination.page, pagination.limit)
        .await
        .unwrap();
    Ok(HttpResponse::Ok().json(results))
}

pub fn route(conf: &mut web::ServiceConfig) {
    conf.service(
        web::scope("/bounty-contest")
            .route("/points", web::post().to(submit_points))
            .route("/dashboard", web::get().to(request_dashboard)),
    );
}
