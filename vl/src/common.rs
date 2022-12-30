use bytes::Bytes;
use futures::Stream;
use once_cell::sync::Lazy;
use reqwest::{header::{self, HeaderMap}, Error, IntoUrl, Method, Response};
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};

pub struct Client {
    inner: reqwest::Client
}

impl Client {
    fn new() -> Client {
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .unwrap();
        Self { inner: client }
    }

    pub async fn request<T, U>(&self, method: Method, url: U, params: &T, headers: Option<HeaderMap>) -> Result<Response>
    where T: Serialize + ?Sized, U: IntoUrl {

        let mut map = DEFAULT_HEADER.clone();
        
        if let Some(headers) = headers {
            map.extend(headers);
        }

        let builder = self.inner.request(method, url)
            .headers(map)
            .query(params);
        
        Ok(builder.send().await?)
    }

    /// Get a struct data as return
    pub async fn get_struct<T, U, D>(&self, url: U, params: &T, headers: Option<HeaderMap>) -> Result<D>
    where
        T: Serialize + ?Sized, for<'a> D: Deserialize<'a>, U: IntoUrl {

        let response = self.request(Method::GET, url, params, headers)
            .await?;
        
        let bytes = response.bytes().await?;

        serde_json::from_slice::<D>(&bytes[..])
            .context("Failed to deserialize data.")
    }

    pub async fn get_byte_stream<U>(&self, url: U, headers: Option<HeaderMap>) -> Result<impl Stream<Item = Result<Bytes, Error>>> 
    where
        U: IntoUrl {

        let response = self.request(Method::GET, url, &(), headers)
            .await?;
        
        let stream = response.bytes_stream();

        Ok(stream)
    }

    pub async fn head<U>(&self, url: U) -> Result<HeaderMap>
    where
        U: IntoUrl {

        let response = self.inner.head(url)
            .headers(DEFAULT_HEADER.clone())
            .send()
            .await?;
        
        let headers = response.headers();
        Ok(headers.to_owned())
    }
}

pub static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::new()
});

pub static DEFAULT_HEADER: Lazy<HeaderMap> = Lazy::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(header::REFERER, header::HeaderValue::from_static("https://www.bilibili.com"));
    headers.insert(header::USER_AGENT, header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36"));
    headers
});
