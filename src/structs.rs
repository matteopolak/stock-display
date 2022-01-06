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

#[allow(non_snake_case)]
#[derive(Deserialize)]
pub struct NameStackDataWrap {
	pub data: Vec<NameStackData>
}

#[allow(non_snake_case)]
#[derive(Deserialize, Clone)]
pub struct NameStackData {
	pub open: f64
}