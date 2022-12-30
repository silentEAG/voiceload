use std::{time::Instant, io::SeekFrom};

use anyhow::Result;
use futures::StreamExt;
use log::trace;
use reqwest::header;
use tokio::fs::File;
use tokio::io::AsyncSeekExt;
use futures::future::join_all;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::IntoUrl;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

use crate::common::CLIENT;


/// 通过 `HEAD` 请求获取文件大小以及是否支持 `ACCEPT_RANGES` 请求
async fn judge(url: &str) -> Result<(u64, bool)> {
    let headers = CLIENT
        .head(url)
        .await?;
    
    let content_length = headers
        .get(header::CONTENT_LENGTH).unwrap().to_str()?.parse::<u64>()?;

    let can_muti = match headers.get(header::ACCEPT_RANGES) {
        None => false,
        Some(v) => {
            matches!(v.to_str(), Ok(v) if v == "bytes")
        }
    };

    Ok((content_length, can_muti))
}


async fn fetch_one(url: &str, mut file: File) -> Result<()> {
    let mut stream = CLIENT
        .get_byte_stream(url, None)
        .await?;
    let mut start: u64 = 0;
    while let Some(chunk) = stream.next().await {
        let mut chunk = chunk?;
        file.seek(SeekFrom::Start(start)).await?;
        start += chunk.len() as u64;
        file.write_all_buf(&mut chunk).await?;
    }
    Ok(())
}

async fn muti_download<U: IntoUrl>(url: U, (mut start, end): (u64, u64), file: Arc<Mutex<File>>) -> Result<()> {
    let mut headers = HeaderMap::new();
    let range = match end {
        u64::MAX => format!("bytes={start}-"),
        _ => format!("bytes={start}-{end}"),
    };
    headers.insert(header::RANGE, HeaderValue::from_str(&range)?);
    let mut stream = CLIENT
        .get_byte_stream(url, Some(headers))
        .await?;

    while let Some(chunk) = stream.next().await {
        let mut chunk = chunk?;
        let mut file = file.lock().await;
        file.seek(SeekFrom::Start(start)).await?;
        start += chunk.len() as u64;
        file.write_all_buf(&mut chunk).await?;
    }
    Ok(())
}   

async fn fetch_muti<U:  IntoUrl>(url: U, content_length: u64, file: File) -> Result<()> {

    let block_num = num_cpus::get() as u64;
    let url = url.into_url()?;

    let mut handles = Vec::with_capacity(block_num as usize);
    let file = Arc::new(Mutex::new(file));
    let block_size = content_length / block_num;
    {
        for i in 0..(block_num - 1) {
            let file = Arc::clone(&file);
            handles.push(tokio::spawn(muti_download(
                url.clone(),
                (block_size * i, block_size * (i + 1) - 1),
                file)));
        }
    }

    let file = Arc::clone(&file);
    handles.push(tokio::spawn(muti_download(
        url.clone(),
        (block_size * (block_num - 1), u64::MAX),
        file)));
    
    let res = join_all(handles).await;
    let error = res.into_iter().flatten().any(|r| r.is_err());

    if error {
        return Err(anyhow::Error::msg("Download file failed."));
    }
    
    Ok(())
}

pub async fn load(url: &str, filename: &str, path: &str) -> Result<()> {
    let (content_length, can_muti) = judge(url).await?;
    let start = Instant::now();

    let filename = PathBuf::from(path).join(filename).with_extension("m4s");

    let file = File::create(filename).await?;
    match can_muti {
        true => fetch_muti(url, content_length, file).await?,
        false => fetch_one(url, file).await?
    };

    // audio file size 98,999 KB

    // 12-core muti Time costs: 4.2598857s 4.9019898s 4.3838892s
    // 8-core muti Time costs: 4.5588841s 4.7504758s 3.8241354s
    // one Time costs: 6.6017306s 6.4260981s 6.5925249s
    trace!("Download time costs: {:?}", start.elapsed());
    Ok(())
}

// #[tokio::test]
// async fn test_load() {
//     let res = crate::catcher::link::api("BV1fB4y1h76Z", 773130617, 16 | 256).await.unwrap();
//     load(&res.dash.flac.unwrap().audio.base_url, "test", "./").await.unwrap();
// }