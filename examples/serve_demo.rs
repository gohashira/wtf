use std::path::PathBuf;
use wtf::{Server, ServerConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get the demo site path relative to the cargo workspace
    let demo_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("demo_site");

    // Create server configuration
    // Default: serves on 0.0.0.0:8080
    let config = ServerConfig::new(demo_path);

    // Alternatively, customize host and port:
    // let config = ServerConfig::new(demo_path)
    //     .with_host("127.0.0.1".to_string())
    //     .with_port(3000);

    // Create and run the server
    let server = Server::new(config)?;

    println!("Try visiting:");
    println!("  http://localhost:8080/");
    println!("  http://localhost:8080/home");
    println!("  http://localhost:8080/home/about");
    println!("  http://localhost:8080/home/about/team");
    println!("  http://localhost:8080/nonexistent (404 page)");
    println!();

    server.run()?;

    Ok(())
}
