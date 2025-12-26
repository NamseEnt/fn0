use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::super::models::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAppCatalogSubscriptionResponse {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.AppCatalogSubscription instance.
    pub app_catalog_subscription: AppCatalogSubscription,
}

/// Required fields for CreateAppCatalogSubscriptionResponse
pub struct CreateAppCatalogSubscriptionResponseRequired {
    /// For optimistic concurrency control. See {@code if-match}.
    pub etag: String,

    /// Unique Oracle-assigned identifier for the request. If you need to contact Oracle about a particular request, please provide the request ID.
    pub opc_request_id: String,

    /// The returned model.AppCatalogSubscription instance.
    pub app_catalog_subscription: AppCatalogSubscription,
}

impl CreateAppCatalogSubscriptionResponse {
    /// Create a new CreateAppCatalogSubscriptionResponse with required fields
    pub fn new(required: CreateAppCatalogSubscriptionResponseRequired) -> Self {
        Self {
            etag: required.etag,

            opc_request_id: required.opc_request_id,

            app_catalog_subscription: required.app_catalog_subscription,
        }
    }

    /// Set etag
    pub fn set_etag(mut self, value: String) -> Self {
        self.etag = value;
        self
    }

    /// Set opc_request_id
    pub fn set_opc_request_id(mut self, value: String) -> Self {
        self.opc_request_id = value;
        self
    }

    /// Set app_catalog_subscription
    pub fn set_app_catalog_subscription(mut self, value: AppCatalogSubscription) -> Self {
        self.app_catalog_subscription = value;
        self
    }
}
