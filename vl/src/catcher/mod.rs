use serde::Deserialize;

pub mod auth;
pub mod link;
pub mod view;

pub static API_VIEW: &str = "http://api.bilibili.com/x/web-interface/view";
pub static API_PLAYURL: &str = "http://api.bilibili.com/x/player/playurl";

#[derive(Deserialize, Debug)]
struct Response<T> {
    code: isize,
    message: String,
    data: T,
}
