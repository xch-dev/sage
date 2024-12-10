package com.rigidnetwork.sage

import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.updatePadding

class MainActivity : TauriActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)

      val rootView = window.decorView
      ViewCompat.setOnApplyWindowInsetsListener(rootView) { view, windowInsets ->
        val systemBars = windowInsets.getInsets(WindowInsetsCompat.Type.systemBars())
        val gestureInsets = windowInsets.getInsets(WindowInsetsCompat.Type.systemGestures())

        view.updatePadding(
          gestureInsets.left,
          gestureInsets.top,
          gestureInsets.right,
          gestureInsets.bottom
        )

        // You can access systemBars values here
        println("Top: ${systemBars.top}, Bottom: ${systemBars.bottom}")

        WindowInsetsCompat.CONSUMED
      }
    }
}