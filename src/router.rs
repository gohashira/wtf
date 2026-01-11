use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

// Constants
const ROOT_FILENAME: &str = "root.md";
const NOTFOUND_FILENAME: &str = "404.md";
const MD_EXTENSION: &str = ".md";
const PATH_SEPARATOR: char = '/';
const PARENT_DIR: &str = "..";
const CURRENT_DIR: &str = ".";
const ROOT_URL_PATH: &str = "/";

// HTML constants for footer generation
const HR_DOUBLE: &str = "<hr><hr>";
const UL_OPEN: &str = "<ul>";
const UL_CLOSE: &str = "</ul>";
const LI_OPEN: &str = "<li>";
const LI_CLOSE: &str = "</li>";
const A_HREF_OPEN: &str = "<a href=\"";
const A_HREF_MIDDLE: &str = "\">";
const A_CLOSE: &str = "</a>";

// ============================================================================
// ERROR HANDLING
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum RouterError {
    InvalidContentRoot { path: PathBuf, reason: String },
    InvalidPath { path: String, reason: String },
    IoError { path: PathBuf, message: String },
}

impl fmt::Display for RouterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RouterError::InvalidContentRoot { path, reason } => {
                write!(f, "Invalid content root '{}': {}", path.display(), reason)
            }
            RouterError::InvalidPath { path, reason } => {
                write!(f, "Invalid path '{}': {}", path, reason)
            }
            RouterError::IoError { path, message } => {
                write!(f, "IO error for '{}': {}", path.display(), message)
            }
        }
    }
}

impl std::error::Error for RouterError {}

// ============================================================================
// SITEMAP ENTRY
// ============================================================================

/// Represents a single entry in the site's hierarchical sitemap
#[derive(Debug, Clone, PartialEq)]
pub struct SitemapEntry {
    /// Display name for the entry (derived from filename)
    pub name: String,
    /// URL path for the entry (e.g., "/home", "/home/about")
    pub url_path: String,
    /// Child entries (nested pages)
    pub children: Vec<SitemapEntry>,
}

impl SitemapEntry {
    /// Creates a new sitemap entry
    fn new(name: String, url_path: String) -> Self {
        Self {
            name,
            url_path,
            children: Vec::new(),
        }
    }
}

// ============================================================================
// RESOLVED PATH
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedPath {
    Found(PathBuf),
    NotFound { attempted_paths: Vec<PathBuf> },
}

impl ResolvedPath {
    pub fn is_found(&self) -> bool {
        matches!(self, ResolvedPath::Found(_))
    }

    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            ResolvedPath::Found(path) => Some(path),
            ResolvedPath::NotFound { .. } => None,
        }
    }
}

// ============================================================================
// ROUTER
// ============================================================================

/// Directory-based router for markdown files
///
/// Converts URL paths to filesystem paths following these rules:
/// - `/` → `<content_root>/root.md`
/// - `/home` → `<content_root>/home.md` OR `<content_root>/home/home.md`
/// - `/home/about` → `<content_root>/home/about.md` OR `<content_root>/home/about/about.md`
///
/// Security: Validates paths to prevent directory traversal attacks
pub struct Router {
    content_root: PathBuf,
}

impl Router {
    /// Creates a new router with the specified content root directory
    ///
    /// # Arguments
    /// * `content_root` - Absolute path to the directory containing markdown files
    ///
    /// # Returns
    /// * `Ok(Router)` - Successfully created router
    /// * `Err(RouterError)` - If content_root doesn't exist or isn't a directory
    pub fn new(content_root: PathBuf) -> Result<Self, RouterError> {
        // Validate content root exists
        if !content_root.exists() {
            return Err(RouterError::InvalidContentRoot {
                path: content_root.clone(),
                reason: "path does not exist".to_string(),
            });
        }

        // Validate content root is a directory
        if !content_root.is_dir() {
            return Err(RouterError::InvalidContentRoot {
                path: content_root.clone(),
                reason: "path is not a directory".to_string(),
            });
        }

        // Canonicalize to get absolute path
        let content_root = content_root
            .canonicalize()
            .map_err(|e| RouterError::IoError {
                path: content_root.clone(),
                message: e.to_string(),
            })?;

        Ok(Self { content_root })
    }

