package com.quantumframes

import android.view.Surface

object Renderer {
    private var rendererPtr: Long = 0

    fun isStarted(): Boolean = rendererPtr != 0L

    fun start(surface: Surface, width: Int, height: Int) {
        rendererPtr = nativeStart(surface, width, height)
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
    private external fun nativeStart(surface: Surface, width: Int, height: Int): Long
    private external fun nativeResize(ptr: Long, width: Int, height: Int)
    private external fun nativeStop(ptr: Long)
}
