//! Pack configuration types
//!
//! This module provides runtime configuration types for the packer.
//! Common types are re-exported from the `common` module for consistency.

use crate::common::{
    default_module_search_paths, default_optimize, default_python_version, CollectPattern,
    HooksConfig, VxHooksConfig,
};
use crate::protection::ProtectionConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

// Re-export common types
pub use crate::common::{
    BundleStrategy, DebugConfig, IsolationConfig, LicenseConfig, TargetPlatform, WindowConfig,
    WindowsPlatformConfig,
};

// ============================================================================
// Pack Mode
// ============================================================================

/// Pack mode determines how the application loads content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PackMode {
    /// Load content from a URL
    Url {
        /// The URL to load (will be normalized to include https:// if missing)
        url: String,
    },
    /// Load content from embedded frontend assets
    Frontend {
        /// Path to the frontend directory or HTML file
        #[serde(skip)]
        path: PathBuf,
    },
    /// FullStack mode: Frontend + Python backend
    FullStack {
        /// Path to the frontend directory
        #[serde(skip)]
        frontend_path: PathBuf,
        /// Python configuration (boxed to reduce enum size)
        python: Box<PythonBundleConfig>,
    },
}

impl PackMode {
    /// Get the mode name
    pub fn name(&self) -> &'static str {
        match self {
            PackMode::Url { .. } => "url",
            PackMode::Frontend { .. } => "frontend",
            PackMode::FullStack { .. } => "fullstack",
        }
    }

    /// Check if this mode embeds assets
    pub fn embeds_assets(&self) -> bool {
        matches!(self, PackMode::Frontend { .. } | PackMode::FullStack { .. })
    }

    /// Check if this mode includes Python backend
    pub fn has_python(&self) -> bool {
        matches!(self, PackMode::FullStack { .. })
    }

    /// Get the frontend path if applicable
    pub fn frontend_path(&self) -> Option<&PathBuf> {
        match self {
            PackMode::Frontend { path } => Some(path),
            PackMode::FullStack { frontend_path, .. } => Some(frontend_path),
            PackMode::Url { .. } => None,
        }
    }

    /// Get the URL if applicable
    pub fn url(&self) -> Option<&str> {
        match self {
            PackMode::Url { url } => Some(url),
            _ => None,
        }
    }

    /// Get the Python config if applicable
    pub fn python_config(&self) -> Option<&PythonBundleConfig> {
        match self {
            PackMode::FullStack { python, .. } => Some(python),
            _ => None,
        }
    }
}

// ============================================================================
// Python Bundle Configuration
// ============================================================================

/// Python bundle configuration for FullStack mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonBundleConfig {
    /// Entry point (e.g., "myapp.main:run" or "main.py")
    pub entry_point: String,

    /// Python source paths to include
    #[serde(default)]
    pub include_paths: Vec<PathBuf>,

    /// Pip packages to install
    #[serde(default)]
    pub packages: Vec<String>,

    /// Path to requirements.txt
    #[serde(default)]
    pub requirements: Option<PathBuf>,

    /// Bundle strategy
    #[serde(default)]
    pub strategy: BundleStrategy,

    /// Python version (e.g., "3.11")
    #[serde(default = "default_python_version")]
    pub version: String,

    /// Bytecode optimization level (0, 1, or 2)
    #[serde(default = "default_optimize")]
    pub optimize: u8,

    /// Exclude patterns
    #[serde(default)]
    pub exclude: Vec<String>,

    /// External binaries to bundle (paths to executables)
    #[serde(default)]
    pub external_bin: Vec<PathBuf>,

    /// Additional resource files/directories
    #[serde(default)]
    pub resources: Vec<PathBuf>,

    /// Include pip in the bundle (for PyOxidizer)
    #[serde(default)]
    pub include_pip: bool,

    /// Include setuptools in the bundle (for PyOxidizer)
    #[serde(default)]
    pub include_setuptools: bool,

    /// PyOxidizer distribution flavor
    #[serde(default)]
    pub distribution_flavor: Option<String>,

    /// Custom PyOxidizer executable path
    #[serde(default)]
    pub pyoxidizer_path: Option<PathBuf>,

    /// Module search paths (relative to extract directory).
    /// Special variables: $EXTRACT_DIR, $RESOURCES_DIR, $SITE_PACKAGES, $PYTHON_HOME
    #[serde(default = "default_module_search_paths")]
    pub module_search_paths: Vec<String>,

    /// Whether to use filesystem importer (allows dynamic imports)
    #[serde(default = "default_true")]
    pub filesystem_importer: bool,

    /// Show console window for Python process (Windows only)
    #[serde(default)]
    pub show_console: bool,

    /// Environment isolation configuration
    #[serde(default)]
    pub isolation: IsolationConfig,

    /// Code protection configuration (py2pyd compilation)
    #[serde(default)]
    pub protection: ProtectionConfig,
}

