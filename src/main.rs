mod constants;
mod utils;
mod structs;

#[tokio::main]
async fn main() -> () {
	// get the stock ticker from the user
	let ticker = utils::get_stock_ticker();

	// initialize the last price to be 0
	let mut last_price: f64 = 0.0;
	let mut first = true;

	loop {
		let price: f64 = match utils::get_stock_price(&ticker).await {
			Some(p) => p,
			None => break
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