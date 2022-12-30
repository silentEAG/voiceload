use bytes::Bytes;
use futures::Stream;
use once_cell::sync::Lazy;
use reqwest::{Url, header::{self, HeaderMap}, Error, IntoUrl};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

pub struct Client {
    inner: reqwest::Client
}

impl Client {
    fn new() -> Client {
        let mut headers = header::HeaderMap::new();
        headers.insert(header::COOKIE, header::HeaderValue::from_static("SESSDATA=d13cb7d3%2C1687954518%2Cba133%2Ac1"));
        headers.insert(header::REFERER, header::HeaderValue::from_static("https://www.bilibili.com"));
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36"));
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .unwrap();
        Self { inner: client }
    }

    /// Get json data as return
    pub async fn get<T, U, D>(&self, url: U, params: &T) -> Result<D>
    where
        T: Serialize + ?Sized, for<'a> D: Deserialize<'a>, U: IntoUrl {

        let response = self.inner.get(url)
            .query(params)
            .send()
            .await?;
        
        let data = response.bytes().await?;

        serde_json::from_slice::<D>(&data[..])
            .context("Failed to deserialize data.")
    }

    pub async fn get_byte_stream<U>(&self, url: U, headers: Option<HeaderMap>) 
        -> Result<impl Stream<Item = Result<Bytes, Error>>> 
    where U: IntoUrl {

        let mut reqest_builder = self.inner.get(url);

        if let Some(headers) = headers {
            reqest_builder = reqest_builder.headers(headers)
        }

        let response = reqest_builder
            .send()
            .await?;
        
        let stream = response.bytes_stream();
        Ok(stream)
    }

    pub async fn head(&self, url: &str) -> Result<HeaderMap> {

        let url = Url::parse(url)?;

        let response = self.inner.head(url)
            .send()
            .await?;
        
        let headers = response.headers();
        Ok(headers.to_owned())
    }
}

pub static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::new()
});
