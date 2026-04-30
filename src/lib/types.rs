use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Currency {
    pub chain_id: Option<u64>,
    pub address: Option<String>,
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub decimals: Option<u8>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeBreakdown {
    pub currency: Option<Currency>,
    pub amount: Option<String>,
    pub amount_formatted: Option<String>,
    pub amount_usd: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDetails {
    pub sender: Option<String>,
    pub recipient: Option<String>,
    pub currency_in: Option<CurrencyAmount>,
    pub currency_out: Option<CurrencyAmount>,
    pub total_impact: Option<Impact>,
    pub time_estimate: Option<f64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyAmount {
    pub currency: Option<Currency>,
    pub amount: Option<String>,
    pub amount_formatted: Option<String>,
    pub amount_usd: Option<String>,
    pub minimum_amount: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Impact {
    pub percent: Option<String>,
    pub usd: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TxHash {
    pub tx_hash: String,
    pub chain_id: u64,
    pub is_batch_tx: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionData {
    pub chain_id: Option<u64>,
    pub data: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub gas: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignData {
    pub signature_kind: Option<String>,
    pub domain: Option<serde_json::Value>,
    pub types: Option<serde_json::Value>,
    pub primary_type: Option<String>,
    pub value: Option<serde_json::Value>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PostData {
    pub body: Option<serde_json::Value>,
    pub method: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepItemData {
    pub sign: Option<SignData>,
    pub post: Option<PostData>,
    // transaction fields
    pub chain_id: Option<u64>,
    pub data: Option<serde_json::Value>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub value: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub gas: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckApi {
    pub endpoint: Option<String>,
    pub method: Option<String>,
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepItem {
    pub status: String,
    pub data: Option<StepItemData>,
    pub tx_hashes: Option<Vec<TxHash>>,
    pub internal_tx_hashes: Option<Vec<TxHash>>,
    pub check_status: Option<String>,
    pub progress_state: Option<String>,
    pub check: Option<CheckApi>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub id: Option<String>,
    pub action: String,
    pub description: String,
    pub kind: StepKind,
    pub items: Vec<StepItem>,
    pub error: Option<String>,
    pub request_id: Option<String>,
    pub deposit_address: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StepKind {
    Transaction,
    Signature,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Execute {
    pub steps: Vec<Step>,
    pub fees: Option<serde_json::Value>,
    pub breakdown: Option<serde_json::Value>,
    pub details: Option<QuoteDetails>,
    pub errors: Option<Vec<ExecuteError>>,
    pub error: Option<serde_json::Value>,
    pub refunded: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteError {
    pub message: Option<String>,
    pub order_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChain {
    pub id: u64,
    pub name: String,
    pub display_name: Option<String>,
    pub icon: Option<ChainIcon>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainIcon {
    pub dark: Option<String>,
    pub light: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainsResponse {
    pub chains: Vec<RelayChain>,
}