    /// Resolves a URL path to a filesystem path
    ///
    /// # Arguments
    /// * `url_path` - The URL path (e.g., "/home/about")
    ///
    /// # Returns
    /// * `ResolvedPath::Found(PathBuf)` - If a matching file was found
    /// * `ResolvedPath::NotFound` - If no matching file was found, with attempted paths
    pub fn resolve_path(&self, url_path: &str) -> Result<ResolvedPath, RouterError> {
        // Sanitize and validate the path
        let sanitized = self.sanitize_path(url_path)?;

        // Handle root path specially
        if sanitized.is_empty() {
            let root_file = self.content_root.join(ROOT_FILENAME);
            return if root_file.exists() && root_file.is_file() {
                Ok(ResolvedPath::Found(root_file))
            } else {
                Ok(ResolvedPath::NotFound {
                    attempted_paths: vec![root_file],
                })
            };
        }

        // Try to resolve as directory index or file
        // Priority: Directory index has higher priority than direct file
        let mut attempted_paths = Vec::new();

        // Check if this path would be accessing a directory's index file explicitly
        // E.g., /home/home should NOT resolve to home/home.md
        // because /home already serves that file
        let components: Vec<&str> = sanitized.split(PATH_SEPARATOR).collect();
        if components.len() >= 2 {
            let last = components.last().unwrap();
            let second_last = components[components.len() - 2];

            // If the last two components are the same (e.g., "home/home"),
            // this is an attempt to access an index file explicitly - reject it
            if *last == second_last {
                return Ok(ResolvedPath::NotFound {
                    attempted_paths: vec![],
                });
            }
        }

        // Try 1: Directory index (e.g., /home → home/home.md)
        // This takes priority to avoid ambiguity
        let dir_index_path = self.build_dir_index_path(&sanitized);
        attempted_paths.push(dir_index_path.clone());
        if dir_index_path.exists() && dir_index_path.is_file() {
            return Ok(ResolvedPath::Found(dir_index_path));
        }

        // Try 2: Direct file (e.g., /home → home.md)
        // Only if no directory index exists
        let file_path = self.build_file_path(&sanitized);
        attempted_paths.push(file_path.clone());
        if file_path.exists() && file_path.is_file() {
            return Ok(ResolvedPath::Found(file_path));
        }

        // Not found
        Ok(ResolvedPath::NotFound { attempted_paths })
    }

    /// Resolves a 404.md file with hierarchical fallback
    ///
    /// Searches for 404.md files from the requested path up to the root:
    /// 1. /home/about/me → home/about/me/404.md
    /// 2. /home/about/me → home/about/404.md
    /// 3. /home/about/me → home/404.md
    /// 4. /home/about/me → 404.md (root fallback)
    ///
    /// # Arguments
    /// * `url_path` - The original URL path that was not found
    ///
    /// # Returns
    /// * `Some(PathBuf)` - If a 404.md file was found in the hierarchy
    /// * `None` - If no 404.md files exist
    pub fn resolve_404(&self, url_path: &str) -> Option<PathBuf> {
        // Sanitize path (ignore errors, just return None)
        let sanitized = self.sanitize_path(url_path).ok()?;

        // Build path components
        let components: Vec<&str> = if sanitized.is_empty() {
            vec![]
        } else {
            sanitized.split(PATH_SEPARATOR).collect()
        };

        // Try from deepest to shallowest
        // Start with the full path, then remove one component at a time
        for depth in (0..=components.len()).rev() {
            let mut path = self.content_root.clone();

            // Add components up to current depth
            for component in components.iter().take(depth) {
                path.push(component);
            }

            // Add 404.md
            path.push(NOTFOUND_FILENAME);

            if path.exists() && path.is_file() {
                return Some(path);
            }
        }

        None
    }

    /// Returns the content root directory
    pub fn content_root(&self) -> &Path {
        &self.content_root
    }

