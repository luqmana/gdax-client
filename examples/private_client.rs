extern crate gdax_client;

use gdax_client::PrivateClient;

const CB_KEY: &'static str = env!("CB_KEY");
const CB_SECRET: &'static str = env!("CB_SECRET");
const CB_PASSPHRASE: &'static str = env!("CB_PASSPHRASE");

fn main() {
    let private_client = PrivateClient::new(CB_KEY, CB_SECRET, CB_PASSPHRASE);

    if let Ok(accounts) = private_client.get_accounts() {
        println!("Accounts: {:?}", accounts);

        println!("Account [{}]: {:?}",
                 accounts[0].id,
                 private_client.get_account(accounts[0].id));
    }
}
