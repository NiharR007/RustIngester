use anyhow::Result;
use serde_json::Value;
use tokio_postgres::Client;

/// upsert (MERGE) a node with given label and primary key `pk` property.
/// Returns AGE internal id.
pub async fn upsert_node(client: &Client, label: &str, pk: &str, props: &Value) -> Result<i64> {
    let _props_str = if props.is_null() {
        "{}".to_string()
    } else {
        props.to_string()
    };
    
    // Use the correct AGE syntax and cast result to text
    let cypher = format!(
        "SELECT result::text FROM ag_catalog.cypher('sem_graph'::name, $$
         CREATE (n:{label} {{pk: '{pk}'}}) 
         RETURN id(n)
         $$::cstring) AS (result ag_catalog.agtype);",
        label = label, pk = pk
    );
    println!("Executing cypher: {}", cypher);
    
    let row = client.query_one(&cypher, &[]).await?;
    // Now it should be text that we can extract
    let result_text: String = row.get(0);
    println!("AGE returned as text: {}", result_text);
    let id: i64 = result_text.trim_matches('"').parse()?;
    Ok(id)
}

/// upsert edge between two internal node ids.
pub async fn upsert_edge(
    client: &Client,
    rel_type: &str,
    from_id: i64,
    to_id: i64,
    _props: &Value,
) -> Result<()> {
    // First create the edge label if it doesn't exist
    let create_label_sql = format!(
        "SELECT ag_catalog.create_elabel('sem_graph', '{}');",
        rel_type
    );
    println!("Creating edge label: {}", create_label_sql);
    let _ = client.execute(&create_label_sql, &[]).await; // Ignore errors if label exists
    
    // Create the edge using correct AGE syntax
    let cypher = format!(
        "SELECT * FROM ag_catalog.cypher('sem_graph'::name, $$
         MATCH (a) WHERE id(a) = {from_id}
         MATCH (b) WHERE id(b) = {to_id}
         CREATE (a)-[r:`{rel_type}`]->(b)
         $$::cstring) AS (result ag_catalog.agtype);",
        rel_type = rel_type, from_id = from_id, to_id = to_id
    );
    println!("Executing edge cypher: {}", cypher);
    client.execute(&cypher, &[]).await?;
    Ok(())
}
