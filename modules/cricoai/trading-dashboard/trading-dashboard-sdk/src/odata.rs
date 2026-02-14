//! OData filter field definitions for trading-dashboard DTOs.
//!
//! These are used by the REST layer for `$filter`, `$orderby`, and `$select` support.
//! Feature-gated behind `odata`.

use modkit_odata_macros::ODataFilterable;

/// OData filter fields for recent trades.
#[derive(Debug, Clone, ODataFilterable)]
pub struct TradeRecordDto {
    #[odata(filter(kind = "I64"))]
    pub id: i32,
    #[odata(filter(kind = "String"))]
    pub symbol: String,
    #[odata(filter(kind = "String"))]
    pub side: String,
    #[odata(filter(kind = "String"))]
    pub status: String,
}
