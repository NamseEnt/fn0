# OCI Rust SDK - Development Guidelines

## Struct Builder Pattern Standards

All model structs in this codebase MUST follow these builder pattern rules:

### 1. Required Fields Pattern

If a struct has required (non-Optional) fields:
- Create a companion struct named `{ModelName}Required` containing only the required fields
- The `new()` method MUST accept `{ModelName}Required` as an argument
- Example:
  ```rust
  pub struct LaunchInstanceDetailsRequired {
      pub compartment_id: String,
      pub availability_domain: String,
      pub shape: String,
      pub source_details: InstanceSourceDetails,
  }

  impl LaunchInstanceDetails {
      pub fn new(required: LaunchInstanceDetailsRequired) -> Self {
          Self {
              compartment_id: required.compartment_id,
              availability_domain: required.availability_domain,
              shape: required.shape,
              source_details: required.source_details,
              // ... optional fields set to None
          }
      }
  }
  ```

If a struct has NO required fields:
- The `new()` method takes no arguments
- Example:
  ```rust
  impl SomeDetails {
      pub fn new() -> Self {
          Self {
              field1: None,
              field2: None,
              // ... all fields are None
          }
      }
  }
  ```

### 2. Setter Methods for All Fields

Every field (both required and optional) MUST have a `set_{field}` method:
```rust
pub fn set_{field}(mut self, {field}: {FieldType}) -> Self {
    self.{field} = {field};
    self
}
```

Examples:
```rust
// For required String field
pub fn set_compartment_id(mut self, compartment_id: String) -> Self {
    self.compartment_id = compartment_id;
    self
}

// For optional bool field
pub fn set_is_read_only(mut self, is_read_only: Option<bool>) -> Self {
    self.is_read_only = is_read_only;
    self
}
```

### 3. With Methods for Optional Fields Only

Every Optional field MUST have a `with_{field}` method that unwraps the Option:
```rust
pub fn with_{field}(mut self, {field}: {T}) -> Self
where FieldType = Option<T>
{
    self.{field} = Some({field});
    self
}
```

Examples:
```rust
// For Option<String> field
pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
    self.display_name = Some(display_name.into());
    self
}

// For Option<bool> field
pub fn with_is_read_only(mut self, is_read_only: bool) -> Self {
    self.is_read_only = Some(is_read_only);
    self
}
```

### 4. No Separate Builder Structs

**DO NOT** create separate `{ModelName}Builder` structs.

**REMOVE** any existing Builder structs and their `builder()` methods.

The struct itself serves as the builder using method chaining.

### Complete Example

```rust
// Required fields struct
pub struct LaunchInstanceDetailsRequired {
    pub compartment_id: String,
    pub availability_domain: String,
    pub shape: String,
    pub source_details: InstanceSourceDetails,
}

// Main struct
pub struct LaunchInstanceDetails {
    // Required fields
    pub compartment_id: String,
    pub availability_domain: String,
    pub shape: String,
    pub source_details: InstanceSourceDetails,
    // Optional fields
    pub display_name: Option<String>,
    pub is_pv_encryption_in_transit_enabled: Option<bool>,
}

impl LaunchInstanceDetails {
    // Constructor with required fields
    pub fn new(required: LaunchInstanceDetailsRequired) -> Self {
        Self {
            compartment_id: required.compartment_id,
            availability_domain: required.availability_domain,
            shape: required.shape,
            source_details: required.source_details,
            display_name: None,
            is_pv_encryption_in_transit_enabled: None,
        }
    }

    // Setters for all fields (required + optional)
    pub fn set_compartment_id(mut self, compartment_id: String) -> Self {
        self.compartment_id = compartment_id;
        self
    }

    pub fn set_display_name(mut self, display_name: Option<String>) -> Self {
        self.display_name = display_name;
        self
    }

    // With methods for optional fields only
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    pub fn with_is_pv_encryption_in_transit_enabled(mut self, enabled: bool) -> Self {
        self.is_pv_encryption_in_transit_enabled = Some(enabled);
        self
    }
}

// Usage
let instance = LaunchInstanceDetails::new(LaunchInstanceDetailsRequired {
    compartment_id: "ocid1.compartment...".to_string(),
    availability_domain: "AD-1".to_string(),
    shape: "VM.Standard2.1".to_string(),
    source_details: source,
})
.with_display_name("my-instance")
.set_compartment_id("ocid1.compartment.new...".to_string());
```

## Apply to All Models

These rules apply to:
- All model structs in `src/compute/models/`
- All model structs in `src/virtual_network/models/`
- All model structs in `src/resource_search/models/`
- All model structs in `src/container_instances/models/`
- All model structs in `src/os_management_hub/models/`
- Any other model structs in the codebase

This ensures consistency across the entire SDK.
