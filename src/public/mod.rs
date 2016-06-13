use chrono::{self, DateTime};
use hyper::client::Client as HttpClient;
use hyper::header::UserAgent;
use serde_json::de;

use super::Error;

const PUBLIC_API_URL: &'static str = "https://api.gdax.com";

pub enum Level {
    Best    = 1,
    Top50   = 2,
    Full    = 3
}

#[derive(Deserialize, Debug)]
pub struct Product {
    id: String,
    base_currency: String,
    quote_currency: String,
    base_min_size: f64,
    base_max_size: f64,
    quote_increment: f64
}

#[derive(Deserialize, Debug)]
pub struct BookEntry {
    price: f64,
    size: f64,
    num_orders: u64
}

#[derive(Deserialize, Debug)]
pub struct FullBookEntry {
    price: f64,
    size: f64,
    order_id: String
}

#[derive(Deserialize, Debug)]
pub struct OrderBook<T> {
    sequence: usize,
    bids: Vec<T>,
    asks: Vec<T>
}

#[derive(Deserialize, Debug)]
pub struct Tick {
    trade_id: u64,
    price: f64,
    size: f64,
    bid: f64,
    ask: f64,
    volume: f64,
    time: DateTime<chrono::UTC>
}

pub struct Client {
    http_client: HttpClient,
}

impl Client {
    pub fn new() -> Client {
        Client {
            http_client: HttpClient::new()
        }
    }

    pub fn get_products(&self) -> Result<Vec<Product>, Error> {
        let url = format!("{}/products", PUBLIC_API_URL);
        let mut res = self.http_client.get(&url)
                                      .header(UserAgent("HakunaMatata/1.0".to_owned()))
                                      .send()?;

        if !res.status.is_success() {
            return Err(Error::Api);
        }

        Ok(de::from_reader(&mut res)?)
    }

    pub fn get_best_order(&self, product: &str) -> Result<OrderBook<BookEntry>, Error> {
        let url = format!("{}/products/{}/book?level={}", PUBLIC_API_URL, product, Level::Best as u8);
        let mut res = self.http_client.get(&url)
                                      .header(UserAgent("HakunaMatata/1.0".to_owned()))
                                      .send()?;

        if !res.status.is_success() {
            return Err(Error::Api);
        }

        Ok(de::from_reader(&mut res)?)
    }

    pub fn get_top50_orders(&self, product: &str) -> Result<OrderBook<BookEntry>, Error> {
        let url = format!("{}/products/{}/book?level={}", PUBLIC_API_URL, product, Level::Top50 as u8);
        let mut res = self.http_client.get(&url)
                                      .header(UserAgent("HakunaMatata/1.0".to_owned()))
                                      .send()?;

        if !res.status.is_success() {
            return Err(Error::Api);
        }

        Ok(de::from_reader(&mut res)?)
    }

    pub fn get_full_book(&self, product: &str) -> Result<OrderBook<FullBookEntry>, Error> {
        let url = format!("{}/products/{}/book?level={}", PUBLIC_API_URL, product, Level::Full as u8);
        let mut res = self.http_client.get(&url)
                                      .header(UserAgent("HakunaMatata/1.0".to_owned()))
                                      .send()?;

        if !res.status.is_success() {
            return Err(Error::Api);
        }

        Ok(de::from_reader(&mut res)?)
    }

    pub fn get_product_ticker(&self, product: &str) -> Result<Tick, Error> {
        let url = format!("{}/products/{}/ticker", PUBLIC_API_URL, product);
        let mut res = self.http_client.get(&url)
                                      .header(UserAgent("HakunaMatata/1.0".to_owned()))
                                      .send()?;

        if !res.status.is_success() {
            return Err(Error::Api);
        }

        Ok(de::from_reader(&mut res)?)
    }
}
