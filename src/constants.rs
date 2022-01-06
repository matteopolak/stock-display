pub const NASDAQ_API_ENDPOINT: &str =
	"https://api.nasdaq.com/api/quote/{ticker}/info?assetclass=stocks";
pub const USER_AGENT_HEADER: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36";
pub const MARKETSTACK_API_ENDPOINT: &str = "http://api.marketstack.com/v1/eod?access_key=51d7cbf4ff50d6af109c7f65477b2633&symbols={ticker}&date_from={start}&date_end={end}&limit=366";
