//! JNI boundary for the Android NOCTIS Engine Lab.
//! The parser itself remains in the independent noctis-html crate.

use jni::objects::{JObject, JString};
use jni::sys::jstring;
use jni::JNIEnv;
use noctis_html::parse_document;
use std::ptr;

/// Parse one HTML document with NOCTIS and return the diagnostic DOM tree.
///
/// JNI name corresponds to:
/// tech.dmj.noctis.MainActivity.nativeParseHtml(String)
#[no_mangle]
pub extern "system" fn Java_tech_dmj_noctis_MainActivity_nativeParseHtml(
    mut env: JNIEnv,
    _activity: JObject,
    input: JString,
) -> jstring {
    let source: String = match env.get_string(&input) {
        Ok(value) => value.into(),
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/IllegalArgumentException",
                "NOCTIS could not read the supplied HTML string",
            );
            return ptr::null_mut();
        }
    };

    let output = parse_document(&source).dump();
    match env.new_string(output) {
        Ok(value) => value.into_raw(),
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                "NOCTIS could not allocate the parser output string",
            );
            ptr::null_mut()
        }
    }
}

/// Return a small build identifier for the Android preview UI.
#[no_mangle]
pub extern "system" fn Java_tech_dmj_noctis_MainActivity_nativeEngineVersion(
    mut env: JNIEnv,
    _activity: JObject,
) -> jstring {
    match env.new_string("NOCTIS Engine 0.1.0 | HTML/DOM preview") {
        Ok(value) => value.into_raw(),
        Err(_) => {
            let _ = env.throw_new(
                "java/lang/RuntimeException",
                "Could not allocate engine version",
            );
            ptr::null_mut()
        }
    }
}
