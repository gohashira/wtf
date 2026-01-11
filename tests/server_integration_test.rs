use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use wtf::{Router, Server, ServerConfig};

fn create_test_site() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let root = temp_dir.path();

    // Create test site structure:
    // root.md
    // 404.md
    // home.md
    // home/
    //   home.md
    //   about.md
    //   about/
    //     about.md
    //     404.md
    //     team.md

    fs::write(root.join("root.md"), "# Welcome\n\nWelcome to the site!").unwrap();
    fs::write(root.join("404.md"), "# Not Found\n\nRoot level 404").unwrap();
    fs::write(root.join("home.md"), "# Home Page\n\nThis is home.").unwrap();

    fs::create_dir(root.join("home")).unwrap();
    fs::write(
        root.join("home/home.md"),
        "# Home Directory\n\nHome dir content.",
    )
    .unwrap();
    fs::write(root.join("home/about.md"), "# About\n\nAbout content.").unwrap();

    fs::create_dir(root.join("home/about")).unwrap();
    fs::write(
        root.join("home/about/about.md"),
        "# About Us\n\nAbout us content.",
    )
    .unwrap();
    fs::write(
        root.join("home/about/404.md"),
        "# Page Not Found\n\nCustom 404 for about section.",
    )
    .unwrap();
    fs::write(
        root.join("home/about/team.md"),
        "# Our Team\n\nMeet the team.",
    )
    .unwrap();

    temp_dir
}

#[test]
fn test_server_config_default() {
    let config = ServerConfig::new(PathBuf::from("/test"));
    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
}

#[test]
fn test_server_config_custom() {
    let config = ServerConfig::new(PathBuf::from("/test"))
        .with_host("127.0.0.1".to_string())
        .with_port(3000);

    assert_eq!(config.host, "127.0.0.1");
    assert_eq!(config.port, 3000);
}

#[test]
fn test_server_new_valid_root() {
    let temp_dir = create_test_site();
    let config = ServerConfig::new(temp_dir.path().to_path_buf());
    let server = Server::new(config);
    assert!(server.is_ok());
}

#[test]
fn test_server_new_invalid_root() {
    let config = ServerConfig::new(PathBuf::from("/nonexistent/path"));
    let server = Server::new(config);
    assert!(server.is_err());
}

#[test]
fn test_router_resolves_root() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let result = router.resolve_path("/").unwrap();
    assert!(result.is_found());
    assert!(result.path().unwrap().ends_with("root.md"));
}

#[test]
fn test_router_resolves_file() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let result = router.resolve_path("/home").unwrap();
    assert!(result.is_found());
    // Should resolve to home/home.md (directory index takes priority)
    assert!(result.path().unwrap().ends_with("home/home.md"));
}

#[test]
fn test_router_resolves_dir_index() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    // Remove home.md so it falls back to home/home.md
    fs::remove_file(temp_dir.path().join("home.md")).unwrap();

    let result = router.resolve_path("/home").unwrap();
    assert!(result.is_found());
    assert!(result.path().unwrap().ends_with("home/home.md"));
}

#[test]
fn test_router_resolves_nested() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let result = router.resolve_path("/home/about/team").unwrap();
    assert!(result.is_found());
    assert!(result.path().unwrap().ends_with("home/about/team.md"));
}

#[test]
fn test_router_404_not_found() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let result = router.resolve_path("/nonexistent").unwrap();
    assert!(!result.is_found());
}

#[test]
fn test_router_404_hierarchical_deepest() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let not_found_path = router.resolve_404("/home/about/nonexistent");
    assert!(not_found_path.is_some());
    assert!(not_found_path.unwrap().ends_with("home/about/404.md"));
}

#[test]
fn test_router_404_hierarchical_fallback() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let not_found_path = router.resolve_404("/nonexistent");
    assert!(not_found_path.is_some());
    assert!(not_found_path.unwrap().ends_with("404.md"));
}

#[test]
fn test_router_404_no_custom_page() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    // Remove all 404.md files
    fs::remove_file(temp_dir.path().join("404.md")).unwrap();
    fs::remove_file(temp_dir.path().join("home/about/404.md")).unwrap();

    let not_found_path = router.resolve_404("/home/about/nonexistent");
    assert!(not_found_path.is_none());
}

#[test]
fn test_router_security_parent_dir() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let result = router.resolve_path("/home/../etc/passwd");
    assert!(result.is_err());
}

#[test]
fn test_router_security_current_dir() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let result = router.resolve_path("/./home");
    assert!(result.is_err());
}

#[test]
fn test_router_security_trailing_slash() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let result = router.resolve_path("/home/");
    assert!(result.is_err());
}

#[test]
fn test_router_security_double_slash() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    let result = router.resolve_path("/home//about");
    assert!(result.is_err());
}

#[test]
fn test_router_path_resolution_structure() {
    let temp_dir = create_test_site();
    let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

    // Test that router correctly resolves various path structures
    let result = router.resolve_path("/home/about/team").unwrap();
    assert!(result.is_found());
    assert!(result.path().unwrap().ends_with("home/about/team.md"));

    let result = router.resolve_path("/home/about").unwrap();
    assert!(result.is_found());
    assert!(result.path().unwrap().ends_with("home/about/about.md"));
}
