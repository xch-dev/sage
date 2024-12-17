package com.plugin.safeareainsets

import android.app.Activity
import android.view.View
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import app.tauri.plugin.Invoke

@TauriPlugin
class InsetPlugin(private val activity: Activity): Plugin(activity) {
   private fun toDIPFromPixel(pixels: Float): Float {
       val density = activity.resources.displayMetrics.density
       return pixels / density
   }

   @Command
   fun getInsets(invoke: Invoke) {
       val rootView = activity.window.decorView
       ViewCompat.getRootWindowInsets(rootView)?.let { windowInsets ->
           val systemBars = windowInsets.getInsets(WindowInsetsCompat.Type.systemBars())
           val result = JSObject()
           result.put("top", toDIPFromPixel(systemBars.top.toFloat()).toDouble())
           result.put("bottom", toDIPFromPixel(systemBars.bottom.toFloat()).toDouble())
           result.put("left", toDIPFromPixel(systemBars.left.toFloat()).toDouble())
           result.put("right", toDIPFromPixel(systemBars.right.toFloat()).toDouble())
           invoke.resolve(result)
       } ?: invoke.reject("Failed to get window insets")
   }
}