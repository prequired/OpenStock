use std::process::Command;
use std::fs::{self, File};
use std::io::Read;
use tempfile::TempDir;

fn docker_available() -> bool {
    Command::new("docker").arg("--version").output().is_ok()
}

#[test]
fn test_docker_build() -> anyhow::Result<()> {
    if !docker_available() {
        println!("Docker not available, skipping test");
        return Ok(());
    }
    
    let output = Command::new("docker")
        .args(["build", "-t", "inventory-cli:test", "."])
        .output()?;
    assert!(output.status.success(), "Docker build failed: {:?}", output.stderr);
    Ok(())
}

#[test]
fn test_docker_run_add_item() -> anyhow::Result<()> {
    if !docker_available() {
        println!("Docker not available, skipping test");
        return Ok(());
    }
    
    let temp_dir = TempDir::new()?;
    let volume_path = temp_dir.path().join(".inventory");
    fs::create_dir_all(&volume_path)?;

    let output = Command::new("docker")
        .args([
            "run", "--rm",
            "-v", &format!("{}:/root/.inventory", volume_path.display()),
            "inventory-cli:test",
            "add-item", "--title", "Test Item", "--price", "10.0",
            "--quantity", "5", "--category", "sneakers", "--condition", "new"
        ])
        .output()?;
    assert!(output.status.success(), "Docker add-item failed: {:?}", output.stderr);

    let db_path = volume_path.join("inventory.db");
    assert!(db_path.exists(), "Database file should exist");
    Ok(())
}

#[test]
fn test_docker_run_filter() -> anyhow::Result<()> {
    if !docker_available() {
        println!("Docker not available, skipping test");
        return Ok(());
    }
    
    let temp_dir = TempDir::new()?;
    let volume_path = temp_dir.path().join(".inventory");
    fs::create_dir_all(&volume_path)?;

    // Initialize database with an item
    Command::new("docker")
        .args([
            "run", "--rm",
            "-v", &format!("{}:/root/.inventory", volume_path.display()),
            "inventory-cli:test",
            "add-item", "--title", "Test Item", "--price", "10.0",
            "--quantity", "5", "--category", "sneakers", "--condition", "new"
        ])
        .output()?;

    let output = Command::new("docker")
        .args([
            "run", "--rm",
            "-v", &format!("{}:/root/.inventory", volume_path.display()),
            "inventory-cli:test",
            "filter", "--price", "0-20", "-f", "id,title,price"
        ])
        .output()?;
    assert!(output.status.success(), "Docker filter failed: {:?}", output.stderr);
    let output_str = String::from_utf8(output.stdout)?;
    assert!(output_str.contains("Test Item"), "Filter output should contain Test Item");
    Ok(())
}

#[test]
fn test_docker_run_stats() -> anyhow::Result<()> {
    if !docker_available() {
        println!("Docker not available, skipping test");
        return Ok(());
    }
    
    let temp_dir = TempDir::new()?;
    let volume_path = temp_dir.path().join(".inventory");
    fs::create_dir_all(&volume_path)?;

    // Initialize database with an item
    Command::new("docker")
        .args([
            "run", "--rm",
            "-v", &format!("{}:/root/.inventory", volume_path.display()),
            "inventory-cli:test",
            "add-item", "--title", "Test Item", "--price", "10.0",
            "--quantity", "5", "--category", "sneakers", "--condition", "new"
        ])
        .output()?;

    let output = Command::new("docker")
        .args([
            "run", "--rm",
            "-v", &format!("{}:/root/.inventory", volume_path.display()),
            "inventory-cli:test",
            "stats", "--format", "json"
        ])
        .output()?;
    assert!(output.status.success(), "Docker stats failed: {:?}", output.stderr);
    let output_str = String::from_utf8(output.stdout)?;
    assert!(output_str.contains("Total Items"), "Stats output should contain Total Items");
    Ok(())
}

#[test]
fn test_dockerfile_exists() -> anyhow::Result<()> {
    assert!(std::path::Path::new("Dockerfile").exists(), "Dockerfile should exist");
    Ok(())
}

#[test]
fn test_ci_workflow_exists() -> anyhow::Result<()> {
    assert!(std::path::Path::new(".github/workflows/ci.yml").exists(), "CI workflow should exist");
    Ok(())
} 