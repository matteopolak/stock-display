use std::io::{self, Write};
use serde::Deserialize;
use reqwest::{Client, header};

mod constants;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct NasdaqDataWrap {
	data: NasdaqData
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct NasdaqData {
	primaryData: NasdaqPrimaryData
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct NasdaqPrimaryData {
	lastSalePrice: String
}

#[tokio::main]
async fn main() -> io::Result<()> {
	// create a string with a capacity of `5`, as tickers
	// can only be up to 5 characters in length
	let mut ticker: String = String::with_capacity(5);

	print!("      stock ticker: ");
	io::stdout().flush().ok();

	// read from stdin into `ticker`, panicking if it cannot be reached 
	io::stdin()
		.read_line(&mut ticker)
		.expect("Could not read from stdin");

	// remove `\n` (and `\r` on windows) from end of string
	if ticker.ends_with('\n') {
		ticker.pop();

		if ticker.ends_with('\r') {
			ticker.pop();
		}
	}

	let mut last_price: f64 = 0.0;

	loop {
		let price: f64 = match get_stock_price(&ticker).await {
			Some(p) => p,
			None => break
		};

		pretty_print_data(price, last_price);

		last_price = price;

		sleep(60).await;
	}

	// show that there were no errors on exit
	Ok(())
}

// prints out the price changes to the console
// TODO: add a nice chart
fn pretty_print_data(current_price: f64, last_price: f64) -> () {
	print!("      current price: ${} | last price: ${} | change: {}${}\r", current_price, last_price, if current_price > last_price { '+' } else { '-' }, (current_price - last_price).abs());
	io::stdout().flush().ok();
}

// returns a Future that resolves after `s` seconds
fn sleep(s: u64) -> tokio::time::Sleep {
	tokio::time::sleep(tokio::time::Duration::from_secs(s))
}

async fn get_stock_price(ticker: &str) -> Option<f64> {
	let client = Client::builder()
		.min_tls_version(reqwest::tls::Version::TLS_1_2)
		.build()
		.unwrap();

	// build the URI from the ticker name
	let uri = constants::NASDAQ_API_ENDPOINT.replace("{ticker}", ticker);

	// send the request to receive ticker price data
	// note:	these headers are required to avoid NASDAQ
	// 				rejecting our request
	let request = client
		.get(uri)
		.header(header::ACCEPT_LANGUAGE, "en-US;q=0.9")
		.header(header::ACCEPT_ENCODING, "text")
		.header(header::USER_AGENT, constants::USER_AGENT_HEADER)
		.send().await;

	if let Ok(response) = request {
		let json = response.json::<NasdaqDataWrap>().await.unwrap();
		let mut raw = json.data.primaryData.lastSalePrice;

		// remove the leading `$` of the string
		raw.remove(0);
		
		// parse the string into a 64-bit float
		let price = raw.parse::<f64>().unwrap();

		// return the price
		return Some(price);
	}

	// return nothing
	None
}