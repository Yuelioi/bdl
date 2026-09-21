#[cfg(target_os = "android")]
mod android;

#[cfg(target_os = "android")]
pub use android::init;
