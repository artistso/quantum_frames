package com.quantumframes

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.viewinterop.AndroidView
import android.view.SurfaceHolder
import android.view.SurfaceView

class MainActivity : ComponentActivity() {
    companion object {
        init {
            System.loadLibrary("quantum_engine")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            AndroidView(
                factory = { context ->
                    SurfaceView(context).apply {
                        holder.addCallback(object : SurfaceHolder.Callback {
                            override fun surfaceCreated(holder: SurfaceHolder) {
                                Renderer.start(holder.surface)
                            }
                            override fun surfaceChanged(
                                holder: SurfaceHolder,
                                format: Int,
                                width: Int,
                                height: Int
                            ) {
                                Renderer.resize(width, height)
                            }
                            override fun surfaceDestroyed(holder: SurfaceHolder) {
                                Renderer.stop()
                            }
                        })
                    }
                },
                modifier = Modifier.fillMaxSize()
            )
        }
    }
}
