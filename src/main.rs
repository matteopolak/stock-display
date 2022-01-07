#![feature(int_log)]

use reqwest::Client;
use std::collections::VecDeque;

mod constants;
mod structs;
mod utils;

#[tokio::main]
async fn main() -> () {
	let terminal = console::Term::stdout();

	// create a new http client from which to dispatch requests
	let client: Client = Client::builder()
		.min_tls_version(reqwest::tls::Version::TLS_1_2)
		.build()
		.unwrap();

	// note: this loop will continue looping until it is broken
	// out of (with a value here, which is what the variable
	// is going to be assigned to)
	let ticker: String = loop {
		// get the stock ticker from the user
		// of 5 bytes, which is the maximum length of a ticker
		let ticker = utils::get_input_string("      stock ticker: ", 5).to_uppercase();

		// if the ticker is valid...
		if utils::is_valid_ticker(&ticker, &client).await {
			// break out of the loop with the ticker
			break ticker;
		}

		// if not, say it's invalid and start again
		println!("      invalid ticker, please try again");
	};

	// clear the terminal
	terminal.clear_screen().unwrap();

	// hide the cursor
	terminal.hide_cursor().unwrap();

	// build the URI from the ticker name
	let uri: String = constants::NASDAQ_API_ENDPOINT.replace("{ticker}", &ticker);

	// create a vector to store data for the chart
	//
	// note: a VecDeque is similar to a Vec, except that
	// it uses a ring buffer to efficiently allow popping (removing)
	// and pushing (adding) to the front *and* back
	let mut points: VecDeque<(f64, f64)> = VecDeque::new();

	// create a counter
	let mut i: u32 = 1;

	// create some utility variables for metrics
	let mut first: bool = true;
	let mut last_price: f64 = 0.;
	let mut total_price: f64 = 0.;

	let history = match utils::ticker_history(&ticker, &client).await {
		Some(h) => h,
		None => return,
	};

	while let Some(price) = utils::stock_price(&uri, &client).await {
		let (x, y) = utils::terminal_size();

		total_price += price;

		if first {
			first = false;
			last_price = price;
		}

		// if it's larger than the scroll window, then
		// remove the first entry
		if i > x as u32 {
			points.pop_front();
		}

		// add the next point to the end of the vector
		points.push_back((i as f64, price));

		// clear the terminal
		terminal.clear_screen().unwrap();

		// print out the price
		utils::pretty_print_data(
			&ticker,
			&points,
			price,
			last_price,
			total_price / i as f64,
			x as u32,
			y as u32,
			i,
			history,
		);

		// increase the counter by 1
		// note: Rust does not support pre- or
		// post-incrementing to avoid a lot of undefined
		// behaviour (like with C/C++), so this is the
		// only other way to increment
		//
		// fun fact: `n = ++i + i;` is still undefined
		// behaviour in C and C++
		i += 1;

		// wait 5 seconds
		utils::sleep(5).await;
	}

	// this is only reached when the loop is broken out of,
	// which only happens when the stock price can not be fetched
	println!("      error fetching ticker data");
}

#[cfg(test)]
mod tests {
	use super::*;
	use colored::Colorize;

	#[tokio::test]
	async fn test_ticker_history() {
		let client: Client = Client::builder()
			.min_tls_version(reqwest::tls::Version::TLS_1_2)
			.build()
			.unwrap();

		assert!(utils::ticker_history("AAPL", &client).await.is_some());
	}

	#[tokio::test]
	async fn test_stock_price() {
		let client: Client = Client::builder()
			.min_tls_version(reqwest::tls::Version::TLS_1_2)
			.build()
			.unwrap();

		let uri: String = constants::NASDAQ_API_ENDPOINT.replace("{ticker}", "AAPL");

		assert!(utils::stock_price(&uri, &client).await.is_some());
	}

	#[test]
	fn test_positive_diff_with_sign_percent() {
		assert_eq!(utils::diff_with_sign_percent(5., 10.), "+100.00%".green());
	}

	#[test]
	fn test_negative_diff_with_sign_percent() {
		assert_eq!(utils::diff_with_sign_percent(10., 5.), "-50.00%".red());
	}

	#[test]
	fn test_positive_diff_without_sign() {
		assert_eq!(utils::diff_without_sign(5., 10.), "$5.00".green());
	}

	#[test]
	fn test_negative_diff_without_sign() {
		assert_eq!(utils::diff_without_sign(10., 5.), "-$5.00".red());
	}

	#[test]
	fn test_positive_diff_with_sign() {
		assert_eq!(utils::diff_with_sign(5., 10.), "+$5.00".green());
	}

	#[test]
	fn test_negative_diff_with_sign() {
		assert_eq!(utils::diff_with_sign(10., 5.), "-$5.00".red());
	}

	#[test]
	fn test_round_and_whiten() {
		assert_eq!(utils::round_and_whiten(10.365), "$10.37".white());
	}

	#[test]
	fn test_current_year() {
		// check if the year is four digits
		// it's rounded down every time since it's an
		// integer, so I need to check for 3
		//
		// log10(1000) = 3
		// log10(9999) = 3.999
		assert_eq!(utils::current_year().log10(), 3);
	}
}
