use serde::Deserialize;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct NasdaqDataWrap {
	pub data: NasdaqData
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct NasdaqData {
	pub primaryData: NasdaqPrimaryData
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct NasdaqPrimaryData {
	pub lastSalePrice: String
}