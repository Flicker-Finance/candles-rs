#[derive(Debug, Clone)]
pub struct ProcessedSwap {
    pub timestamp: i64,
    pub price: f64,
    pub volume_usd: f64,
}
