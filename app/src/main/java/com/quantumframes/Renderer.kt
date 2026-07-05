package com.quantumframes

import android.view.Surface

object Renderer {
    private var rendererPtr: Long = 0

    fun start(surface: Surface) {
        rendererPtr = nativeStart(surface)
    }

    fun resize(width: Int, height: Int) {
        if (rendererPtr != 0L) {
            nativeResize(rendererPtr, width, height)
        }
    }

    fun stop() {
        if (rendererPtr != 0L) {
            nativeStop(rendererPtr)
            rendererPtr = 0
        }
    }

    // JNI declarations
    private external fun nativeStart(surface: Surface): Long
    private external fun nativeResize(ptr: Long, width: Int, height: Int)
    private external fun nativeStop(ptr: Long)
}
