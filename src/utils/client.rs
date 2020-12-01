use reqwest;
use reqwest::StatusCode;
use serenity::prelude::*;
pub struct ClientKey;

impl TypeMapKey for ClientKey {
    type Value = Client;
}

pub struct Client {
    req: reqwest::Client,
}

impl Client {
    pub fn new() -> Client {
        Client {
            req: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, url: &str, auth: &str) -> Result<reqwest::Response, u16> {
        let res = self
            .req
            .get(url)
            .header("Authorization", auth)
            .send()
            .await
            .unwrap();
        let stat = res.status().as_u16();
        match res.status() {
            StatusCode::OK => Ok(res),
            _ => Err(stat),
        }
    }

    pub async fn post(&self, url: &str, auth: &str) -> Result<reqwest::Response, u16> {
        let res = self
            .req
            .post(url)
            .header("Authorization", auth)
            .send()
            .await
            .unwrap();
        let stat = res.status().as_u16();
        match res.status() {
            StatusCode::OK => Ok(res),
            _ => Err(stat),
        }
    }
}
