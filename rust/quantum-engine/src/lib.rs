use std::sync::Mutex;
use jni::JNIEnv;
use jni::objects::JObject;
use jni::sys::{jlong, jint};
use std::ptr;

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

#[no_mangle]
pub extern "system" fn Java_com_quantumframes_Renderer_nativeStart(
    mut env: JNIEnv,
    _class: JObject,
    surface: JObject,
) -> jlong {
    let ctx = ndk_context::android_context();
    let native_window = unsafe {
        ndk::native_window::NativeWindow::from_surface(env.get_native_interface(), surface.as_raw())
            .unwrap()
    };
    let native_window_ptr = native_window.ptr().as_ptr();

    let engine = Box::new(Engine::new(native_window_ptr as *mut std::ffi::c_void, 0, 0));
    let ptr = Box::into_raw(engine) as jlong;
    *ENGINE.lock().unwrap() = Some(EnginePtr(ptr as *mut Engine));
    ptr
}

#[no_mangle]
pub extern "system" fn Java_com_quantumframes_Renderer_nativeResize(
    mut env: JNIEnv,
    _class: JObject,
    ptr: jlong,
    width: jint,
    height: jint,
) {
    if ptr == 0 { return; }
    let engine = unsafe { &mut *(ptr as *mut Engine) };
    engine.resize(width as u32, height as u32);
}

#[no_mangle]
pub extern "system" fn Java_com_quantumframes_Renderer_nativeStop(
    mut env: JNIEnv,
    _class: JObject,
    ptr: jlong,
) {
    if ptr == 0 { return; }
    unsafe {
        let engine = Box::from_raw(ptr as *mut Engine);
        engine.stop();
    }
    *ENGINE.lock().unwrap() = None;
}
