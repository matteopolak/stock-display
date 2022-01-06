use plotlib::page::Page;
use plotlib::repr::Plot;
use plotlib::style::{PointMarker, PointStyle};
use plotlib::view::ContinuousView;
use reqwest::{header, Client, Error, Response};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::str;
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
) -> () {
	// create a new plot
	//
	// note: copying data here is not very efficient, and is
	// something that could be improved if there was more time
	// to complete this project. for now, this is faster than
	// the alternative of using a Vec when storing the points,
	// so it's the one being used
	let plot: Plot = Plot::new(Vec::from_iter(points.clone().into_iter())).point_style(
		PointStyle::new()
			.marker(PointMarker::Square)
			.colour("#DD3355"),
	);

	// create a new plot viewer
	let view: ContinuousView = ContinuousView::new().add(plot);

	// calculate the price change
	let price_change: f64 = last_price - current_price;

	// print the plot to the console
	println!(
		"\n{}",
		Page::single(&view)
			.dimensions(width, height)
			.to_text()
			.unwrap()
	);

	println!(
		"{} | price: ${:.2} | last: ${:.2} | average: ${:.2} | change: {}${:.2}",
		ticker,
		current_price,
		last_price,
		average_price,
		if price_change >= 0. { '+' } else { '-' },
		price_change.abs()
	);
}

// returns a Future that resolves after `s` seconds
pub fn sleep(s: u64) -> tokio::time::Sleep {
	tokio::time::sleep(tokio::time::Duration::from_secs(s))
}

pub fn get_terminal_size() -> (u32, u32) {
	// get the width and height of the terminal
	let Size { cols: x, rows: y } = termsize::get().unwrap();

	(x as u32 / 2, y as u32 / 2)
}

pub async fn get_stock_price(uri: &str, client: &Client) -> Option<f64> {
	// send the request to receive ticker price data
	//
	// note: these headers are required to avoid NASDAQ
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
			Err(_) => return None
		};

		// remove the first character from the string
		//
		// note: you can simply use `String#remove` to remove
		// the first character, but it copies the rest of the string.
		// with this method, I'm converting the string to an iterator,
		// skipping the first byte (first character in this case, as
		// stock tickers can only use letters of the alphabet), and collecting
		// it into a vector
		let raw: Vec<u8> = json.data.primaryData.lastSalePrice
			.into_bytes()
			.into_iter()
			.skip(1)
			.collect::<Vec<u8>>();

		// parse the vec into a string, then into a
		// 64-bit float
		let price: f64 = str::from_utf8(&raw)
			.unwrap()
			.parse::<f64>()
			.unwrap();

		// return the price
		return Some(price);
	}

	// return nothing
	None
}
