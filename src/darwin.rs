//! Darwin platform (iOS and macOS) geolocation implementation
//!
//! Both iOS and macOS use the same CoreLocation framework and share identical
//! APIs for location services. This module provides a unified implementation
//! for both platforms.

use dioxus_platform_bridge::darwin::MainThreadCell;
use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_core_location::{CLAuthorizationStatus, CLLocation, CLLocationManager};

/// Global location manager instance
static LOCATION_MANAGER: MainThreadCell<Retained<CLLocationManager>> = MainThreadCell::new();

/// Get or create the global location manager
fn get_location_manager(mtm: MainThreadMarker) -> &'static Retained<CLLocationManager> {
    LOCATION_MANAGER.get_or_init_with(mtm, || {
        // SAFETY: `CLLocationManager` is main-thread-only; the marker provided to
        // `get_or_init_with` ensures we're on the main thread.
        unsafe { CLLocationManager::new() }
    })
}

/// Request location authorization
pub fn request_permission() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };

    let manager = get_location_manager(mtm);

    // Check authorization status first
    let auth_status = unsafe { manager.authorizationStatus() };

    // Only request if not determined (NotDetermined)
    if auth_status == CLAuthorizationStatus::NotDetermined {
        unsafe {
            manager.requestWhenInUseAuthorization();
        }
    }

    true
}

/// Get the last known location
pub fn last_known() -> Option<(f64, f64)> {
    let mtm = MainThreadMarker::new()?;

    let manager = get_location_manager(mtm);

    // Check authorization status before attempting to get location
    let auth_status = unsafe { manager.authorizationStatus() };

    // Only proceed if authorized
    match auth_status {
        CLAuthorizationStatus::AuthorizedAlways | CLAuthorizationStatus::AuthorizedWhenInUse => {
            // Can proceed to get location
        }
        _ => {
            // Not authorized - try to get last known location anyway
            // This might work for locations cached before permission was revoked
        }
    }

    // First, try to get the cached location without starting updates
    let location: Option<Retained<CLLocation>> = unsafe { manager.location() };

    if location.is_some() {
        let loc = location.unwrap();
        let coordinate = unsafe { loc.coordinate() };
        return Some((coordinate.latitude, coordinate.longitude));
    }

    // If no cached location, start updates
    // Note: In a proper implementation, we would set up a delegate to receive
    // location updates asynchronously. For now, we'll use a simple approach
    // that starts updates and then checks after a delay.
    unsafe {
        manager.startUpdatingLocation();
    }

    // Wait for location to be obtained (allowing GPS to get a fix)
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Try again now that updates are running
    let location: Option<Retained<CLLocation>> = unsafe { manager.location() };

    // Stop updating to conserve battery
    unsafe {
        manager.stopUpdatingLocation();
    }

    location.map(|loc| {
        let coordinate = unsafe { loc.coordinate() };
        (coordinate.latitude, coordinate.longitude)
    })
}
