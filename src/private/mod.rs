use base64;
use chrono::{DateTime, UTC};
use crypto::hmac::Hmac;
use crypto::mac::Mac;
use crypto::sha2::Sha256;
use hyper::client::Client as HttpClient;
use hyper::header::{Headers, UserAgent};
use serde::{self, Deserialize};
use serde_json::de;
use time::get_time;
use uuid::Uuid;

use super::Error;

const PRIVATE_API_URL: &'static str = "https://api.gdax.com";

pub struct Client {
    http_client: HttpClient,
    key: String,
    secret: String,
    passphrase: String
}

#[derive(Deserialize, Debug)]
pub struct Account {
    pub id: Uuid,
    pub balance: f64,
    pub hold: f64,
    pub available: f64,
    pub currency: String
}

pub type Ledger = Vec<LedgerEntry>;

#[derive(Deserialize, Debug)]
pub struct LedgerEntry {
    pub id: u64,
    pub created_at: DateTime<UTC>,
    pub amount: f64,
    pub balance: f64,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub details: Option<EntryDetails>
}

#[derive(Deserialize, Debug)]
pub struct EntryDetails {
    pub order_id: Option<Uuid>,
    pub trade_id: Option<u64>,
    pub product_id: Option<String>,
    pub transfer_id: Option<Uuid>,
    pub transfer_type: Option<String>
}

#[derive(Debug)]
pub enum EntryType {
    Fee,
    Match,
    Transfer
}

// We manually implement Deserialize for EntryType here
// because the default encoding/decoding scheme that derive
// gives us isn't the straightforward mapping unfortunately
impl serde::Deserialize for EntryType {
    fn deserialize<D>(deserializer: &mut D) -> Result<EntryType, D::Error>
        where D: serde::Deserializer {

        struct EntryTypeVisitor;
        impl serde::de::Visitor for EntryTypeVisitor {
            type Value = EntryType;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::Error {
                match &*v.to_lowercase() {
                    "fee" => Ok(EntryType::Fee),
                    "match" => Ok(EntryType::Match),
                    "transfer" => Ok(EntryType::Transfer),
                    _ => Err(E::invalid_value("entry type must be either `fee`, `match` or `transfer`"))
                }
            }
        }
        deserializer.deserialize(EntryTypeVisitor)
    }
}

#[derive(Deserialize, Debug)]
pub struct Hold {
    pub id: Uuid,
    pub account_id: Uuid,
    pub created_at: DateTime<UTC>,
    pub updated_at: DateTime<UTC>,
    pub amount: f64,
    #[serde(rename = "type")]
    pub hold_type: HoldType,
    #[serde(rename = "ref")]
    pub ref_id: Uuid
}

#[derive(Debug)]
pub enum HoldType {
    Order,
    Transfer
}

// We manually implement Deserialize for HoldType here
// because the default encoding/decoding scheme that derive
// gives us isn't the straightforward mapping unfortunately
impl serde::Deserialize for HoldType {
    fn deserialize<D>(deserializer: &mut D) -> Result<HoldType, D::Error>
        where D: serde::Deserializer {

        struct HoldTypeVisitor;
        impl serde::de::Visitor for HoldTypeVisitor {
            type Value = HoldType;

            fn visit_str<E>(&mut self, v: &str) -> Result<Self::Value, E>
                where E: serde::Error {
                match &*v.to_lowercase() {
                    "order" => Ok(HoldType::Order),
                    "transfer" => Ok(HoldType::Transfer),
                    _ => Err(E::invalid_value("hold type must be either `order` or `transfer`"))
                }
            }
        }
        deserializer.deserialize(HoldTypeVisitor)
    }
}

impl Client {
    pub fn new(key: &str, secret: &str, passphrase: &str) -> Client {
        Client {
            http_client: HttpClient::new(),
            key: key.to_owned(),
            secret: secret.to_owned(),
            passphrase: passphrase.to_owned()
        }
    }

    fn signature(&self, path: &str, body: &str, timestamp: &str, method: &str)
        -> Result<String, Error> {

        let key = base64::decode(&self.secret)?;
        let what = format!("{}{}{}{}",
                           timestamp,
                           method.to_uppercase(),
                           path,
                           body);

        let mut hmac = Hmac::new(Sha256::new(), &key);
        hmac.input(what.as_bytes());

        Ok(base64::encode(hmac.result().code()))
    }

    fn get_and_decode<T>(&self, path: &str) -> Result<T, Error>
        where T: Deserialize {

        let timestamp = get_time().sec.to_string();
        let signature = self.signature(path, "", &timestamp, "GET")?;

        let mut headers = Headers::new();
        headers.set(UserAgent("rust-gdax-client/0.1.0".to_owned()));
        headers.set_raw("CB-ACCESS-KEY", vec![self.key.clone().into_bytes()]);
        headers.set_raw("CB-ACCESS-SIGN", vec![signature.into_bytes()]);
        headers.set_raw("CB-ACCESS-PASSPHRASE", vec![self.passphrase.clone().into_bytes()]);
        headers.set_raw("CB-ACCESS-TIMESTAMP", vec![timestamp.into_bytes()]);

        let url = format!("{}{}", PRIVATE_API_URL, path);
        let mut res = self.http_client.get(&url)
                                      .headers(headers)
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

    pub fn get_accounts(&self) -> Result<Vec<Account>, Error> {
        self.get_and_decode("/accounts")
    }

    pub fn get_account(&self, id: Uuid) -> Result<Account, Error> {
        self.get_and_decode(&format!("/accounts/{}", id))
    }

    pub fn get_account_history(&self, id: Uuid) -> Result<Ledger, Error> {
        self.get_and_decode(&format!("/accounts/{}/ledger", id))
    }

    pub fn get_account_holds(&self, id: Uuid) -> Result<Vec<Hold>, Error> {
        self.get_and_decode(&format!("/accounts/{}/holds", id))
    }
}
