use oci_rust_sdk::{
    auth::ConfigFileAuthProvider,
    core::{Retrier, region::Region},
    resourcesearch::{
        self, SearchDetails, SearchDetailsMatchingContextType, SearchDetailsRequired,
        SearchResourcesRequest, SearchResourcesRequestRequired,
    },
};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auth = Arc::new(ConfigFileAuthProvider::from_default()?);

    let client = resourcesearch::client(resourcesearch::ClientConfig {
        auth_provider: auth,
        region: Region::ApSeoul1,
        timeout: Duration::from_secs(30),
        retry: Retrier::new(),
    })?;

    // Example 1: Structured search query
    println!("=== Example 1: Structured Search ===");

    let search_details = SearchDetails::new(SearchDetailsRequired {
        r#type: "Structured".to_string(),
    })
    .with_matching_context_type(SearchDetailsMatchingContextType::Highlights);

    let request = SearchResourcesRequest::new(SearchResourcesRequestRequired { search_details })
        .with_limit(10);

    match client.search_resources(request).await {
        Ok(response) => {
            if let Some(items) = &response.resource_summary_collection.items {
                println!("Found {} resources", items.len());
                for resource in items {
                    println!("  - {}: {}", resource.resource_type, resource.identifier);
                    if let Some(display_name) = &resource.display_name {
                        println!("    Name: {}", display_name);
                    }
                    if let Some(state) = &resource.lifecycle_state {
                        println!("    State: {}", state);
                    }
                    if let Some(tags) = &resource.freeform_tags {
                        println!("    Tags: {:?}", tags);
                    }
                }

                if let Some(next_page) = response.opc_next_page {
                    println!("Next page token: {}", next_page);
                }
            } else {
                println!("No resources found");
            }
        }
        Err(e) => {
            eprintln!("Error searching resources: {}", e);
        }
    }

    // Example 2: Free Text Search
    println!("\n=== Example 2: Free Text Search ===");

    let free_text_search = SearchDetails::new(SearchDetailsRequired {
        r#type: "FreeText".to_string(),
    })
    .with_matching_context_type(SearchDetailsMatchingContextType::None);

    let request = SearchResourcesRequest::new(SearchResourcesRequestRequired {
        search_details: free_text_search,
    })
    .with_limit(5);

    match client.search_resources(request).await {
        Ok(response) => {
            if let Some(items) = &response.resource_summary_collection.items {
                println!("Found {} resources with free text search", items.len());
                for resource in items {
                    println!("  - {} ({})", resource.identifier, resource.resource_type);
                }
            } else {
                println!("No resources found");
            }
        }
        Err(e) => {
            eprintln!("Error searching resources: {}", e);
        }
    }

    Ok(())
}
