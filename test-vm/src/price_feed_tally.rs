use anyhow::Result;
use seda_sdk_rs::{get_reveals, Process};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct ExecutionResult {
    pub price: u64,
}

fn median(mut nums: Vec<u64>) -> u64 {
    nums.sort();
    let middle = nums.len() / 2;

    if nums.len() % 2 == 0 {
        return (nums[middle - 1] + nums[middle]) / 2;
    }

    nums[middle]
}

pub fn price_feed_tally() -> Result<()> {
    let reveals = get_reveals()?;
    let mut prices: Vec<u64> = Vec::new();

    for reveal in reveals {
        let price = match serde_json::from_slice::<ExecutionResult>(&reveal.body.reveal) {
            Ok(value) => value.price,
            Err(error) => {
                eprintln!("Could not decode reveal {error}");
                continue;
            }
        };

        prices.push(price);
    }

    if prices.len() > 0 {
        let final_price = median(prices);

        println!("{}", &final_price);

        Process::success(&final_price.to_be_bytes());

        return Ok(());
    }

    Process::error("No consensus among revealed results".as_bytes());

    Ok(())
}
