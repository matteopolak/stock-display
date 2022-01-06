use reqwest::Client;
use std::collections::VecDeque;

mod constants;
mod structs;
mod utils;

#[tokio::main]
async fn main() -> () {
	let terminal = console::Term::stdout();

	// get the stock ticker from the user
	// of 5 bytes, which is the maximum length of a ticker
	let ticker: String = utils::get_input_string("      stock ticker: ", 5).to_uppercase();

	// clear the terminal
	terminal.clear_screen().unwrap();

	// hide the cursor
	terminal.hide_cursor().unwrap();

	// create a new http client from which to dispatch requests
	let client: Client = Client::builder()
		.min_tls_version(reqwest::tls::Version::TLS_1_2)
		.build()
		.unwrap();

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

	loop {
		let price: f64 = match utils::get_stock_price(&uri, &client).await {
			Some(p) => p,
			None => {
				println!("      invalid stock ticker");

				break;
			}
		};

		let (x, y) = utils::get_terminal_size();

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

		// move cursor to top left
		terminal.move_cursor_to(0, 0).unwrap();

		// print out the price
		utils::pretty_print_data(
			&ticker,
			&points,
			price,
			last_price,
			total_price / i as f64,
			x as u32,
			y as u32,
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

		// wait 60 seconds (NASDAQ real-time API updates every minute)
		utils::sleep(60).await;
	}
}