    /// Builds a hierarchical sitemap by scanning the content directory
    ///
    /// The sitemap includes:
    /// - root.md as "Root" at "/"
    /// - All other .md files except 404.md
    /// - Entries sorted alphabetically at each level
    ///
    /// # Returns
    /// * `Ok(Vec<SitemapEntry>)` - List of top-level sitemap entries
    /// * `Err(RouterError)` - If directory traversal fails
    pub fn build_sitemap(&self) -> Result<Vec<SitemapEntry>, RouterError> {
        let mut entries = Vec::new();

        // Add root entry if root.md exists
        let root_file = self.content_root.join(ROOT_FILENAME);
        if root_file.exists() && root_file.is_file() {
            entries.push(SitemapEntry::new(
                ROOT_FILENAME
                    .strip_suffix(MD_EXTENSION)
                    .unwrap_or(&ROOT_FILENAME)
                    .to_string(),
                ROOT_URL_PATH.to_string(),
            ));
        }

        // Scan content root directory recursively (no index file to skip at root)
        self.scan_directory(&self.content_root, "", &mut entries, None)?;

        Ok(entries)
    }

    // ========================================================================
    // PRIVATE HELPER METHODS
    // ========================================================================

    /// Sanitizes a URL path and validates it for security
    ///
    /// - Removes leading and trailing slashes
    /// - Validates no ".." or "." components (directory traversal prevention)
    /// - Validates no trailing slashes
    fn sanitize_path(&self, url_path: &str) -> Result<String, RouterError> {
        // Check for trailing slash (not allowed per spec)
        if url_path.len() > 1 && url_path.ends_with(PATH_SEPARATOR) {
            return Err(RouterError::InvalidPath {
                path: url_path.to_string(),
                reason: "trailing slashes are not allowed".to_string(),
            });
        }

        // Remove leading slash
        let path = url_path.trim_start_matches(PATH_SEPARATOR);

        // Handle empty path (root)
        if path.is_empty() {
            return Ok(String::new());
        }

        // Split into components and validate
        let components: Vec<&str> = path.split(PATH_SEPARATOR).collect();

        for component in &components {
            // Check for parent directory traversal
            if *component == PARENT_DIR {
                return Err(RouterError::InvalidPath {
                    path: url_path.to_string(),
                    reason: "path contains '..' component".to_string(),
                });
            }

            // Check for current directory reference
            if *component == CURRENT_DIR {
                return Err(RouterError::InvalidPath {
                    path: url_path.to_string(),
                    reason: "path contains '.' component".to_string(),
                });
            }

            // Check for empty components (double slashes)
            if component.is_empty() {
                return Err(RouterError::InvalidPath {
                    path: url_path.to_string(),
                    reason: "path contains empty component (double slash)".to_string(),
                });
            }
        }

        Ok(path.to_string())
    }

    /// Builds a file path from sanitized path components
    /// Example: "home/about" → <content_root>/home/about.md
    fn build_file_path(&self, sanitized_path: &str) -> PathBuf {
        let mut path = self.content_root.clone();
        path.push(format!("{}{}", sanitized_path, MD_EXTENSION));
        path
    }

    /// Builds a directory index path from sanitized path components
    /// Example: "home/about" → <content_root>/home/about/about.md
    fn build_dir_index_path(&self, sanitized_path: &str) -> PathBuf {
        let mut path = self.content_root.clone();

        // Get the last component (directory name)
        let last_component = sanitized_path
            .split(PATH_SEPARATOR)
            .last()
            .unwrap_or(sanitized_path);

        // Build path: parent_dirs/last_component/last_component.md
        path.push(sanitized_path);
        path.push(format!("{}{}", last_component, MD_EXTENSION));
        path
    }

