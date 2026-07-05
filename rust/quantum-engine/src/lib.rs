use std::sync::Mutex;
use jni::JNIEnv;
use jni::objects::JObject;
use jni::sys::{jlong, jint};
use std::panic::{catch_unwind, AssertUnwindSafe};

mod renderer;

struct Engine {
    renderer: renderer::Renderer,
}

impl Engine {
    fn new(surface: *mut std::ffi::c_void, width: u32, height: u32) -> Self {
        let renderer = pollster::block_on(renderer::Renderer::new(surface, width, height));
        Self { renderer }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    fn stop(self) {
        // drop renderer, cleanup wgpu
    }
}

struct EnginePtr(*mut Engine);
unsafe impl Send for EnginePtr {}
unsafe impl Sync for EnginePtr {}

static ENGINE: Mutex<Option<EnginePtr>> = Mutex::new(None);

fn log_err(msg: &str) {
    log::error!("{msg}");
    // Also write directly to logcat in case the logger isn't initialized.
    #[cfg(target_os = "android")]
    unsafe {
        use std::ffi::CString;
        extern "C" {
            fn __android_log_write(
                prio: std::os::raw::c_int,
                tag: *const std::os::raw::c_char,
                text: *const std::os::raw::c_char,
            ) -> std::os::raw::c_int;
        }
        let tag = CString::new("quantum_engine").unwrap();
        let text = CString::new(msg.replace('\0', " ")).unwrap_or_default();
        __android_log_write(6 /* ERROR */, tag.as_ptr(), text.as_ptr());
    }
}

#[no_mangle]
pub extern "system" fn Java_com_quantumframes_Renderer_nativeStart(
    env: JNIEnv,
    _class: JObject,
    surface: JObject,
    width: jint,
    height: jint,
) -> jlong {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let native_window = unsafe {
            ndk::native_window::NativeWindow::from_surface(
                env.get_native_interface(),
                surface.as_raw(),
            )
            .expect("failed to obtain ANativeWindow from Surface")
        };
        let native_window_ptr = native_window.ptr().as_ptr();

        // Keep the NativeWindow acquired for the lifetime of the engine by
        // leaking our reference; nativeStop releases the engine that owns it.
        std::mem::forget(native_window);

        let engine = Box::new(Engine::new(
            native_window_ptr as *mut std::ffi::c_void,
            width.max(1) as u32,
            height.max(1) as u32,
        ));
        Box::into_raw(engine) as jlong
    }));

    match result {
        Ok(ptr) => {
            *ENGINE.lock().unwrap() = Some(EnginePtr(ptr as *mut Engine));
            ptr
        }
        Err(e) => {
            let msg = e
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| e.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic".to_string());
            log_err(&format!("nativeStart failed: {msg}"));
            0
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_quantumframes_Renderer_nativeResize(
    _env: JNIEnv,
    _class: JObject,
    ptr: jlong,
    width: jint,
    height: jint,
) {
    if ptr == 0 { return; }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let engine = unsafe { &mut *(ptr as *mut Engine) };
        engine.resize(width as u32, height as u32);
    }));
    if result.is_err() {
        log_err("nativeResize panicked");
    }
}

#[no_mangle]
pub extern "system" fn Java_com_quantumframes_Renderer_nativeStop(
    _env: JNIEnv,
    _class: JObject,
    ptr: jlong,
) {
    if ptr == 0 { return; }
    let result = catch_unwind(AssertUnwindSafe(|| {
        unsafe {
            let engine = Box::from_raw(ptr as *mut Engine);
            engine.stop();
        }
    }));
    if result.is_err() {
        log_err("nativeStop panicked");
    }
    *ENGINE.lock().unwrap() = None;
}
