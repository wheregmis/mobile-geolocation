//! Cross-platform geolocation for Dioxus mobile, desktop, and web apps
//!
//! This crate provides geolocation functionality for Android, iOS, macOS, and Web platforms
//! using clean, direct bindings without external build tools. Android uses JNI
//! with a single Java file compiled to DEX, while iOS and macOS use objc2 for direct
//! Objective-C bindings to the CoreLocation framework. Web uses the browser's Geolocation API
//! via wasm-bindgen. Permissions are automatically embedded via linker symbols and injected
//! into platform manifests by the Dioxus CLI.
//!
//! ## Features
//!
//! - `location-coarse`: Request coarse location permission (default)
//! - `location-fine`: Request fine/precise location permission
//! - `background-location`: Request background location access
//!
//! ## Usage
//!
//! ```rust,no_run
//! use dioxus_mobile_geolocation::last_known_location;
//!
//! // For Android, iOS, and macOS
//! if let Some((lat, lon)) = last_known_location() {
//!     println!("Location: {}, {}", lat, lon);
//! }
//! ```
//!
//! ### Web Platform
//!
//! On web, geolocation is asynchronous. Use the `get_current_position` function:
//!
//! ```rust,no_run
//! #[cfg(target_arch = "wasm32")]
//! use dioxus_mobile_geolocation::get_current_position;
//! #[cfg(target_arch = "wasm32")]
//! use wasm_bindgen::prelude::*;
//!
//! #[cfg(target_arch = "wasm32")]
//! {
//!     let success = Closure::wrap(Box::new(|pos: web_sys::Position| {
//!         let coords = pos.coords();
//!         println!("Lat: {}, Lon: {}", coords.latitude(), coords.longitude());
//!     }) as Box<dyn Fn(web_sys::Position)>);
//!     
//!     let error = Closure::wrap(Box::new(|err: web_sys::PositionError| {
//!         eprintln!("Error: {}", err.message());
//!     }) as Box<dyn Fn(web_sys::PositionError)>);
//!     
//!     get_current_position(
//!         success.as_ref().unchecked_ref(),
//!         Some(error.into_js_value().unchecked_into())
//!     );
//! }
//! ```
//!
//! ## Permissions
//!
//! This crate uses the linker-based permission system. When you enable
//! `location-coarse` or `location-fine` features, the appropriate permissions
//! are embedded as linker symbols. The Dioxus CLI will automatically:
//!
//! - Add `<uses-permission>` entries to AndroidManifest.xml
//! - Add Info.plist keys to iOS/macOS bundles
//!
//! No manual manifest editing required!

// Platform modules
#[cfg(target_os = "android")]
mod android;

// Darwin-based platforms (iOS and macOS) share the same CoreLocation implementation
#[cfg(any(target_os = "ios", target_os = "macos"))]
mod darwin;

// Web platform uses browser's Geolocation API
#[cfg(target_arch = "wasm32")]
mod web;

// Re-export web-specific async API for proper usage on web
#[cfg(target_arch = "wasm32")]
pub use web::{get_current_position, get_current_position_sync};

#[cfg(not(any(
    target_os = "android",
    target_os = "ios",
    target_os = "macos",
    target_arch = "wasm32"
)))]
mod unsupported;

use permissions::{static_permission, Permission};

// Declare Java sources for Android using the macro system
// This embeds absolute paths and generates linker symbols automatically
#[cfg(target_os = "android")]
dioxus_platform_bridge::java_plugin!(
    package = "dioxus.mobile.geolocation",
    plugin = "geolocation",
    files = ["src/android/PermissionsHelper.java"]
);

#[cfg(any(target_os = "ios", target_os = "macos"))]
dioxus_platform_bridge::darwin_plugin!(
    plugin = "geolocation",
    frameworks = ["CoreLocation", "Foundation"]
);

// Error types
/// Result type for geolocation operations
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can occur when fetching the location.
#[derive(Copy, Clone, Debug)]
pub enum Error {
    /// An error occurred with the Android Java environment.
    AndroidEnvironment,
    /// The user denied authorization.
    AuthorizationDenied,
    /// A network error occurred.
    Network,
    /// The function was not called from the main thread.
    NotMainThread,
    /// Location data is temporarily unavailable.
    TemporarilyUnavailable,
    /// This device does not support location data.
    PermanentlyUnavailable,
    /// An unknown error occurred.
    Unknown,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AndroidEnvironment => write!(f, "Android Java environment error"),
            Error::AuthorizationDenied => write!(f, "Location authorization denied"),
            Error::Network => write!(f, "Network error"),
            Error::NotMainThread => write!(f, "Function must be called from main thread"),
            Error::TemporarilyUnavailable => write!(f, "Location temporarily unavailable"),
            Error::PermanentlyUnavailable => write!(f, "Location not supported on this device"),
            Error::Unknown => write!(f, "Unknown error"),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(target_os = "android")]
impl From<jni::errors::Error> for Error {
    fn from(_: jni::errors::Error) -> Self {
        Error::AndroidEnvironment
    }
}

/// Represents a geographic coordinate
#[derive(Debug, Clone, Copy)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