    /// Recursively scans a directory and builds sitemap entries
    ///
    /// # Arguments
    /// * `dir_path` - Absolute path to directory to scan
    /// * `url_prefix` - URL prefix for entries in this directory (e.g., "home", "home/about")
    /// * `entries` - Vector to accumulate sitemap entries
    /// * `index_filename` - Optional name of the directory's index file to skip (e.g., "home.md")
    fn scan_directory(
        &self,
        dir_path: &Path,
        url_prefix: &str,
        entries: &mut Vec<SitemapEntry>,
        index_filename: Option<&str>,
    ) -> Result<(), RouterError> {
        // Read directory entries
        let dir_entries = fs::read_dir(dir_path).map_err(|e| RouterError::IoError {
            path: dir_path.to_path_buf(),
            message: e.to_string(),
        })?;

        for entry in dir_entries {
            let entry = entry.map_err(|e| RouterError::IoError {
                path: dir_path.to_path_buf(),
                message: e.to_string(),
            })?;

            let path = entry.path();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            if path.is_file() && file_name_str.ends_with(MD_EXTENSION) {
                // Skip 404.md and root.md (root is handled separately)
                if file_name_str == NOTFOUND_FILENAME || file_name_str == ROOT_FILENAME {
                    continue;
                }

                // Skip the directory's index file (e.g., skip "home.md" when scanning inside "home/" directory)
                if let Some(idx_filename) = index_filename {
                    if file_name_str == idx_filename {
                        continue;
                    }
                }

                // Extract name without extension
                let name = file_name_str
                    .strip_suffix(MD_EXTENSION)
                    .unwrap_or(&file_name_str)
                    .to_string();

                // Check if a directory with the same name exists and has an index file
                // If so, skip this standalone file to avoid duplicate routes
                let potential_dir = dir_path.join(&name);
                let potential_index = potential_dir.join(file_name_str.as_ref());
                if potential_dir.is_dir() && potential_index.exists() && potential_index.is_file() {
                    // Directory with index file exists - skip standalone file
                    continue;
                }

                // Build URL path
                let url_path = if url_prefix.is_empty() {
                    format!("/{}", name)
                } else {
                    format!("{}/{}", url_prefix, name)
                };

                entries.push(SitemapEntry::new(name, url_path));
            } else if path.is_dir() {
                // Check if this directory has a corresponding index file
                let dir_name = file_name_str.to_string();
                let index_file_name = format!("{}{}", dir_name, MD_EXTENSION);
                let index_path = path.join(&index_file_name);

                if index_path.exists() && index_path.is_file() {
                    // This directory has an index file, create entry with children
                    let url_path = if url_prefix.is_empty() {
                        format!("/{}", dir_name)
                    } else {
                        format!("{}/{}", url_prefix, dir_name)
                    };

                    let mut dir_entry = SitemapEntry::new(dir_name.clone(), url_path);

                    // Recursively scan subdirectory
                    let sub_url_prefix = if url_prefix.is_empty() {
                        format!("/{}", dir_name)
                    } else {
                        format!("{}/{}", url_prefix, dir_name)
                    };

                    self.scan_directory(
                        &path,
                        &sub_url_prefix,
                        &mut dir_entry.children,
                        Some(&index_file_name),
                    )?;

                    // Sort children alphabetically
                    dir_entry.children.sort_by(|a, b| a.name.cmp(&b.name));

                    entries.push(dir_entry);
                }
            }
        }

        Ok(())
    }
}

// ============================================================================
// SITEMAP FOOTER GENERATION
// ============================================================================

/// Generates HTML footer with double HR separator and nested sitemap
///
/// # Arguments
/// * `entries` - Slice of top-level sitemap entries
/// * `current_path` - Optional current URL path to highlight with "you're here"
///
/// # Returns
/// HTML string containing:
/// - Double `<hr>` separator
/// - Nested `<ul><li>` structure with links
/// - Bold + "← you're here" indicator for current page
///
/// # Example Output
/// ```html
/// <hr><hr>
/// <ul>
///   <li><a href="/">Root</a></li>
///   <li><a href="/home"><b>home ← you're here</b></a>
///     <ul>
///       <li><a href="/home/about">about</a></li>
///     </ul>
///   </li>
/// </ul>
/// ```
pub fn generate_sitemap_footer(entries: &[SitemapEntry], current_path: Option<&str>) -> String {
    let mut output = String::new();

    // Add double HR separator
    output.push_str(HR_DOUBLE);

    // Generate nested sitemap list
    if !entries.is_empty() {
        output.push_str(&render_sitemap_list(entries, current_path));
    }

    output
}

