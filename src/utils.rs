use reqwest::{header, Client};
use std::io::{self, Write};

use crate::constants;
use crate::structs;

pub fn get_stock_ticker() -> String {
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

	ticker
}

// prints out the price changes to the console
// TODO: add a nice chart
pub fn pretty_print_data(current_price: f64, last_price: f64) -> () {
	let number_prefix: char = if current_price >= last_price {
		'+'
	} else {
		'-'
	};

	print!(
		"      current price: ${} | last price: ${} | change: {}${}\r",
		current_price,
		last_price,
		number_prefix,
		(current_price - last_price).abs()
	);
	io::stdout().flush().ok();
}

// returns a Future that resolves after `s` seconds
pub fn sleep(s: u64) -> tokio::time::Sleep {
	tokio::time::sleep(tokio::time::Duration::from_secs(s))
}

pub async fn get_stock_price(ticker: &str) -> Option<f64> {
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
		.send()
		.await;

	if let Ok(response) = request {
		let json = response.json::<structs::NasdaqDataWrap>().await.unwrap();
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
