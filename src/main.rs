use core::str;
use std::collections::HashMap;
use std::str::FromStr;
use std::ops::{Div, Mul, Add };
use ethers::{
    abi::{AbiDecode, Address},
    providers::{Middleware, Provider, StreamExt},
    types::{Filter, U256},
};
use rust_decimal::MathematicalOps;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::Serialize;
use serde_json::json;

const PROVIDER_URL: &str =
    "wss://lb.drpc.org/ogws?network=ethereum&dkey=AneYGS0T00lnn0-FO6V-2jKtj5W_gZIR74kNpldAe0Cl";

const CONTRACT_ADDRESS: &str = "0x0d4a11d5EEaaC28EC3F61d100daF4d40471f1852";

const USDT_DEC: u32 = 6;
const WETH_DEC:u32 = 18;


#[derive(Serialize)]
struct VOb(HashMap<String, String>);


// Calculate execution price
fn execution_price(token0_amount: Decimal, reserve0: Decimal, reserve1:Decimal) -> (Decimal, Decimal){


    let out = token0_amount.mul(reserve1).div(token0_amount.add(reserve0));

    (token0_amount.div(out), out)
    
}


#[tokio::main]
async fn main() {
    let provider: Provider<ethers::providers::Ws> = match Provider::connect(PROVIDER_URL).await {
        Ok(p) => p,
        Err(_) => return,
    };

    let address: Address = match CONTRACT_ADDRESS.parse::<Address>() {
        Ok(a) => a,
        Err(_) => return,
    };

    let filter = Filter::new()
        .address(address)
        .event("Sync(uint112,uint112)");

    let sub = provider.subscribe_logs(&filter).await;

    if let Ok(mut subscription) =  sub {

        while let Some(log) = subscription.next().await {

            let mut vob: HashMap<String, String> = HashMap::new();

            let reserves = log.data.split_at(32);

            if let (Ok(res0), Ok(res1)) = (U256::decode(reserves.0), U256::decode(reserves.1)) {

                

                if let (Ok(r0), Ok(r1), Some(decimals_0), Some(decimals_1)) = (Decimal::from_str(&res0.to_string()), Decimal::from_str(&res1.to_string()), Decimal::from_u32(WETH_DEC), Decimal::from_u32(USDT_DEC)){                


                    let volume_increment = Decimal::from_u128(10000).unwrap();
                    let mut in_usdt = Decimal::ZERO;
                    let r0_adjusted = r0.div(Decimal::TEN.powd(decimals_0));
                    let r1_adjusted = r1.div(Decimal::TEN.powd(decimals_1));

                    // increment amount in to simulate different execution prices
                    for _ in 1..10{

                        in_usdt = in_usdt.add(volume_increment);

                        let (price_weth, _) = execution_price(in_usdt, r1_adjusted, r0_adjusted);
    

                        vob.insert(price_weth.to_string(), in_usdt.to_string());

                    }

                    let vob_json = json!(VOb(vob));
                    println!("VOB for USDT -> WETH SWAP [price: volume]:\n{:#}", vob_json);
  

                }

            } else {
                return;
            }
        }
    }
}
