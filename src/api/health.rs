use rouille;
use serde::Serialize;

#[derive(Serialize)]
struct Response {
    message: String,
}

pub fn serve() -> rouille::Response {
    rouille::Response::json(&Response {
        message: String::from("works fine"),
    })
}