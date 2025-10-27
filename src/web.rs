//! Web platform geolocation implementation
//!
//! Uses the browser's Geolocation API to access location data.
//! Since the browser API is asynchronous, this module provides both sync and async interfaces.
//! The sync `last_known()` function returns cached position if available.

use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Position, PositionError, PositionOptions};

thread_local! {
    static CACHED_POSITION: RefCell<Option<(f64, f64)>> = RefCell::new(None);
}

/// Request location permission
///
/// On web, this checks if the Geolocation API is available and initiates
/// a position request to trigger the permission dialog.
/// Returns true to indicate the API is available and request was initiated.
pub fn request_permission() -> bool {
    // Check if geolocation is available
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };

    let navigator = window.navigator();
    if navigator.geolocation().is_err() {
        return false;
    }

    // Also initiate a position request to populate the cache
    get_current_position_sync()
}

/// Get the last known (cached) location
///
/// Returns the cached location if one was previously obtained via `get_current_position_sync()`.
/// Returns `None` if no location has been cached yet.
///
/// For web, you should call `get_current_position_sync()` first to populate the cache.
pub fn last_known() -> Option<(f64, f64)> {
    CACHED_POSITION.with(|pos| *pos.borrow())
}

/// Update the cached position (internal use)
fn update_cached_position(lat: f64, lon: f64) {
    CACHED_POSITION.with(|pos| {
        *pos.borrow_mut() = Some((lat, lon));
    });
}

/// Get current position synchronously by triggering the async API
///
/// This function initiates the geolocation request and returns immediately.
/// When the position is obtained, it's cached and can be retrieved via `last_known()`.
///
/// Returns `true` if the request was initiated successfully, `false` otherwise.
pub fn get_current_position_sync() -> bool {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };

    let navigator = window.navigator();
    let geolocation = match navigator.geolocation() {
        Ok(geo) => geo,
        Err(_) => return false,
    };

    // Create success callback
    let success = Closure::wrap(Box::new(move |pos: Position| {
        let coords = pos.coords();
        update_cached_position(coords.latitude(), coords.longitude());
    }) as Box<dyn FnMut(Position)>);

    // Create error callback
    let error = Closure::wrap(Box::new(move |_err: PositionError| {
        // Silently ignore errors for the sync API
    }) as Box<dyn FnMut(PositionError)>);

    let options = PositionOptions::new();
    options.set_enable_high_accuracy(false); // Use network location for faster response
    options.set_timeout(10000);
    options.set_maximum_age(60000); // Allow cached positions up to 1 minute old

    let result = geolocation.get_current_position_with_error_callback_and_options(
        success.as_ref().unchecked_ref(),
        Some(error.as_ref().unchecked_ref()),
        &options,
    );

    // Keep closures alive
    success.forget();
    error.forget();

    result.is_ok()
}

/// Get current position asynchronously (proper web implementation)
///
/// This is the recommended way to get location on web platforms for more control.
/// Takes success and error callbacks.
#[wasm_bindgen]
pub fn get_current_position(
    success_callback: &js_sys::Function,
    error_callback: Option<js_sys::Function>,
) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window object")?;
    let navigator = window.navigator();
    let geolocation = navigator
        .geolocation()
        .map_err(|_| "Geolocation not available")?;

    let options = PositionOptions::new();
    options.set_enable_high_accuracy(true);
    options.set_timeout(10000); // 10 second timeout
    options.set_maximum_age(0); // Don't use cached position

    geolocation
        .get_current_position_with_error_callback_and_options(
            success_callback,
            error_callback.as_ref(),
            &options,
        )
        .map_err(|_| "Failed to request position")?;

    Ok(())
}
