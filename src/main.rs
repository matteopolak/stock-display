use reqwest::Client;

mod constants;
mod structs;
mod utils;

#[tokio::main]
async fn main() -> () {
	// get the stock ticker from the user
	// of 5 bytes, which is the maximum length of a ticker
	let ticker: String = utils::get_input_string("      stock ticker: ", 5);

	// initialize the last price to be 0
	let mut last_price: f64 = 0.0;
	let mut first: bool = true;

	// create a new http client from which to dispatch requests
	let client: Client = Client::builder()
		.min_tls_version(reqwest::tls::Version::TLS_1_2)
		.build()
		.unwrap();

	// build the URI from the ticker name
	let uri: String = constants::NASDAQ_API_ENDPOINT.replace("{ticker}", &ticker);

	loop {
		let price: f64 = match utils::get_stock_price(&uri, &client).await {
			Some(p) => p,
			None => break,
		};

		if first {
			last_price = price;
			first = false;
		}

		// print out the price
		utils::pretty_print_data(price, last_price);

		// set `last_price` to the current price (`price`)
		last_price = price;

		// wait 60 seconds (NASDAQ real-time API updates every minute)
		utils::sleep(60).await;
	}
}
