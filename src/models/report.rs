
use clickhouse::Row;
use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Row)]
pub struct ReportRow {
    pub period: String,
    pub product_name: String,
    pub total_units_sold: u64,
    pub total_sales_amount: f64,
}

#[derive(Debug, Deserialize)]
pub struct ReportPath {
    pub report_type: String,
}