fn default_true() -> bool {
    true
}

impl Default for PythonBundleConfig {
    fn default() -> Self {
        Self {
            entry_point: String::new(),
            include_paths: Vec::new(),
            packages: Vec::new(),
            requirements: None,
            strategy: BundleStrategy::default(),
            version: default_python_version(),
            optimize: default_optimize(),
            exclude: Vec::new(),
            external_bin: Vec::new(),
            resources: Vec::new(),
            include_pip: false,
            include_setuptools: false,
            distribution_flavor: None,
            pyoxidizer_path: None,
            module_search_paths: default_module_search_paths(),
            filesystem_importer: true,
            show_console: false,
            isolation: IsolationConfig::default(),
            protection: ProtectionConfig::default(),
        }
    }
}

impl PythonBundleConfig {
    /// Create a new Python bundle config with entry point
    pub fn new(entry_point: impl Into<String>) -> Self {
        Self {
            entry_point: entry_point.into(),
            ..Default::default()
        }
    }

    /// Set Python version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Add include paths
    pub fn with_include_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.include_paths = paths;
        self
    }

    /// Set bundle strategy
    pub fn with_strategy(mut self, strategy: BundleStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set isolation config
    pub fn with_isolation(mut self, isolation: IsolationConfig) -> Self {
        self.isolation = isolation;
        self
    }
}

// ============================================================================
// Complete Pack Configuration
// ============================================================================

/// Complete pack configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackConfig {
    /// Pack mode (URL, Frontend, or FullStack)
    pub mode: PackMode,

    /// Output executable name (without extension)
    pub output_name: String,

    /// Output directory
    #[serde(skip)]
    pub output_dir: PathBuf,

    /// Window configuration
    pub window: WindowConfig,

    /// Target platform
    #[serde(default)]
    pub target_platform: TargetPlatform,

    /// Enable debug mode
    #[serde(default)]
    pub debug: bool,

    /// Allow opening new windows
    #[serde(default)]
    pub allow_new_window: bool,

    /// Custom user agent
    #[serde(default)]
    pub user_agent: Option<String>,

    /// JavaScript to inject
    #[serde(default)]
    pub inject_js: Option<String>,

    /// CSS to inject
    #[serde(default)]
    pub inject_css: Option<String>,

    /// Icon path (for resource injection)
    #[serde(skip)]
    pub icon_path: Option<PathBuf>,

    /// Window icon PNG data (embedded at pack time)
    #[serde(default)]
    #[serde(with = "serde_bytes_base64")]
    pub window_icon: Option<Vec<u8>>,

    /// Environment variables to inject at runtime
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// License configuration for authorization
    #[serde(default)]
    pub license: Option<LicenseConfig>,

    /// Hooks configuration for collecting additional files
    #[serde(default)]
    pub hooks: Option<HooksConfig>,

    /// Remote debugging port for CDP connections
    #[serde(default)]
    pub remote_debugging_port: Option<u16>,

    /// Windows-specific resource configuration
    #[serde(skip)]
    pub windows_resource: WindowsPlatformConfig,

    /// Vx configuration for dependency bootstrap
    #[serde(default)]
    pub vx: Option<crate::manifest::VxConfig>,

    /// Downloads configuration for embedding external dependencies
    #[serde(default)]
    pub downloads: Vec<crate::manifest::DownloadEntry>,

    /// Compression level for assets (1-22, default 19 for best ratio)
    /// Higher levels = better compression but slower packing
    /// Recommended: 19 for release, 3 for development
    #[serde(default = "default_compression_level")]
    pub compression_level: i32,

    /// Chrome extensions configuration
    #[serde(default)]
    pub extensions: Option<crate::manifest::ExtensionsConfig>,
}

