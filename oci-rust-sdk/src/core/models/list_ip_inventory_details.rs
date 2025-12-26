use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use super::*;
/// Required input parameters for retrieving IP Inventory data within the specified compartments of a region.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListIpInventoryDetails {
    /// Lists the selected regions.
    pub region_list: Vec<String>,

    /// List the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartments.
    pub compartment_list: Vec<String>,

    /// List of selected filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_filters: Option<bool>,

    /// The CIDR utilization of a VCN. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub utilization: Option<i64>,

    /// List of overlapping VCNs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlapping_vcns_only: Option<bool>,

    /// List of IP address types used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_type_list: Option<Vec<AddressType>>,

    /// List of VCN resource types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_type_list: Option<Vec<ListIpInventoryDetailsResourceTypeList>>,

    /// Filters the results for the specified string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_keyword: Option<String>,

    /// Provide the sort order ({@code sortOrder}) to sort the fields such as TIMECREATED in descending or descending order, and DISPLAYNAME in case sensitive. <p> *Note:** For some \"List\" operations (for example, {@code ListInstances}), sort resources by an availability domain when the resources belong to a single availability domain. If you sort the \"List\" operations without specifying an availability domain, the resources are grouped by availability domains and then sorted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<ListIpInventoryDetailsSortBy>,

    /// Specifies the sort order to use. Select either ascending ({@code ASC}) or descending ({@code DESC}) order. The DISPLAYNAME sort order is case sensitive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<ListIpInventoryDetailsSortOrder>,

    /// Most List operations paginate results. Results are paginated for the ListInstances operations. When you call a paginated List operation, the response indicates more pages of results by including the opc-next-page header. For more information, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_offset: Option<i64>,

    /// Specifies the maximum number of results displayed per page for a paginated \"List\" call. For more information, see [List Pagination](https://docs.oracle.com/iaas/Content/API/Concepts/usingapi.htm#nine). Example: {@code 50} Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination_limit: Option<i64>,
}

/// Required fields for ListIpInventoryDetails
pub struct ListIpInventoryDetailsRequired {
    /// Lists the selected regions.
    pub region_list: Vec<String>,

    /// List the [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartments.
    pub compartment_list: Vec<String>,
}

impl ListIpInventoryDetails {
    /// Create a new ListIpInventoryDetails with required fields
    pub fn new(required: ListIpInventoryDetailsRequired) -> Self {
        Self {
            region_list: required.region_list,

            compartment_list: required.compartment_list,

            override_filters: None,

            utilization: None,

            overlapping_vcns_only: None,

            address_type_list: None,

            resource_type_list: None,

            search_keyword: None,

            sort_by: None,

            sort_order: None,

            pagination_offset: None,

            pagination_limit: None,
        }
    }

    /// Set region_list
    pub fn set_region_list(mut self, value: Vec<String>) -> Self {
        self.region_list = value;
        self
    }

    /// Set compartment_list
    pub fn set_compartment_list(mut self, value: Vec<String>) -> Self {
        self.compartment_list = value;
        self
    }

    /// Set override_filters
    pub fn set_override_filters(mut self, value: Option<bool>) -> Self {
        self.override_filters = value;
        self
    }

    /// Set utilization
    pub fn set_utilization(mut self, value: Option<i64>) -> Self {
        self.utilization = value;
        self
    }

    /// Set overlapping_vcns_only
    pub fn set_overlapping_vcns_only(mut self, value: Option<bool>) -> Self {
        self.overlapping_vcns_only = value;
        self
    }

    /// Set address_type_list
    pub fn set_address_type_list(mut self, value: Option<Vec<AddressType>>) -> Self {
        self.address_type_list = value;
        self
    }

    /// Set resource_type_list
    pub fn set_resource_type_list(
        mut self,
        value: Option<Vec<ListIpInventoryDetailsResourceTypeList>>,
    ) -> Self {
        self.resource_type_list = value;
        self
    }

    /// Set search_keyword
    pub fn set_search_keyword(mut self, value: Option<String>) -> Self {
        self.search_keyword = value;
        self
    }

    /// Set sort_by
    pub fn set_sort_by(mut self, value: Option<ListIpInventoryDetailsSortBy>) -> Self {
        self.sort_by = value;
        self
    }

    /// Set sort_order
    pub fn set_sort_order(mut self, value: Option<ListIpInventoryDetailsSortOrder>) -> Self {
        self.sort_order = value;
        self
    }

    /// Set pagination_offset
    pub fn set_pagination_offset(mut self, value: Option<i64>) -> Self {
        self.pagination_offset = value;
        self
    }

    /// Set pagination_limit
    pub fn set_pagination_limit(mut self, value: Option<i64>) -> Self {
        self.pagination_limit = value;
        self
    }

    /// Set override_filters (unwraps Option)
    pub fn with_override_filters(mut self, value: bool) -> Self {
        self.override_filters = Some(value);
        self
    }

    /// Set utilization (unwraps Option)
    pub fn with_utilization(mut self, value: i64) -> Self {
        self.utilization = Some(value);
        self
    }

    /// Set overlapping_vcns_only (unwraps Option)
    pub fn with_overlapping_vcns_only(mut self, value: bool) -> Self {
        self.overlapping_vcns_only = Some(value);
        self
    }

    /// Set address_type_list (unwraps Option)
    pub fn with_address_type_list(mut self, value: Vec<AddressType>) -> Self {
        self.address_type_list = Some(value);
        self
    }

    /// Set resource_type_list (unwraps Option)
    pub fn with_resource_type_list(
        mut self,
        value: Vec<ListIpInventoryDetailsResourceTypeList>,
    ) -> Self {
        self.resource_type_list = Some(value);
        self
    }

    /// Set search_keyword (unwraps Option)
    pub fn with_search_keyword(mut self, value: impl Into<String>) -> Self {
        self.search_keyword = Some(value.into());
        self
    }

    /// Set sort_by (unwraps Option)
    pub fn with_sort_by(mut self, value: ListIpInventoryDetailsSortBy) -> Self {
        self.sort_by = Some(value);
        self
    }

    /// Set sort_order (unwraps Option)
    pub fn with_sort_order(mut self, value: ListIpInventoryDetailsSortOrder) -> Self {
        self.sort_order = Some(value);
        self
    }

    /// Set pagination_offset (unwraps Option)
    pub fn with_pagination_offset(mut self, value: i64) -> Self {
        self.pagination_offset = Some(value);
        self
    }

    /// Set pagination_limit (unwraps Option)
    pub fn with_pagination_limit(mut self, value: i64) -> Self {
        self.pagination_limit = Some(value);
        self
    }
}
