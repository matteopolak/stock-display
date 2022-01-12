use colored::{ColoredString, Colorize};
use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use reqwest::{header, Client, Error, Response};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::str;
use std::time::{Duration, SystemTime};
use termsize::{self, Size};

use crate::constants;
use crate::structs;

// generic function that reads `input_length` bytes from stdin
pub fn get_input_string(phrase: &str, input_length: usize) -> String {
	// create a string with a capacity of `input_length` bytes
	let mut input: String = String::with_capacity(input_length);

	// print out the phrase
	print!("{}", phrase);

	// important: flush the stdout buffer manually
	// or it will not print anything until a new-line character
	// is reached
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
pub fn pretty_print_data(
	ticker: &str,
	points: &VecDeque<(f64, f64)>,
	current_price: f64,
	last_price: f64,
	average_price: f64,
	width: u32,
	height: u32,
	index: u32,
	(mtd, qtd, ytd): (f64, f64, f64),
) -> () {
	// create a new plot
	//
	// note: copying data here is not very efficient, and is
	// something that could be improved if this library requests
	// for a &Vec<(f64, 64)> instead of Vec<(f64, f64)> in the future
	let plot: Plot = Plot::new(Vec::from_iter(points.clone().into_iter())).point_style(
		PointStyle::new()
			.marker(PointMarker::Circle)
			.colour("#DD3355"),
	);

	// create a new plot viewer
	let view: ContinuousView = ContinuousView::new().add(plot).x_range(
		(if index <= width { 0 } else { index - width }) as f64,
		width as f64,
	);

	// print the plot to the console
	println!(
		"{}",
		Page::single(&view)
			.dimensions(width, height)
			.to_text()
			.unwrap()
	);

	// print out some metrics
	println!(
		"   {} | Price: {} | Last: {} | Average: {} | Change: {} | MTD: {} | QTD: {} | YTD: {}",
		ticker.cyan(),
		round_and_whiten(current_price),
		round_and_whiten(last_price),
		diff_without_sign(average_price, current_price),
		diff_with_sign(last_price, current_price),
		diff_with_sign_percent(mtd, current_price),
		diff_with_sign_percent(qtd, current_price),
		diff_with_sign_percent(ytd, current_price),
	);
}

// round to two decimal places and make the text white
pub fn round_and_whiten(num: f64) -> ColoredString {
	format!("${:.2}", num).white()
}

// utility for printing the difference between two numbers,
// explicitly putting a `+` when it's greater
pub fn diff_with_sign(old: f64, new: f64) -> ColoredString {
	let diff = new - old;
	let greater = diff >= 0.;

	let string = format!("{}${:.2}", if greater { '+' } else { '-' }, diff.abs());

	if greater {
		string.green()
	} else {
		string.red()
	}
}

// utility for printing the difference between two numbers
pub fn diff_without_sign(old: f64, new: f64) -> ColoredString {
	let diff = new - old;
	let string = format!("${:.2}", old);

	if diff >= 0. {
		string.green()
	} else {
		string.red()
	}
}

// utility for printing the percentage change between two numbers
pub fn diff_with_sign_percent(old: f64, new: f64) -> ColoredString {
	let diff = new - old;

	// `:+.2` = round to 2 decimals, and include the `+`
	// character if it's positive
	let string = format!("{:+.2}%", diff / old * 100.);

	if diff >= 0. {
		string.green()
	} else {
		string.red()
	}
}

// returns a Future that resolves after `s` seconds
pub fn sleep(s: u64) -> tokio::time::Sleep {
	tokio::time::sleep(tokio::time::Duration::from_secs(s))
}

pub fn terminal_size() -> (u32, u32) {
	// get the width and height of the terminal
	let Size { cols: x, rows: y } = termsize::get().unwrap();

	(x as u32 - 15, y as u32 - 6)
}

pub async fn stock_price(uri: &str, client: &Client) -> Option<f64> {
	// send the request to receive ticker price data
	//
	// note: these headers are required to avoid Nasdaq
	// rejecting our request
	let request: Result<Response, Error> = client
		.get(uri)
		.header(header::ACCEPT_LANGUAGE, "en-US;q=0.9")
		.header(header::ACCEPT_ENCODING, "text")
		.header(header::USER_AGENT, constants::USER_AGENT_HEADER)
		.send()
		.await;

	if let Ok(response) = request {
		let json: structs::NasdaqDataWrap = match response.json::<structs::NasdaqDataWrap>().await {
			Ok(j) => j,
			Err(_) => return None,
		};

		// remove the first character from the string
		//
		// note: you can simply use `String#remove` to remove
		// the first character, but it copies the rest of the string.
		// with this method, I'm converting the string to an iterator,
		// skipping the first byte (first character in this case, as
		// stock tickers can only use letters of the alphabet), and collecting
		// it into a vector
		//
		// also note: a String in Rust is simply a wrapper around Vec<u8>, so
		// this conversion is costless
		let raw: Vec<u8> = json
			.data
			.primaryData
			.lastSalePrice
			.into_bytes()
			.into_iter()
			.skip(1)
			.collect::<Vec<u8>>();

		// parse the vec into a string, then into a
		// 64-bit float
		let price: f64 = str::from_utf8(&raw).unwrap().parse::<f64>().unwrap();

		// return the price
		return Some(price);
	}

	// return nothing
	None
}

pub async fn is_valid_ticker(ticker: &str, client: &Client) -> bool {
	// check if the ticker length is between 1 and 5
	if !(1..5).contains(&ticker.len()) {
		return false;
	}

	// build a new uri with the ticker
	let uri: String = constants::NASDAQ_API_ENDPOINT.replace("{ticker}", ticker);

	// construct and send an http request to the Nasdaq API
	let request: Result<Response, Error> = client
		.get(uri)
		.header(header::ACCEPT_LANGUAGE, "en-US;q=0.9")
		.header(header::ACCEPT_ENCODING, "text")
		.header(header::USER_AGENT, constants::USER_AGENT_HEADER)
		.send()
		.await;

	// if the response is ok...
	if let Ok(response) = request {
		// parse the response body
		let status = match response.json::<structs::NasdaqStatusWrap>().await {
			// if it parses successfully, return the response code
			Ok(j) => j.status.rCode,
			Err(_) => return false,
		};

		// it's successful if the code is 200 (200 = OK)
		return status == 200;
	}

	// if the response was not ok, it will skip
	// the `if let` statement and just return false
	false
}

// get the stock price history for a ticker
pub async fn ticker_history(ticker: &str, client: &Client) -> Option<(f64, f64, f64)> {
	// get the current year
	let year = current_year();

	// construct the uri
	let uri = constants::MARKETSTACK_API_ENDPOINT
		.replace("{ticker}", ticker)
		.replace("{start}", &format!("{}-01-01", year))
		.replace("{end}", &format!("{}-12-31", year));

	// create an http request to fetch the data
	let request: Result<Response, Error> = client.get(uri).send().await;

	// if it's successful
	if let Ok(response) = request {
		// parse the json response into a vector
		let days = match response.json::<structs::NameStackDataWrap>().await {
			Ok(j) => j.data,
			Err(_) => return None,
		};

		// get the length of the vector
		let length = days.len();

		// get the last entry in the vector
		let last = match days.get(length - 1) {
			Some(d) => d,
			None => return None,
		};

		// entries are ordered from newest to oldest, so this is
		// getting the nth day in the past from today, which is useful
		// for an approximate price for mtd, qtd, and ytd
		//
		// note: `unwrap_or` makes `last` a fallback, which is used when
		// there is not enough history for the stock price
		let mtd = days.get(29).unwrap_or(last).open;
		let qtd = days.get(90).unwrap_or(last).open;
		let ytd = days.get(364).unwrap_or(last).open;

		// return a tuple of the data
		return Some((mtd, qtd, ytd));
	}

	None
}

pub fn current_year() -> u64 {
	let now: Duration = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.expect("We must be in Back to the Future 4, where they go to the past...");

	// divide the number of seconds since January 1, 1970
	// by the number of seconds in a year to get the number
	// of years elapsed since 1970
	1970 + now.as_secs() / 31536000
}
