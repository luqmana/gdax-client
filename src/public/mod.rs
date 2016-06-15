use chrono::{DateTime, UTC};
use hyper::client::Client as HttpClient;
use hyper::header::UserAgent;
use serde::{self, Deserialize};
use serde_json::de;
use uuid::Uuid;

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
    order_id: Uuid
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
    time: DateTime<UTC>
}

#[derive(Deserialize, Debug)]
pub struct Trade {
    time: DateTime<UTC>,
    trade_id: u64,
    price: f64,
    size: f64,
    side: Side,
}

#[derive(Debug)]
enum Side {
    Buy,
    Sell
}

// We manually implement Deserialize for Side here
// because the default encoding/decoding scheme that derive
// gives us isn't the straightforward mapping unfortunately
impl serde::Deserialize for Side {
    fn deserialize<D>(deserializer: &mut D) -> Result<Side, D::Error>
        where D: serde::Deserializer {

        struct SideVisitor;
        impl serde::de::Visitor for SideVisitor {
            type Value = Side;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::Error {
                match &*v.to_lowercase() {
                    "buy" => Ok(Side::Buy),
                    "sell" => Ok(Side::Sell),
                    _ => Err(E::invalid_value("side must be either `buy` or `sell`"))
                }
            }
        }
        deserializer.deserialize(SideVisitor)
    }
}

#[derive(Deserialize, Debug)]
pub struct Candle {
    time: u64,
    low: f64,
    high: f64,
    open: f64,
    close: f64,
    volume: f64
}

#[derive(Deserialize, Debug)]
pub struct Stats {
    open: f64,
    high: f64,
    low: f64,
    volume: f64
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

    fn get_and_decode<T>(&self, url: &str) -> Result<T, Error>
        where T: Deserialize {

        let mut res = self.http_client.get(url)
                                      .header(UserAgent("rust-gdax-client/0.1.0".to_owned()))
                                      .send()?;

        if !res.status.is_success() {
            #[derive(Deserialize, Debug)]
            struct E {
                message: String
            }
            return Err(Error::Api((de::from_reader(&mut res)?: E).message));
        }

        Ok(de::from_reader(&mut res)?)
    }

    pub fn get_products(&self) -> Result<Vec<Product>, Error> {
        self.get_and_decode(&format!("{}/products", PUBLIC_API_URL))
    }

    pub fn get_best_order(&self, product: &str) -> Result<OrderBook<BookEntry>, Error> {
        self.get_and_decode(&format!("{}/products/{}/book?level={}",
                                     PUBLIC_API_URL,
                                     product,
                                     Level::Best as u8))
    }

    pub fn get_top50_orders(&self, product: &str) -> Result<OrderBook<BookEntry>, Error> {
        self.get_and_decode(&format!("{}/products/{}/book?level={}",
                                     PUBLIC_API_URL,
                                     product,
                                     Level::Top50 as u8))
    }

    pub fn get_full_book(&self, product: &str) -> Result<OrderBook<FullBookEntry>, Error> {
        self.get_and_decode(&format!("{}/products/{}/book?level={}",
                                     PUBLIC_API_URL,
                                     product,
                                     Level::Full as u8))
    }

    pub fn get_product_ticker(&self, product: &str) -> Result<Tick, Error> {
        self.get_and_decode(&format!("{}/products/{}/ticker", PUBLIC_API_URL, product))
    }

    pub fn get_trades(&self, product: &str) -> Result<Vec<Trade>, Error> {
        self.get_and_decode(&format!("{}/products/{}/trades", PUBLIC_API_URL, product))
    }

    // XXX: Returns invalid interval?
    pub fn get_historic_rates(&self,
                              product: &str,
                              start_time: DateTime<UTC>,
                              end_time: DateTime<UTC>,
                              granularity: u64)
        -> Result<Vec<Candle>, Error> {

        self.get_and_decode(&format!("{}/products/{}/candles?start={}&end={}&granularity={}",
                                     PUBLIC_API_URL,
                                     product,
                                     start_time.to_rfc3339(),
                                     end_time.to_rfc3339(),
                                     granularity))
    }

    pub fn get_24hr_stats(&self, product: &str) -> Result<Stats, Error> {
        self.get_and_decode(&format!("{}/products/{}/stats", PUBLIC_API_URL, product))
    }
}