// Embed location permissions as linker symbols when features are enabled
#[cfg(feature = "location-fine")]
pub const LOCATION_FINE: Permission = static_permission!(
    Location(Fine),
    description = "Precise location for geolocation features"
);

#[cfg(feature = "location-coarse")]
pub const LOCATION_COARSE: Permission = static_permission!(
    Location(Coarse),
    description = "Approximate location for geolocation features"
);

// Optional background location (Android + iOS)
#[cfg(feature = "background-location")]
pub const BACKGROUND_LOCATION: Permission = static_permission!(
    Custom {
        android = "android.permission.ACCESS_BACKGROUND_LOCATION",
        ios = "NSLocationAlwaysAndWhenInUseUsageDescription",
        macos = "NSLocationUsageDescription",
        windows = "location",
        linux = "",
        web = ""
    },
    description = "Background location access"
);

/// Internal function to ensure permission constants are linked into the binary.
/// This prevents the linker from optimizing them away as dead code.
/// DO NOT REMOVE - this is required for the permission system to work.
#[doc(hidden)]
#[inline(never)]
pub fn __ensure_permissions_linked() {
    #[cfg(feature = "location-fine")]
    {
        let _ = &LOCATION_FINE;
    }
    #[cfg(feature = "location-coarse")]
    {
        let _ = &LOCATION_COARSE;
    }
    #[cfg(feature = "background-location")]
    {
        let _ = &BACKGROUND_LOCATION;
    }
}

/// Ensure metadata is linked into the binary
#[inline(never)]
#[doc(hidden)]
fn __ensure_metadata_linked() {
    // Metadata is automatically linked via the macro-generated static
    // The #[link_section] and #[used] attributes ensure the data is included
    #[cfg(target_os = "ios")]
    let _ = &IOS_FRAMEWORK_METADATA;
    // macOS uses the same ios_plugin! macro, so metadata is handled there
}

/// Request location permissions at runtime.
///
/// This function triggers the system permission dialog for location access.
/// Returns `true` if the permission request was sent successfully, `false` otherwise.
///
/// ## Platform behavior
///
/// - **Android**: Calls `ActivityCompat.requestPermissions()` via JNI
/// - **iOS/macOS**: Calls `CLLocationManager.requestWhenInUseAuthorization()` via objc2
/// - **Web**: Initiates geolocation request (triggers permission dialog and populates cache)
/// - **Other platforms**: Always returns `false`
///
/// ## Usage
///
/// Call this function before `last_known_location()` to ensure permissions are granted.
/// The user will see a system dialog asking for location permission.
///
/// On web, this also starts fetching the location asynchronously. You can then
/// call `last_known_location()` after a short delay to retrieve the cached result.
pub fn request_location_permission() -> bool {
    // Ensure permissions and metadata are linked (prevents dead code elimination)
    __ensure_permissions_linked();
    __ensure_metadata_linked();

    #[cfg(target_os = "android")]
    return android::request_permission();
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    return darwin::request_permission();
    #[cfg(target_arch = "wasm32")]
    return web::request_permission();
    #[cfg(not(any(
        target_os = "android",
        target_os = "ios",
        target_os = "macos",
        target_arch = "wasm32"
    )))]
    return unsupported::request_permission();
}

/// Get the last known location from the device.
///
/// Returns `Some((latitude, longitude))` if a location is available,
/// or `None` if no location has been cached or permissions are denied.
///
/// ## Platform behavior
///
/// - **Android**: Queries `LocationManager.getLastKnownLocation()` via JNI
/// - **iOS/macOS**: Queries `CLLocationManager.location` via objc2
/// - **Web**: Returns cached position (must call `get_current_position_sync()` first to populate cache)
/// - **Other platforms**: Always returns `None`
///
/// ## Web Usage
///
/// On web, you need to initiate a location request first:
///
/// ```rust,no_run
/// #[cfg(target_arch = "wasm32")]
/// {
///     use dioxus_mobile_geolocation::{get_current_position_sync, last_known_location};
///     
///     // Initiate location request (async, populates cache when ready)
///     get_current_position_sync();
///     
///     // Later, check if location is available
///     if let Some((lat, lon)) = last_known_location() {
///         println!("Location: {}, {}", lat, lon);
///     }
/// }
/// ```
///
/// ## Permissions
///
/// This function requires location permissions to be granted at runtime.
/// The compile-time permissions are automatically embedded when you enable
/// the `location-coarse` or `location-fine` features.
///
/// On Android, you should request permissions using `request_location_permission()`
/// before calling this function.
///
/// On iOS/macOS, permissions are handled via Info.plist configuration.
///
/// On Web, permissions are requested automatically when you call the Geolocation API.
pub fn last_known_location() -> Option<(f64, f64)> {
    // Ensure permissions and metadata are linked (prevents dead code elimination)
    __ensure_permissions_linked();
    __ensure_metadata_linked();

    #[cfg(target_os = "android")]
    return android::last_known();
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    return darwin::last_known();
    #[cfg(target_arch = "wasm32")]
    return web::last_known();
    #[cfg(not(any(
        target_os = "android",
        target_os = "ios",
        target_os = "macos",
        target_arch = "wasm32"
    )))]
    return unsupported::last_known();
}