/// Default compression level (19 = high compression, good for releases)
fn default_compression_level() -> i32 {
    19
}

/// Serde helper module for serializing Option<Vec<u8>> as base64
mod serde_bytes_base64 {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match data {
            Some(bytes) => serializer.serialize_some(&STANDARD.encode(bytes)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(s) => STANDARD
                .decode(&s)
                .map(Some)
                .map_err(serde::de::Error::custom),
            None => Ok(None),
        }
    }
}

impl PackConfig {
    /// Create a URL mode configuration
    pub fn url(url: impl Into<String>) -> Self {
        let url = url.into();
        let output_name = url
            .replace("https://", "")
            .replace("http://", "")
            .replace("www.", "")
            .split('.')
            .next()
            .unwrap_or("app")
            .to_string();

        Self {
            mode: PackMode::Url { url },
            output_name,
            output_dir: PathBuf::from("."),
            window: WindowConfig::default(),
            target_platform: TargetPlatform::Current,
            debug: false,
            allow_new_window: false,
            user_agent: None,
            inject_js: None,
            inject_css: None,
            icon_path: None,
            window_icon: None,
            env: HashMap::new(),
            license: None,
            hooks: None,
            remote_debugging_port: None,
            windows_resource: WindowsPlatformConfig::default(),
            vx: None,
            downloads: vec![],
            compression_level: default_compression_level(),
            extensions: None,
        }
    }

    /// Create a frontend mode configuration
    pub fn frontend(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let output_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app")
            .to_string();

        Self {
            mode: PackMode::Frontend { path },
            output_name,
            output_dir: PathBuf::from("."),
            window: WindowConfig::default(),
            target_platform: TargetPlatform::Current,
            debug: false,
            allow_new_window: false,
            user_agent: None,
            inject_js: None,
            inject_css: None,
            icon_path: None,
            window_icon: None,
            env: HashMap::new(),
            license: None,
            hooks: None,
            remote_debugging_port: None,
            windows_resource: WindowsPlatformConfig::default(),
            vx: None,
            downloads: vec![],
            compression_level: default_compression_level(),
            extensions: None,
        }
    }

    /// Create a fullstack mode configuration (frontend + Python backend)
    pub fn fullstack(frontend_path: impl Into<PathBuf>, entry_point: impl Into<String>) -> Self {
        let frontend_path = frontend_path.into();
        let output_name = frontend_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app")
            .to_string();

        Self {
            mode: PackMode::FullStack {
                frontend_path,
                python: Box::new(PythonBundleConfig::new(entry_point)),
            },
            output_name,
            output_dir: PathBuf::from("."),
            window: WindowConfig::default(),
            target_platform: TargetPlatform::Current,
            debug: false,
            allow_new_window: false,
            user_agent: None,
            inject_js: None,
            inject_css: None,
            icon_path: None,
            window_icon: None,
            env: HashMap::new(),
            license: None,
            hooks: None,
            remote_debugging_port: None,
            windows_resource: WindowsPlatformConfig::default(),
            vx: None,
            downloads: vec![],
            compression_level: default_compression_level(),
            extensions: None,
        }
    }