/// Recursively renders a nested sitemap list as HTML
///
/// # Arguments
/// * `entries` - Slice of sitemap entries at current level
/// * `current_path` - Optional current URL path to highlight
///
/// # Returns
/// HTML string containing nested `<ul><li>` structure
fn render_sitemap_list(entries: &[SitemapEntry], current_path: Option<&str>) -> String {
    let mut output = String::new();

    output.push_str(UL_OPEN);

    for entry in entries {
        output.push_str(LI_OPEN);

        // Add link
        output.push_str(A_HREF_OPEN);
        output.push_str(&escape_html_attr(&entry.url_path));
        output.push_str(A_HREF_MIDDLE);

        // Check if this is the current page
        let is_current = current_path.map_or(false, |path| path == entry.url_path);

        if is_current {
            output.push_str("<b>");
            output.push_str(&escape_html_text(&entry.name));
            output.push_str(" ← you're here");
            output.push_str("</b>");
        } else {
            output.push_str(&escape_html_text(&entry.name));
        }

        output.push_str(A_CLOSE);

        // Recursively render children
        if !entry.children.is_empty() {
            output.push_str(&render_sitemap_list(&entry.children, current_path));
        }

        output.push_str(LI_CLOSE);
    }

    output.push_str(UL_CLOSE);

    output
}

