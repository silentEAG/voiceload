use serde::Deserialize;


pub mod auth;
pub mod view;
pub mod link;

pub static API_VIEW: &str = "https://api.bilibili.com/x/web-interface/view";
pub static API_PLAYURL: &str = "https://api.bilibili.com/x/player/playurl";


#[derive(Deserialize, Debug)]
struct Response<T> {
    code: isize,
    message: String,
    data: T,
}