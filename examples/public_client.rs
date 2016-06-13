extern crate gdax_client;

use gdax_client::PublicClient;

fn main() {
    let public_client = PublicClient::new();

    println!("Products:\n{:?}", public_client.get_products());
    println!("Product Order Book: \n{:?} \n{:?} \n{:?}",
             public_client.get_best_order("BTC-USD"),
             public_client.get_top50_orders("BTC-USD"),
             public_client.get_full_book("BTC-USD"));
    println!("Product Ticker: {:?}", public_client.get_product_ticker("BTC-USD"));
}
