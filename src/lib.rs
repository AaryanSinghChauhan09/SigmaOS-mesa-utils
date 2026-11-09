// SPDX-License-Identifier: MIT
// SigmaOS-mesa-utils - Mesa 3D Graphics Utilities component
// mesa-utils-style GPU info tools (glxinfo, glxgears equivalents),
// inspired by Arch's mesa-utils package.

use std::fmt;

// ============================================================================
// OPENGL INFO
// ============================================================================

/// OpenGL version info
#[derive(Debug, Clone)]
pub struct GlVersion {
    pub major: u32,
    pub minor: u32,
    pub profile: GlProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlProfile {
    Compatibility,
    Core,
    Es,
    Unknown,
}

impl GlVersion {
    pub fn new(major: u32, minor: u32, profile: GlProfile) -> Self {
        Self { major, minor, profile }
    }

    pub fn as_string(&self) -> String {
        format!("{}.{}", self.major, self.minor)
    }

    /// Whether this version supports a given feature
    pub fn supports(&self, major: u32, minor: u32) -> bool {
        self.major > major || (self.major == major && self.minor >= minor)
    }
}

/// GPU/driver info for OpenGL
#[derive(Debug, Clone)]
pub struct GlInfo {
    /// Renderer string (e.g. "AMD Radeon RX 7900 XT")
    pub renderer: String,
    /// Vendor string
    pub vendor: String,
    /// OpenGL version
    pub version: GlVersion,
    /// GLSL version
    pub glsl_version: String,
    /// Whether direct rendering is enabled
    pub direct_rendering: bool,
    /// Available extensions
    pub extensions: Vec<String>,
    /// Memory info
    pub memory: GlMemory,
}

impl GlInfo {
    /// Whether a specific extension is supported
    pub fn has_extension(&self, ext: &str) -> bool {
        self.extensions.iter().any(|e| e == ext)
    }

    /// Number of supported extensions
    pub fn extension_count(&self) -> usize {
        self.extensions.len()
    }

    /// Check if OpenGL version is at least X.Y
    pub fn gl_version_at_least(&self, major: u32, minor: u32) -> bool {
        self.version.supports(major, minor)
    }
}

/// GPU memory information
#[derive(Debug, Clone, Default)]
pub struct GlMemory {
    pub total_mb: u64,
    pub free_mb: u64,
    pub used_mb: u64,
}

impl GlMemory {
    pub fn usage_percent(&self) -> f64 {
        if self.total_mb == 0 { return 0.0; }
        self.used_mb as f64 / self.total_mb as f64 * 100.0
    }
}

// ============================================================================
// VULKAN INFO
// ============================================================================

/// Vulkan device info
#[derive(Debug, Clone)]
pub struct VulkanInfo {
    /// Device name
    pub device_name: String,
    /// Vendor name
    pub vendor_name: String,
    /// API version
    pub api_version: String,
    /// Driver version
    pub driver_version: String,
    /// Whether device is discrete
    pub discrete: bool,
    /// Available extensions
    pub extensions: Vec<String>,
    /// Queue families
    pub queue_families: u32,
    /// Max image dimension 2D
    pub max_image_dim_2d: u32,
    /// Max push constants size
    pub max_push_constants: u32,
}

impl VulkanInfo {
    pub fn has_extension(&self, ext: &str) -> bool {
        self.extensions.iter().any(|e| e == ext)
    }
}

// ============================================================================
// GPU QUERY
// ============================================================================

/// All GPU-related info
#[derive(Debug, Clone, Default)]
pub struct GpuQuery {
    pub opengl: Option<GlInfo>,
    pub vulkan: Option<VulkanInfo>,
    pub gpus: Vec<GpuDevice>,
}

/// A GPU device
#[derive(Debug, Clone)]
pub struct GpuDevice {
    pub name: String,
    pub vendor: String,
    pub pci_id: String,
    pub driver: String,
    pub vram_mb: u64,
    pub is_integrated: bool,
    pub is_discrete: bool,
}

impl GpuDevice {
    pub fn type_str(&self) -> &'static str {
        if self.is_discrete { "discrete" } else if self.is_integrated { "integrated" } else { "unknown" }
    }
}

// ============================================================================
// MESA INFO REPORT
// ============================================================================

