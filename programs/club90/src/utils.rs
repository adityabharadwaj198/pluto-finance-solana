use std::convert::TryInto;
use crate::*;
use pyth_client::{
    PriceType,
    PriceStatus,
    CorpAction,
    Product,
    Price,
    load_mapping,
    load_product,
    load_price
  };

let url = "http://api.devnet.solana.com";
let key = "BmA9Z6FjioHJPpjT39QazZyhDRUdZy2ezwx4GiDdE2u2";
let clnt = RpcClient::new( url.to_string() );
  
pub fn get_prod_data_from_pyth(prod_address: Pubkey) -> Result<Product> {
    let prod_data = clnt.get_account_data(&prod_address).unwrap();
    let prod_account = load_product(&prod_data).unwrap();
    Ok(prod_acct)
}

pub fn get_price_data_from_pyth(price_address: Pubkey) -> Result<Price> {
    let price_data = clnt.get_account_data(&price_address).unwrap();
    let price_account = load_price(&price_data).unwrap();
    Ok(price_account)
}
