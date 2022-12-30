use reqwest::header::HeaderMap;
use serde::{Serialize, Deserialize};
use anyhow::Result;
use serde_json::Value;

use crate::common::CLIENT;

use super::{API_PLAYURL, Response};


#[derive(Serialize, Debug)]
struct LinkReq<'a> {
    bvid: &'a str,
    cid: usize,
    fnval: usize
}

#[derive(Deserialize, Debug)]
pub struct Audio {
    pub id: usize,
    pub base_url: String,
    pub backup_url: Vec<String>,
    pub mime_type: String,
    pub codecs: String
}

#[derive(Deserialize, Debug)]
pub struct Dolby {
    pub audio: Value
}

#[derive(Deserialize, Debug)]
pub struct Flac {
    pub audio: Option<Audio>
}

#[derive(Deserialize, Debug)]
pub struct Dash {
    pub audio: Vec<Audio>,
    pub dolby: Option<Dolby>,
    pub flac: Option<Flac>
}

#[derive(Deserialize, Debug)]
pub struct LinkRsp {
    pub dash: Dash
}

pub async fn api(bvid: &str, cid: usize, fnval: usize, headers: Option<HeaderMap>) -> Result<LinkRsp> {
    let link_req = LinkReq {
        bvid,
        cid,
        fnval
    };
    let response = CLIENT
        .get_struct::<_, _, Response<LinkRsp>>(API_PLAYURL, &link_req, headers)
        .await.unwrap();
    Ok(response.data)
}

#[tokio::test]
async fn api_test() {
    let res = api("BV1fB4y1h76Z", 773130617, 16 | 256, None).await.unwrap();
    // for x in res.dash.audio {
    //     let code = x.id;
    //     let url = x.base_url;
    //     println!("code: {code}, url: {url}");
    // }
    println!("{:?}", res.dash.flac);
}