/// Escapes HTML entities in attribute values (URLs)
fn escape_html_attr(content: &str) -> String {
    content
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Escapes HTML entities in text content (display names)
fn escape_html_text(content: &str) -> String {
    content
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn create_test_content_root() -> tempfile::TempDir {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();

        // Create test structure:
        // root.md
        // 404.md
        // home.md
        // home/
        //   home.md
        //   about.md
        //   about/
        //     about.md
        //     404.md
        //     me.md

        fs::write(root.join("root.md"), "Root content").unwrap();
        fs::write(root.join("404.md"), "Root 404").unwrap();
        fs::write(root.join("home.md"), "Home file").unwrap();

        fs::create_dir(root.join("home")).unwrap();
        fs::write(root.join("home/home.md"), "Home dir").unwrap();
        fs::write(root.join("home/about.md"), "About file").unwrap();

        fs::create_dir(root.join("home/about")).unwrap();
        fs::write(root.join("home/about/about.md"), "About dir").unwrap();
        fs::write(root.join("home/about/404.md"), "About 404").unwrap();
        fs::write(root.join("home/about/me.md"), "Me").unwrap();

        temp_dir
    }

    #[test]
    fn test_router_new_valid() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf());
        assert!(router.is_ok());
    }

    #[test]
    fn test_router_new_nonexistent() {
        let router = Router::new(PathBuf::from("/nonexistent/path"));
        assert!(matches!(
            router,
            Err(RouterError::InvalidContentRoot { .. })
        ));
    }

    #[test]
    fn test_resolve_root_path() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_path("/").unwrap();
        assert!(result.is_found());
        assert!(result.path().unwrap().ends_with("root.md"));
    }

    #[test]
    fn test_resolve_file_path() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_path("/home").unwrap();
        assert!(result.is_found());
        assert!(result.path().unwrap().ends_with("home.md"));
    }

    #[test]
    fn test_resolve_dir_index_path() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        // Remove home.md so it falls back to home/home.md
        fs::remove_file(temp_dir.path().join("home.md")).unwrap();

        let result = router.resolve_path("/home").unwrap();
        assert!(result.is_found());
        assert!(result.path().unwrap().ends_with("home/home.md"));
    }

    #[test]
    fn test_resolve_nested_file() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_path("/home/about/me").unwrap();
        assert!(result.is_found());
        assert!(result.path().unwrap().ends_with("home/about/me.md"));
    }

    #[test]
    fn test_resolve_not_found() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_path("/nonexistent").unwrap();
        assert!(matches!(result, ResolvedPath::NotFound { .. }));
    }

    #[test]
    fn test_sanitize_path_parent_dir() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_path("/home/../etc/passwd");
        assert!(matches!(result, Err(RouterError::InvalidPath { .. })));
    }

    #[test]
    fn test_sanitize_path_trailing_slash() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_path("/home/");
        assert!(matches!(result, Err(RouterError::InvalidPath { .. })));
    }

    #[test]
    fn test_resolve_404_deepest() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_404("/home/about/nonexistent");
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("home/about/404.md"));
    }

    #[test]
    fn test_resolve_404_fallback_to_root() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_404("/nonexistent");
        assert!(result.is_some());
        assert!(result.unwrap().ends_with("404.md"));
    }

    #[test]
    fn test_resolve_404_not_found() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        // Remove all 404.md files
        fs::remove_file(temp_dir.path().join("404.md")).unwrap();
        fs::remove_file(temp_dir.path().join("home/about/404.md")).unwrap();

        let result = router.resolve_404("/home/about/nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_build_sitemap_includes_root() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let sitemap = router.build_sitemap().unwrap();

        // Check that root entry exists
        let root_entry = sitemap.iter().find(|e| e.url_path == "/");
        assert!(root_entry.is_some());
        assert_eq!(root_entry.unwrap().name, "root");
    }

    #[test]
    fn test_build_sitemap_excludes_404_files() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let sitemap = router.build_sitemap().unwrap();

        // Recursively check that no entry has name "404"
        fn has_404_entry(entries: &[SitemapEntry]) -> bool {
            for entry in entries {
                if entry.name == "404" {
                    return true;
                }
                if has_404_entry(&entry.children) {
                    return true;
                }
            }
            false
        }

        assert!(!has_404_entry(&sitemap));
    }

    #[test]
    fn test_build_sitemap_alphabetical_order() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let sitemap = router.build_sitemap().unwrap();

        // Find home entry (should have children)
        let home_entry = sitemap.iter().find(|e| e.name == "home");
        assert!(home_entry.is_some());

        let home_children = &home_entry.unwrap().children;

        // Verify children are sorted alphabetically
        for i in 1..home_children.len() {
            assert!(home_children[i - 1].name <= home_children[i].name);
        }
    }

    #[test]
    fn test_build_sitemap_nested_structure() {
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let sitemap = router.build_sitemap().unwrap();

        // Find home -> about path
        let home_entry = sitemap.iter().find(|e| e.name == "home");
        assert!(home_entry.is_some());

        let about_entry = home_entry
            .unwrap()
            .children
            .iter()
            .find(|e| e.name == "about");
        assert!(about_entry.is_some());

        // Check that about has children (e.g., me.md)
        let me_entry = about_entry
            .unwrap()
            .children
            .iter()
            .find(|e| e.name == "me");
        assert!(me_entry.is_some());
        assert_eq!(me_entry.unwrap().url_path, "/home/about/me");
    }

    #[test]
    fn test_generate_sitemap_footer_basic() {
        let entries = vec![
            SitemapEntry::new("Root".to_string(), "/".to_string()),
            SitemapEntry::new("home".to_string(), "/home".to_string()),
        ];

        let html = generate_sitemap_footer(&entries, None);

        // Check for double HR
        assert!(html.contains("<hr><hr>"));

        // Check for UL structure
        assert!(html.contains("<ul>"));
        assert!(html.contains("</ul>"));

        // Check for links
        assert!(html.contains("<a href=\"/\">Root</a>"));
        assert!(html.contains("<a href=\"/home\">home</a>"));
    }

    #[test]
    fn test_generate_sitemap_footer_nested() {
        let mut home_entry = SitemapEntry::new("home".to_string(), "/home".to_string());
        home_entry.children.push(SitemapEntry::new(
            "about".to_string(),
            "/home/about".to_string(),
        ));

        let entries = vec![home_entry];

        let html = generate_sitemap_footer(&entries, None);

        // Check for nested UL
        assert!(html.matches("<ul>").count() >= 2);
        assert!(html.matches("</ul>").count() >= 2);

        // Check for nested links
        assert!(html.contains("<a href=\"/home\">home</a>"));
        assert!(html.contains("<a href=\"/home/about\">about</a>"));
    }

    #[test]
    fn test_generate_sitemap_footer_empty() {
        let entries: Vec<SitemapEntry> = vec![];
        let html = generate_sitemap_footer(&entries, None);

        // Should only have double HR when empty
        assert_eq!(html, "<hr><hr>");
    }

    #[test]
    fn test_generate_sitemap_footer_with_current_path() {
        let entries = vec![
            SitemapEntry::new("root".to_string(), "/".to_string()),
            SitemapEntry::new("home".to_string(), "/home".to_string()),
        ];

        let html = generate_sitemap_footer(&entries, Some("/home"));

        // Check that /home has the indicator
        assert!(html.contains("<b>home ← you're here</b>"));

        // Check that root doesn't have the indicator
        assert!(html.contains("<a href=\"/\">root</a>"));
        assert!(!html.contains("<b>root"));
    }

    #[test]
    fn test_generate_sitemap_footer_with_nested_current_path() {
        let mut home_entry = SitemapEntry::new("home".to_string(), "/home".to_string());
        home_entry.children.push(SitemapEntry::new(
            "about".to_string(),
            "/home/about".to_string(),
        ));

        let entries = vec![home_entry];

        let html = generate_sitemap_footer(&entries, Some("/home/about"));

        // Check that /home/about has the indicator
        assert!(html.contains("<b>about ← you're here</b>"));

        // Check that /home doesn't have the indicator
        assert!(html.contains("<a href=\"/home\">home</a>"));
        assert!(!html.contains("<b>home ← you're here</b>"));
    }

    #[test]
    fn test_escape_html_attr() {
        let escaped = escape_html_attr("<script>alert('XSS')</script>");
        assert_eq!(escaped, "&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;");
    }

    #[test]
    fn test_escape_html_text() {
        let escaped = escape_html_text("<b>Bold & Beautiful</b>");
        assert_eq!(escaped, "&lt;b&gt;Bold &amp; Beautiful&lt;/b&gt;");
    }

    #[test]
    fn test_resolve_path_prefers_directory_index() {
        // When both home.md and home/home.md exist, /home should resolve to home/home.md
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_path("/home").unwrap();
        assert!(result.is_found());

        // Should resolve to home/home.md (directory index), not home.md
        let resolved_path = result.path().unwrap();
        assert!(resolved_path.ends_with("home/home.md"));
        assert!(!resolved_path.ends_with("home.md") || resolved_path.ends_with("home/home.md"));
    }

    #[test]
    fn test_sitemap_excludes_shadowed_files() {
        // When both home.md and home/home.md exist, sitemap should only include /home once
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let sitemap = router.build_sitemap().unwrap();

        // Count how many entries have url_path = "/home"
        let home_count = sitemap.iter().filter(|e| e.url_path == "/home").count();

        // Should only have one /home entry
        assert_eq!(home_count, 1, "Expected exactly one /home entry in sitemap");

        // The /home entry should have children (since it's a directory index)
        let home_entry = sitemap.iter().find(|e| e.url_path == "/home");
        assert!(home_entry.is_some());
        assert!(
            !home_entry.unwrap().children.is_empty(),
            "/home should have children"
        );
    }

    #[test]
    fn test_sitemap_excludes_nested_shadowed_files() {
        // home/about.md is shadowed by home/about/about.md
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let sitemap = router.build_sitemap().unwrap();

        // Find home entry
        let home_entry = sitemap.iter().find(|e| e.url_path == "/home");
        assert!(home_entry.is_some());

        // Count how many children have url_path = "/home/about"
        let about_count = home_entry
            .unwrap()
            .children
            .iter()
            .filter(|e| e.url_path == "/home/about")
            .count();

        // Should only have one /home/about entry
        assert_eq!(about_count, 1, "Expected exactly one /home/about entry");

        // The /home/about entry should have children
        let about_entry = home_entry
            .unwrap()
            .children
            .iter()
            .find(|e| e.url_path == "/home/about");
        assert!(about_entry.is_some());
        assert!(
            !about_entry.unwrap().children.is_empty(),
            "/home/about should have children"
        );
    }

    #[test]
    fn test_reject_explicit_index_file_access() {
        // Accessing /home/home should be rejected (404) because it's trying to
        // explicitly access the index file that /home already serves
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_path("/home/home").unwrap();
        assert!(matches!(result, ResolvedPath::NotFound { .. }));

        let result = router.resolve_path("/home/about/about").unwrap();
        assert!(matches!(result, ResolvedPath::NotFound { .. }));
    }

    #[test]
    fn test_different_named_files_still_work() {
        // /home/about should work fine because "about" != "home"
        let temp_dir = create_test_content_root();
        let router = Router::new(temp_dir.path().to_path_buf()).unwrap();

        let result = router.resolve_path("/home/about").unwrap();
        assert!(result.is_found());

        let result = router.resolve_path("/home/about/me").unwrap();
        assert!(result.is_found());
    }
}
