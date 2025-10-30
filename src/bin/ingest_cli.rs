use rust_ingester::ingest;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Get file path from command line args
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-json-file>", args[0]);
        eprintln!("Example: {} Data/ok.json", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    
    println!("🚀 Starting ingestion from: {}", file_path);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let start = std::time::Instant::now();
    
    match ingest::ingest_from_file(file_path).await {
        Ok(stats) => {
            println!("\n✅ Ingestion completed successfully!");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("📊 Statistics:");
            println!("   Total Sessions:   {}", stats.total_sessions);
            println!("   Total Nodes:      {}", stats.total_nodes);
            println!("   Total Edges:      {}", stats.total_edges);
            println!("   Total Embeddings: {}", stats.total_embeddings);
            println!("   Duration:         {} ms", stats.duration_ms);
            
            if !stats.errors.is_empty() {
                println!("\n⚠️  Errors encountered:");
                for error in &stats.errors {
                    println!("   - {}", error);
                }
            }
            
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            let elapsed = start.elapsed();
            println!("⏱️  Total time: {:.2}s", elapsed.as_secs_f64());
            
            Ok(())
        }
        Err(e) => {
            eprintln!("\n❌ Ingestion failed: {}", e);
            std::process::exit(1);
        }
    }
}
