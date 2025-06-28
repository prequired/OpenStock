use std::process::Command;

#[test]
fn test_cli_help_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check that main help includes all subcommands
    assert!(stdout.contains("add-item"), "Should include add-item command");
    assert!(stdout.contains("update"), "Should include update command");
    assert!(stdout.contains("delete-item"), "Should include delete-item command");
    assert!(stdout.contains("list-inventory"), "Should include list-inventory command");
    assert!(stdout.contains("import"), "Should include import command");
    assert!(stdout.contains("filter"), "Should include filter command");
    assert!(stdout.contains("migrate"), "Should include migrate command");
    assert!(stdout.contains("fields"), "Should include fields command");
    assert!(stdout.contains("commands"), "Should include commands command");
}

#[test]
fn test_add_item_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "add-item", "--help"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check that add-item help includes required arguments
    assert!(stdout.contains("--title"), "Should include title argument");
    assert!(stdout.contains("--price"), "Should include price argument");
    assert!(stdout.contains("--quantity"), "Should include quantity argument");
    assert!(stdout.contains("--category"), "Should include category argument");
    assert!(stdout.contains("--condition"), "Should include condition argument");
}

#[test]
fn test_filter_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "filter", "--help"])
        .output()
        .expect("Failed to execute command");
    
    assert!(output.status.success(), "Command should succeed");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Check that filter help includes filter options
    assert!(stdout.contains("--price"), "Should include price filter");
    assert!(stdout.contains("--category"), "Should include category filter");
    assert!(stdout.contains("--condition"), "Should include condition filter");
    assert!(stdout.contains("--brand"), "Should include brand filter");
    assert!(stdout.contains("--fields"), "Should include fields option");
    assert!(stdout.contains("--format"), "Should include format option");
}

#[test]
fn test_basic_command_execution() {
    // Test that commands can be executed without errors
    let commands = [
        "add-item",
        "update", 
        "delete-item",
        "list-inventory",
        "import",
        "filter",
        "migrate",
        "fields",
        "commands"
    ];
    
    for cmd in commands.iter() {
        let output = Command::new("cargo")
            .args(["run", "--", cmd, "--help"])
            .output()
            .expect(&format!("Failed to execute {}", cmd));
        
        assert!(output.status.success(), "{} command should succeed", cmd);
    }
} 