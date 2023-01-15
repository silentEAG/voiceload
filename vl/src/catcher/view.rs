use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{Response, API_VIEW};
use crate::common::CLIENT;

#[derive(Serialize, Debug)]
pub enum BiliId<'a> {
    #[serde(rename = "bvid")]
    BV(&'a str),
    #[serde(rename = "aid")]
    AV(usize),
}

pub fn get_video_id(id: &str) -> Result<BiliId> {
    let prefix = &id.to_lowercase()[..2];
    match prefix {
        "bv" => Ok(BiliId::BV(id)),
        "av" => Ok(BiliId::AV(id[2..].parse::<usize>()?)),
        _ => Err(anyhow::Error::msg("Avid/Bvid is illegal.")),
    }
}

#[derive(Serialize, Debug)]
struct ViewReq<'a> {
    #[serde(flatten)]
    id: BiliId<'a>,
}

#[derive(Deserialize, Debug)]
pub struct Page {
    // 分P的cid
    pub cid: usize,
    // 分P时长
    pub duration: usize,
}

#[derive(Deserialize, Debug)]
pub struct Owner {
    // UP 主名字
    pub name: String,
    // UP 主头像
    pub face: String,
}

#[derive(Deserialize, Debug)]
pub struct ViewRsp {
    // BV 号
    pub bvid: String,
    // AV 号
    pub aid: usize,
    // 稿件分P总数
    pub videos: usize,
    // 子分区名称
    pub tname: String,
    // 稿件封面图片url
    pub pic: String,
    // 稿件标题
    pub title: String,
    // 稿件发布时间
    pub pubdate: usize,
    // 视频UP主信息
    pub owner: Owner,
    // 稿件总时长(所有分P)
    pub duration: usize,
    // 视频分P列表
    pub pages: Vec<Page>,
}

pub async fn api(id: &str) -> Result<ViewRsp> {
    let video_id = get_video_id(id)?;
    let view_req = ViewReq { id: video_id };

    let response = CLIENT
        .get_struct::<_, _, Response<ViewRsp>>(API_VIEW, &view_req, None)
        .await?;

    if response.code != 0 {
        return Err(anyhow::Error::msg(response.message));
    }

    Ok(response.data)
}

#[tokio::test]
async fn api_test() {
    let res = api("BV12g411r7mB").await.unwrap();
    assert_eq!(
        &res.title,
        "【鹿乃×こはならむ】翻唱《ねぇねぇねぇ（呐呐呐。 ）》"
    );
    println!("{}", res.pic);

    let res = api("av600924585").await.unwrap();
    assert_eq!(&res.owner.name, "影视飓风");
}
