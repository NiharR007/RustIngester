use anyhow::Result;
use rust_ingester::etl::parser::ParsedTriplet;
use rust_ingester::{ingest::ingest_triplet, retrieve::query_similar};

/// Main entry point for the Rust Ingestor CLI.
///
/// This function ingests an example triplet and then queries for similar triplets.
///
/// This CLI should have an exposed entry point to ingest triplets as they stream.
/// Other than this I need to figure out the working of LLM to generate triplets embeddings
/// I can use streaming to make it easy over RAM.
/// Need to figure out how the LLM will use the AGE and LSH to extract and retrieve relevant contents to answer queries.
/// This rust package is working as an Ingestor will develop another one to work as a retriever.
/// Have to check for simultaneous read and write and handle it.
///
#[tokio::main]
async fn main() -> Result<()> {
    // Example triplet ----------------------------------------------
    let t = ParsedTriplet {
        id: 1,
        subject: "alice".into(),    // fill proper fields
        relationship: "AUTHORED_BY".into(),
        object: "email_123".into(),
        ..Default::default()
    };

    // Ingest it
    ingest_triplet(t).await?;

    // Query similar
    let results = query_similar("alice email", 5).await?;
    for (triplet_id, dist) in results {
        println!("id {triplet_id}  -> distance {dist}");
    }
    Ok(())
}