    /// Create a fullstack mode configuration with full Python config
    pub fn fullstack_with_config(
        frontend_path: impl Into<PathBuf>,
        python: PythonBundleConfig,
    ) -> Self {
        let frontend_path = frontend_path.into();
        let output_name = frontend_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("app")
            .to_string();

        Self {
            mode: PackMode::FullStack {
                frontend_path,
                python: Box::new(python),
            },
            output_name,
            output_dir: PathBuf::from("."),
            window: WindowConfig::default(),
            target_platform: TargetPlatform::Current,
            debug: false,
            allow_new_window: false,
            user_agent: None,
            inject_js: None,
            inject_css: None,
            icon_path: None,
            window_icon: None,
            env: HashMap::new(),
            license: None,
            hooks: None,
            remote_debugging_port: None,
            windows_resource: WindowsPlatformConfig::default(),
            vx: None,
            downloads: vec![],
            compression_level: default_compression_level(),
            extensions: None,
        }
    }

    /// Set the output name
    pub fn with_output(mut self, name: impl Into<String>) -> Self {
        self.output_name = name.into();
        self
    }

    /// Set the output directory
    pub fn with_output_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Set the window title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.window.title = title.into();
        self
    }

    /// Set the window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.window.width = width;
        self.window.height = height;
        self
    }

    /// Set debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Set frameless mode
    pub fn with_frameless(mut self, frameless: bool) -> Self {
        self.window.frameless = frameless;
        self
    }

    /// Set always on top
    pub fn with_always_on_top(mut self, always_on_top: bool) -> Self {
        self.window.always_on_top = always_on_top;
        self
    }

    /// Set resizable
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.window.resizable = resizable;
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set icon path
    pub fn with_icon(mut self, path: impl Into<PathBuf>) -> Self {
        self.icon_path = Some(path.into());
        self
    }

    /// Set environment variables
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Add a single environment variable
    pub fn with_env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set license configuration
    pub fn with_license(mut self, license: LicenseConfig) -> Self {
        self.license = Some(license);
        self
    }

    /// Set remote debugging port for CDP connections
    pub fn with_remote_debugging_port(mut self, port: u16) -> Self {
        self.remote_debugging_port = Some(port);
        self
    }

    /// Set expiration date (enables license)
    pub fn with_expiration(mut self, expires_at: impl Into<String>) -> Self {
        self.license = Some(LicenseConfig::time_limited(expires_at));
        self
    }

    /// Require token for authorization
    pub fn with_token_required(mut self) -> Self {
        let mut license = self.license.unwrap_or_default();
        license.enabled = true;
        license.require_token = true;
        self.license = Some(license);
        self
    }

    /// Set hooks configuration for collecting additional files
    pub fn with_hooks(mut self, hooks: HooksConfig) -> Self {
        self.hooks = Some(hooks);
        self
    }

    /// Get debug configuration
    pub fn debug_config(&self) -> DebugConfig {
        DebugConfig {
            enabled: self.debug,
            devtools: self.debug,
            verbose: false,
            remote_debugging_port: self.remote_debugging_port,
        }
    }

    /// Create PackConfig from a Manifest
    pub fn from_manifest(
        manifest: &crate::Manifest,
        base_dir: &Path,
    ) -> Result<Self, crate::PackError> {
        use crate::manifest::BackendType;

        // Determine mode based on frontend and backend config
        let mode = if let Some(ref fe) = manifest.frontend {
            if let Some(ref url) = fe.url {
                PackMode::Url { url: url.clone() }
            } else if let Some(ref path) = fe.path {
                let full_path = if path.is_absolute() {
                    path.clone()
                } else {
                    base_dir.join(path)
                };

                // Check if backend is configured for fullstack mode
                if let Some(ref backend) = manifest.backend {
                    if backend.backend_type == BackendType::Python {
                        // Use the manifest's get_python_bundle_config which properly resolves all fields
                        if let Some(python_config) = manifest.get_python_bundle_config(base_dir) {
                            return Self::fullstack_with_config(full_path, python_config)
                                .apply_manifest(manifest, base_dir);
                        }
                    }
                }

                PackMode::Frontend { path: full_path }
            } else {
                return Err(crate::PackError::Config(
                    "Frontend must have either 'path' or 'url' specified".to_string(),
                ));
            }
        } else {
            return Err(crate::PackError::Config(
                "No frontend configuration specified in manifest".to_string(),
            ));
        };

        // Build base config from mode
        let mut config = match &mode {
            PackMode::Url { url } => Self::url(url),
            PackMode::Frontend { path } => Self::frontend(path),
            _ => unreachable!(),
        };

        config.mode = mode;
        config.apply_manifest(manifest, base_dir)
    }

    /// Apply manifest settings to this config
    fn apply_manifest(
        mut self,
        manifest: &crate::Manifest,
        base_dir: &Path,
    ) -> Result<Self, crate::PackError> {
        // Package settings
        self.output_name = manifest.package.name.clone();

        // Output directory
        if let Some(ref out_dir) = manifest.build.out_dir {
            self.output_dir = if out_dir.is_absolute() {
                out_dir.clone()
            } else {
                base_dir.join(out_dir)
            };
        }

        // Window settings (window is not Option, it has default)
        let win = &manifest.window;
        self.window = WindowConfig {
            title: manifest
                .package
                .title
                .clone()
                .unwrap_or_else(|| manifest.package.name.clone()),
            width: win.width,
            height: win.height,
            min_width: win.min_width,
            min_height: win.min_height,
            max_width: win.max_width,
            max_height: win.max_height,
            resizable: win.resizable,
            fullscreen: win.fullscreen,
            frameless: win.frameless,
            always_on_top: win.always_on_top,
            transparent: win.transparent,
            start_position: win.start_position.clone().into(),
            maximized: win.maximized,
            visible: win.visible,
        };

        // Debug settings
        self.debug = manifest.debug.enabled;

        // Allow new window
        self.allow_new_window = manifest.package.allow_new_window;

        // User agent
        self.user_agent = manifest.package.user_agent.clone();

        // Injection settings
        if let Some(ref inject) = manifest.inject {
            self.inject_js = inject.js_code.clone();
            self.inject_css = inject.css_code.clone();
        }

        // Icon path - use platform-specific icon if available
        if let Some(icon) = manifest.get_icon_path() {
            let icon_path = if icon.is_absolute() {
                icon.clone()
            } else {
                base_dir.join(icon)
            };
            self.icon_path = Some(icon_path);
        }

        // Environment variables
        if let Some(ref runtime) = manifest.runtime {
            self.env = runtime.env.clone();
        }

        // License settings
        if let Some(ref l) = manifest.license {
            self.license = Some(LicenseConfig {
                enabled: l.enabled,
                expires_at: l.expires_at.clone(),
                require_token: l.require_token,
                embedded_token: l.embedded_token.clone(),
                validation_url: l.validation_url.clone(),
                allowed_machines: l.allowed_machines.clone(),
                grace_period_days: l.grace_period_days,
                expiration_message: l.expiration_message.clone(),
            });
        }

        // Hooks settings
        if let Some(ref hooks) = manifest.hooks {
            self.hooks = Some(HooksConfig {
                before_collect: hooks.before_collect.clone(),
                collect: hooks
                    .collect
                    .iter()
                    .map(|c| CollectPattern {
                        source: c.source.clone(),
                        dest: c.dest.clone(),
                        preserve_structure: c.preserve_structure,
                        description: c.description.clone(),
                    })
                    .collect(),
                after_pack: hooks.after_pack.clone(),
                use_vx: hooks.use_vx,
                vx: VxHooksConfig {
                    before_collect: hooks.vx.before_collect.clone(),
                    after_pack: hooks.vx.after_pack.clone(),
                },
            });
        }

        // Vx settings
        self.vx = manifest.vx.clone();

        // Downloads settings
        self.downloads = manifest.downloads.clone();

        // Extensions settings
        self.extensions = manifest.extensions.clone();

        Ok(self)
    }
}
