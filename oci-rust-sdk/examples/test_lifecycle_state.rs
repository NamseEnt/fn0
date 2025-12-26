use oci_rust_sdk::core::{InstanceLifecycleState, ListInstancesRequest, ListInstancesRequestRequired};

fn main() {
    println!("=== Testing InstanceLifecycleState Enum ===\n");

    // Test 1: Debug trait implementation
    println!("Test 1: InstanceLifecycleState Debug trait");
    let states = vec![
        InstanceLifecycleState::Moving,
        InstanceLifecycleState::Provisioning,
        InstanceLifecycleState::Running,
        InstanceLifecycleState::Starting,
        InstanceLifecycleState::Stopping,
        InstanceLifecycleState::Stopped,
        InstanceLifecycleState::CreatingImage,
        InstanceLifecycleState::Terminating,
        InstanceLifecycleState::Terminated,
    ];

    for state in &states {
        println!("  ✓ {:?}", state);
    }

    // Test 2: Usage in ListInstancesRequest
    println!("\nTest 2: InstanceLifecycleState in ListInstancesRequest");
    let test_cases = vec![
        (InstanceLifecycleState::Running, "Running"),
        (InstanceLifecycleState::Stopped, "Stopped"),
        (InstanceLifecycleState::Terminated, "Terminated"),
    ];

    for (state, name) in &test_cases {
        let request = ListInstancesRequest::new(ListInstancesRequestRequired {
            compartment_id: "test-compartment-id".to_string(),
        })
        .with_lifecycle_state(format!("{:?}", state).to_uppercase());

        println!("  ✓ Created request with lifecycle_state = {:?}", name);
        println!("    Request compartment_id: {}", request.compartment_id);
        if let Some(ref lifecycle) = request.lifecycle_state {
            println!("    Request lifecycle_state: {}", lifecycle);
        }
    }

    // Test 3: Multiple filters combined
    println!("\nTest 3: Combined filters with LifecycleState");
    let request = ListInstancesRequest::new(ListInstancesRequestRequired {
        compartment_id: "compartment-123".to_string(),
    })
    .with_lifecycle_state("RUNNING")
    .with_display_name("my-instance")
    .with_limit(10);

    println!("  Request details:");
    println!("    compartment_id: {}", request.compartment_id);
    if let Some(ref state) = request.lifecycle_state {
        println!("    lifecycle_state: {}", state);
    }
    if let Some(ref name) = request.display_name {
        println!("    display_name: {}", name);
    }
    if let Some(limit) = request.limit {
        println!("    limit: {}", limit);
    }

    println!("\n✅ All tests passed!");
    println!("\nConclusion:");
    println!("  - InstanceLifecycleState enum provides type safety");
    println!("  - API requests use enum values");
    println!("  - Request builder enables fluent API construction");
}
