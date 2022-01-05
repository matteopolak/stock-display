use reqwest::{header, Client, Response, Error};
use std::io::{self, Write};

use crate::constants;
use crate::structs;

// generic function that reads `input_length` bytes from stdin
pub fn get_input_string(phrase: &str, input_length: usize) -> String {
	// create a string with a capacity of `input_length` bytes
	let mut input: String = String::with_capacity(input_length);

	print!("{}", phrase);
	io::stdout().flush().ok();

	// read from stdin into `input`, panicking if it cannot be reached
	io::stdin()
		.read_line(&mut input)
		.expect("Could not read from stdin");

	// remove `\n` (and `\r` on windows) from end of string
	if input.ends_with('\n') {
		input.pop();

		if input.ends_with('\r') {
			input.pop();
		}
	}

	// return `input`
	input
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

pub async fn get_stock_price(uri: &str, client: &Client) -> Option<f64> {
	// send the request to receive ticker price data
	// note:	these headers are required to avoid NASDAQ
	// 				rejecting our request
	let request: Result<Response, Error> = client
		.get(uri)
		.header(header::ACCEPT_LANGUAGE, "en-US;q=0.9")
		.header(header::ACCEPT_ENCODING, "text")
		.header(header::USER_AGENT, constants::USER_AGENT_HEADER)
		.send()
		.await;

	if let Ok(response) = request {
		let json: structs::NasdaqDataWrap = response.json::<structs::NasdaqDataWrap>().await.unwrap();
		let mut raw: String = json.data.primaryData.lastSalePrice;

		// remove the leading `$` of the string
		raw.remove(0);

		// parse the string into a 64-bit float
		let price: f64 = raw.parse::<f64>().unwrap();

		// return the price
		return Some(price);
	}

	// return nothing
	None
}