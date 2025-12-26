use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[allow(unused_imports)]
use super::*;
/// These details are included in a request to create a virtual test access point (VTAP).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVtapDetails {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the {@code Vtap} resource.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN containing the {@code Vtap} resource.
    pub vcn_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the source point where packets are captured.
    pub source_id: String,

    /// The capture filter's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub capture_filter_id: String,

    /// Defined tags for this resource. Each key is predefined and scoped to a namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Operations\": {\"CostCenter\": \"42\"}}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defined_tags: Option<HashMap<String, HashMap<String, serde_json::Value>>>,

    /// A user-friendly name. Does not have to be unique, and it's changeable. Avoid entering confidential information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Free-form tags for this resource. Each tag is a simple key-value pair with no predefined name, type, or namespace. For more information, see [Resource Tags](https://docs.oracle.com/iaas/Content/General/Concepts/resourcetags.htm). <p> Example: {@code {\"Department\": \"Finance\"}}
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freeform_tags: Option<HashMap<String, String>>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the destination resource where mirrored packets are sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<String>,

    /// The IP address of the destination resource where mirrored packets are sent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_ip: Option<String>,

    /// Defines an encapsulation header type for the VTAP's mirrored traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encapsulation_protocol: Option<CreateVtapDetailsEncapsulationProtocol>,

    /// The virtual extensible LAN (VXLAN) network identifier (or VXLAN segment ID) that uniquely identifies the VXLAN. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vxlan_network_identifier: Option<i64>,

    /// Used to start or stop a {@code Vtap} resource. <p> {@code TRUE} directs the VTAP to start mirroring traffic. * {@code FALSE} (Default) directs the VTAP to stop mirroring traffic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_vtap_enabled: Option<bool>,

    /// The source type for the VTAP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<CreateVtapDetailsSourceType>,

    /// Used to control the priority of traffic. It is an optional field. If it not passed, the value is DEFAULT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traffic_mode: Option<CreateVtapDetailsTrafficMode>,

    /// The maximum size of the packets to be included in the filter. Note: Numbers greater than Number.MAX_SAFE_INTEGER will result in rounding issues.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_packet_size: Option<i64>,

    /// The target type for the VTAP.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_type: Option<CreateVtapDetailsTargetType>,

    /// The IP Address of the source private endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_private_endpoint_ip: Option<String>,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the subnet that source private endpoint belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_private_endpoint_subnet_id: Option<String>,
}

/// Required fields for CreateVtapDetails
pub struct CreateVtapDetailsRequired {
    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the compartment containing the {@code Vtap} resource.
    pub compartment_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the VCN containing the {@code Vtap} resource.
    pub vcn_id: String,

    /// The [OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm) of the source point where packets are captured.
    pub source_id: String,

    /// The capture filter's Oracle ID ([OCID](https://docs.oracle.com/iaas/Content/General/Concepts/identifiers.htm)).
    pub capture_filter_id: String,
}

