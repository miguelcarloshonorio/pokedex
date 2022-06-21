use rouille;
use serde::Serialize;

#[derive(Serialize)]
struct Response {
    message: String,
}

pub fn serve(_req: &rouille::Request) -> rouille::Response {
    rouille::Response::json(&Response {
        message: String::from("Pokemon created!"),
    })
}