/// Generate glxinfo-style report
pub fn glxinfo_report(info: &GlInfo) -> String {
    let mut out = String::new();
    out.push_str(&format!("name of display: :0\n"));
    out.push_str(&format!("display: :0  screen: 0\n"));
    out.push_str(&format!("direct rendering: {}\n", info.direct_rendering));
    out.push_str(&format!("server glx vendor string: {}\n", info.vendor));
    out.push_str(&format!("client glx vendor string: {}\n", info.vendor));
    out.push_str(&format!("OpenGL vendor string: {}\n", info.vendor));
    out.push_str(&format!("OpenGL renderer string: {}\n", info.renderer));
    out.push_str(&format!("OpenGL core profile version string: {} Core Profile Context {}\n",
        info.version.as_string(), info.version.profile.as_str()));
    out.push_str(&format!("OpenGL shading language version string: {}\n", info.glsl_version));
    out.push_str(&format!("OpenGL memory info: {} MB total, {} MB free\n",
        info.memory.total_mb, info.memory.free_mb));
    out.push_str(&format!("{} extensions supported\n", info.extension_count()));
    if info.extension_count() > 0 {
        out.push_str("GLX extensions:\n");
        for ext in info.extensions.iter().take(20) {
            out.push_str(&format!("  {}\n", ext));
        }
        if info.extension_count() > 20 {
            out.push_str(&format!("  ... and {} more\n", info.extension_count() - 20));
        }
    }
    out
}

impl GlProfile {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Compatibility => "Compatibility",
            Self::Core => "Core",
            Self::Es => "ES",
            Self::Unknown => "Unknown",
        }
    }
}

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Debug)]
pub enum MesaError {
    NoDisplay,
    NoGpu,
    ContextFailed(String),
    DriverMissing(String),
}

impl fmt::Display for MesaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDisplay => write!(f, "no display available"),
            Self::NoGpu => write!(f, "no GPU found"),
            Self::ContextFailed(m) => write!(f, "context creation failed: {}", m),
            Self::DriverMissing(d) => write!(f, "driver missing: {}", d),
        }
    }
}
impl std::error::Error for MesaError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gl_version() {
        let v = GlVersion::new(4, 6, GlProfile::Core);
        assert_eq!(v.as_string(), "4.6");
        assert!(v.supports(4, 5));
        assert!(v.supports(4, 6));
        assert!(!v.supports(4, 7));
        assert!(!v.supports(5, 0));
    }

    #[test]
    fn test_gl_info() {
        let info = GlInfo {
            renderer: "AMD Radeon RX 7900 XT".into(),
            vendor: "AMD".into(),
            version: GlVersion::new(4, 6, GlProfile::Core),
            glsl_version: "4.60".into(),
            direct_rendering: true,
            extensions: vec!["GL_ARB_compute_shader".into(), "GL_ARB_ray_tracing".into()],
            memory: GlMemory { total_mb: 20480, free_mb: 18000, used_mb: 2480 },
        };
        assert!(info.has_extension("GL_ARB_compute_shader"));
        assert!(!info.has_extension("GL_ARB_tessellation_shader"));
        assert_eq!(info.extension_count(), 2);
        assert!(info.gl_version_at_least(4, 6));
        assert!(!info.gl_version_at_least(4, 7));
        assert!((info.memory.usage_percent() - 12.1).abs() < 0.5);
    }

    #[test]
    fn test_glxinfo_report() {
        let info = GlInfo {
            renderer: "Intel UHD 770".into(),
            vendor: "Intel".into(),
            version: GlVersion::new(4, 6, GlProfile::Core),
            glsl_version: "4.60".into(),
            direct_rendering: true,
            extensions: vec!["GL_EXT_direct_state_access".into()],
            memory: GlMemory { total_mb: 512, free_mb: 400, used_mb: 112 },
        };
        let report = glxinfo_report(&info);
        assert!(report.contains("Intel UHD 770"));
        assert!(report.contains("4.6"));
        assert!(report.contains("direct rendering: true"));
        assert!(report.contains("512 MB total"));
    }

    #[test]
    fn test_vulkan_info() {
        let vulkan = VulkanInfo {
            device_name: "NVIDIA GeForce RTX 4090".into(),
            vendor_name: "NVIDIA".into(),
            api_version: "1.3.280".into(),
            driver_version: "535.104".into(),
            discrete: true,
            extensions: vec!["VK_KHR_ray_tracing_pipeline".into()],
            queue_families: 3,
            max_image_dim_2d: 16384,
            max_push_constants: 256,
        };
        assert!(vulkan.has_extension("VK_KHR_ray_tracing_pipeline"));
        assert!(vulkan.discrete);
    }

    #[test]
    fn test_gpu_device() {
        let gpu = GpuDevice {
            name: "AMD Radeon RX 7900 XTX".into(),
            vendor: "AMD".into(),
            pci_id: "1002:744c".into(),
            driver: "amdgpu".into(),
            vram_mb: 24576,
            is_integrated: false,
            is_discrete: true,
        };
        assert_eq!(gpu.type_str(), "discrete");
    }

    #[test]
    fn test_memory_usage() {
        let mem = GlMemory { total_mb: 0, free_mb: 0, used_mb: 0 };
        assert_eq!(mem.usage_percent(), 0.0);
    }
}
