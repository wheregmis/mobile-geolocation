//! Web platform geolocation implementation
//!
//! Uses the browser's Geolocation API to access location data.
//! This implementation is synchronous for `last_known_location()` which returns
//! cached position if available, but the browser API is inherently asynchronous.
//!
//! For a proper implementation, consider using the async Geolocation API with callbacks.

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Position, PositionError, PositionOptions};

/// Request location permission
///
/// On web, this doesn't explicitly request permission - instead, permission
/// is requested when you call `getCurrentPosition()` or `watchPosition()`.
/// Returns true to indicate the API is available.
pub fn request_permission() -> bool {
    // Check if geolocation is available
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };

    let navigator = window.navigator();
    navigator.geolocation().is_ok()
}

/// Get the last known location
///
/// Note: The browser's Geolocation API doesn't provide a "last known" location.
/// This function attempts to get the current position synchronously by checking
/// if a cached position exists, but this is not reliable.
///
/// For web, you should use the async Geolocation API with `getCurrentPosition()`.
/// This implementation returns `None` as web geolocation is inherently async.
pub fn last_known() -> Option<(f64, f64)> {
    // Web Geolocation API is asynchronous and doesn't provide a "last known" sync method
    // To properly implement this, we would need to:
    // 1. Call getCurrentPosition with a callback
    // 2. Store the result somewhere accessible
    // 3. Return the cached value
    //
    // For now, return None to indicate this should be implemented with async APIs
    None
}

/// Get current position asynchronously (proper web implementation)
///
/// This is the recommended way to get location on web platforms.
/// Takes success and error callbacks.
#[wasm_bindgen]
pub fn get_current_position(
    success_callback: &js_sys::Function,
    error_callback: Option<js_sys::Function>,
) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window object")?;
    let navigator = window.navigator();
    let geolocation = navigator.geolocation().map_err(|_| "Geolocation not available")?;

    let mut options = PositionOptions::new();
    options.enable_high_accuracy(true);
    options.timeout(10000); // 10 second timeout
    options.maximum_age(0); // Don't use cached position

    geolocation
        .get_current_position_with_error_callback_and_options(
            success_callback,
            error_callback.as_ref(),
            &options,
        )
        .map_err(|_| "Failed to request position")?;

    Ok(())
}