impl CreateVtapDetails {
    /// Create a new CreateVtapDetails with required fields
    pub fn new(required: CreateVtapDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,

            vcn_id: required.vcn_id,

            source_id: required.source_id,

            capture_filter_id: required.capture_filter_id,

            defined_tags: None,

            display_name: None,

            freeform_tags: None,

            target_id: None,

            target_ip: None,

            encapsulation_protocol: None,

            vxlan_network_identifier: None,

            is_vtap_enabled: None,

            source_type: None,

            traffic_mode: None,

            max_packet_size: None,

            target_type: None,

            source_private_endpoint_ip: None,

            source_private_endpoint_subnet_id: None,
        }
    }

    /// Set compartment_id
    pub fn set_compartment_id(mut self, value: String) -> Self {
        self.compartment_id = value;
        self
    }

    /// Set vcn_id
    pub fn set_vcn_id(mut self, value: String) -> Self {
        self.vcn_id = value;
        self
    }

    /// Set defined_tags
    pub fn set_defined_tags(
        mut self,
        value: Option<HashMap<String, HashMap<String, serde_json::Value>>>,
    ) -> Self {
        self.defined_tags = value;
        self
    }

    /// Set display_name
    pub fn set_display_name(mut self, value: Option<String>) -> Self {
        self.display_name = value;
        self
    }

    /// Set freeform_tags
    pub fn set_freeform_tags(mut self, value: Option<HashMap<String, String>>) -> Self {
        self.freeform_tags = value;
        self
    }

    /// Set source_id
    pub fn set_source_id(mut self, value: String) -> Self {
        self.source_id = value;
        self
    }

    /// Set target_id
    pub fn set_target_id(mut self, value: Option<String>) -> Self {
        self.target_id = value;
        self
    }

    /// Set target_ip
    pub fn set_target_ip(mut self, value: Option<String>) -> Self {
        self.target_ip = value;
        self
    }

    /// Set capture_filter_id
    pub fn set_capture_filter_id(mut self, value: String) -> Self {
        self.capture_filter_id = value;
        self
    }

    /// Set encapsulation_protocol
    pub fn set_encapsulation_protocol(
        mut self,
        value: Option<CreateVtapDetailsEncapsulationProtocol>,
    ) -> Self {
        self.encapsulation_protocol = value;
        self
    }

    /// Set vxlan_network_identifier
    pub fn set_vxlan_network_identifier(mut self, value: Option<i64>) -> Self {
        self.vxlan_network_identifier = value;
        self
    }

    /// Set is_vtap_enabled
    pub fn set_is_vtap_enabled(mut self, value: Option<bool>) -> Self {
        self.is_vtap_enabled = value;
        self
    }

    /// Set source_type
    pub fn set_source_type(mut self, value: Option<CreateVtapDetailsSourceType>) -> Self {
        self.source_type = value;
        self
    }

    /// Set traffic_mode
    pub fn set_traffic_mode(mut self, value: Option<CreateVtapDetailsTrafficMode>) -> Self {
        self.traffic_mode = value;
        self
    }

    /// Set max_packet_size
    pub fn set_max_packet_size(mut self, value: Option<i64>) -> Self {
        self.max_packet_size = value;
        self
    }

    /// Set target_type
    pub fn set_target_type(mut self, value: Option<CreateVtapDetailsTargetType>) -> Self {
        self.target_type = value;
        self
    }

    /// Set source_private_endpoint_ip
    pub fn set_source_private_endpoint_ip(mut self, value: Option<String>) -> Self {
        self.source_private_endpoint_ip = value;
        self
    }

    /// Set source_private_endpoint_subnet_id
    pub fn set_source_private_endpoint_subnet_id(mut self, value: Option<String>) -> Self {
        self.source_private_endpoint_subnet_id = value;
        self
    }

    /// Set defined_tags (unwraps Option)
    pub fn with_defined_tags(
        mut self,
        value: HashMap<String, HashMap<String, serde_json::Value>>,
    ) -> Self {
        self.defined_tags = Some(value);
        self
    }

    /// Set display_name (unwraps Option)
    pub fn with_display_name(mut self, value: impl Into<String>) -> Self {
        self.display_name = Some(value.into());
        self
    }

    /// Set freeform_tags (unwraps Option)
    pub fn with_freeform_tags(mut self, value: HashMap<String, String>) -> Self {
        self.freeform_tags = Some(value);
        self
    }

    /// Set target_id (unwraps Option)
    pub fn with_target_id(mut self, value: impl Into<String>) -> Self {
        self.target_id = Some(value.into());
        self
    }

    /// Set target_ip (unwraps Option)
    pub fn with_target_ip(mut self, value: impl Into<String>) -> Self {
        self.target_ip = Some(value.into());
        self
    }

    /// Set encapsulation_protocol (unwraps Option)
    pub fn with_encapsulation_protocol(
        mut self,
        value: CreateVtapDetailsEncapsulationProtocol,
    ) -> Self {
        self.encapsulation_protocol = Some(value);
        self
    }

    /// Set vxlan_network_identifier (unwraps Option)
    pub fn with_vxlan_network_identifier(mut self, value: i64) -> Self {
        self.vxlan_network_identifier = Some(value);
        self
    }

    /// Set is_vtap_enabled (unwraps Option)
    pub fn with_is_vtap_enabled(mut self, value: bool) -> Self {
        self.is_vtap_enabled = Some(value);
        self
    }

    /// Set source_type (unwraps Option)
    pub fn with_source_type(mut self, value: CreateVtapDetailsSourceType) -> Self {
        self.source_type = Some(value);
        self
    }

    /// Set traffic_mode (unwraps Option)
    pub fn with_traffic_mode(mut self, value: CreateVtapDetailsTrafficMode) -> Self {
        self.traffic_mode = Some(value);
        self
    }

    /// Set max_packet_size (unwraps Option)
    pub fn with_max_packet_size(mut self, value: i64) -> Self {
        self.max_packet_size = Some(value);
        self
    }

    /// Set target_type (unwraps Option)
    pub fn with_target_type(mut self, value: CreateVtapDetailsTargetType) -> Self {
        self.target_type = Some(value);
        self
    }

    /// Set source_private_endpoint_ip (unwraps Option)
    pub fn with_source_private_endpoint_ip(mut self, value: impl Into<String>) -> Self {
        self.source_private_endpoint_ip = Some(value.into());
        self
    }

    /// Set source_private_endpoint_subnet_id (unwraps Option)
    pub fn with_source_private_endpoint_subnet_id(mut self, value: impl Into<String>) -> Self {
        self.source_private_endpoint_subnet_id = Some(value.into());
        self
    }
}
