use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct NasdaqDataWrap {
	pub data: NasdaqData,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct NasdaqData {
	pub primaryData: NasdaqPrimaryData,
}

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct NasdaqPrimaryData {
	pub lastSalePrice: String,
}
