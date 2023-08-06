use std::collections::HashMap;
use std::env;

use log::{error, info};
use serde::Deserialize;
use teloxide::{prelude::*, utils::command::BotCommands};

const API_URL: &str = "https://api.coingecko.com/api/v3/";

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Display this text.")]
    Help,
    #[command(description = "Get coin price (list coins to find coin id)")]
    Coin(String),
    #[command(description = "Get Top Coins by marketcap (indicate number)")]
    Top(String),
}

#[derive(Deserialize, Debug)]
struct Coin {
    id: String,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info) // Set default level to Info
        .init();
    info!("starting bot...");

    dotenv::dotenv().ok();

    let bot = Bot::new(env::var("TOKEN").expect("Token not set"));
    Command::repl(bot, reply).await
}

async fn reply(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Coin(coin) => {
            if coin.is_empty() {
                bot.send_message(msg.chat.id, format!("Take an argument: /coin name"))
                    .await?
            } else {
                let coin = normalize_coin_name(coin);
                let response = get_coin_price(format!(
                    "{API_URL}simple/price?ids={coin}&vs_currencies=usd"
                ))
                .await;

                match response {
                    Ok(price) => {
                        bot.send_message(
                            msg.chat.id,
                            format!("{} price is ${price}", coin),
                        )
                        .await?
                    }
                    Err(e) => bot.send_message(msg.chat.id, e).await?,
                }
            }
        }
        Command::Top(number) => {
            if number.is_empty() {
                bot.send_message(msg.chat.id, format!("Take an argument: /top number"))
                    .await?
            } else {
                let response = get_top_coins(number).await;

                match response {
                    Ok(list) => bot.send_message(msg.chat.id, format!("{:?}", list)).await?,
                    Err(e) => bot.send_message(msg.chat.id, format!("{e}")).await?,
                }
            }
        }
    };

    Ok(())
}

async fn get_coin_price(url: String) -> Result<f32, String> {
    let response: HashMap<String, HashMap<String, f32>> = reqwest::get(&url)
        .await
        .map_err(|e| format!("Couldn't fetch API: {e}"))?
        .json()
        .await
        .map_err(|e| format!("Couldn't parse result: {e}"))?;

    if let Some(token) = response.values().next() {
        if let Some(usd) = token.values().next() {
            info!("Price queried: {usd}");
            return Ok(usd.to_owned());
        }
    }

    error!("Error while querying: {url}");
    Err("Wrong data format, verify your coin id by getting top coins".to_string())
}

async fn get_top_coins(number: String) -> reqwest::Result<Vec<String>> {
    let client = reqwest::Client::new();
    let response = client.get(format!("{API_URL}coins/markets?vs_currency=usd&order=market_cap_desc&per_page={number}&page=1&sparkline=false&locale=en"))
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.3")
        .send()
        .await?
        .json::<Vec<Coin>>()
        .await?
        .into_iter()
        .map(|coin| coin.id)
        .collect::<Vec<String>>();

    Ok(response)
}

fn normalize_coin_name(coin: String) -> String {
    let formatted = match coin.as_str() {
        "sol" | "solana" => "solana",
        "egld" | "elrond" | "egold" | "erd" | "elrond-erd-2" => "elrond-erd-2",
        "btc" | "bitcoin" => "bitcoin",
        "eth" | "ether" | "ethereum" => "ethereum",
        "xrd" | "radix" => "radix",
        "aptos" | "apt" => "aptos",
        "avalanche" | "avax" | "avalanche-2" => "avalanche-2",
        "bnb" | "binance" | "bnbchain" | "bnbcoin" | "bsc" | "binancecoin" | "bsccoin" => "binancecoin",
        coin => coin,
    };

    formatted.to_string()
}
