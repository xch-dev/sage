package com.rigidnetwork.sage_plugin

import android.app.Activity
import android.app.PendingIntent
import android.content.Intent
import android.content.IntentFilter
import android.nfc.NdefMessage
import android.nfc.NfcAdapter
import android.nfc.Tag
import android.os.Build
import android.os.Parcelable
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin

class Session(
    val invoke: Invoke
)

@TauriPlugin
class SagePlugin(private val activity: Activity) : Plugin(activity) {
    private lateinit var webView: WebView
    private var nfcAdapter: NfcAdapter? = null
    private var session: Session? = null

    override fun load(webView: WebView) {
        super.load(webView)
        this.webView = webView
        this.nfcAdapter = NfcAdapter.getDefaultAdapter(activity.applicationContext)
    }

    override fun onNewIntent(intent: Intent) {
        Logger.info("NFC", "onNewIntent")
        super.onNewIntent(intent)
        readTag(intent)
    }

    override fun onPause() {
        disableNFCInForeground()
        super.onPause()
        Logger.info("NFC", "onPause")
    }

    override fun onResume() {
        super.onResume()
        Logger.info("NFC", "onResume")
        session?.let {
            enableNFCInForeground()
        }
    }

    @Command
    fun isNdefAvailable(invoke: Invoke) {
        val ret = JSObject()
        ret.put("available", nfcAdapter?.isEnabled == true)
        invoke.resolve(ret)
    }

    @Command
    fun getNdefPayloads(invoke: Invoke) {
        if (nfcAdapter?.isEnabled != true) {
            invoke.reject("NFC is not available")
            return
        }

        enableNFCInForeground()
        session = Session(invoke)
    }

    private fun readTag(intent: Intent) {
        try {
            val rawMessages = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                intent.getParcelableArrayExtra(NfcAdapter.EXTRA_NDEF_MESSAGES, Parcelable::class.java)
            } else {
                @Suppress("DEPRECATION")
                intent.getParcelableArrayExtra(NfcAdapter.EXTRA_NDEF_MESSAGES)
            }

            val payloads = mutableListOf<ByteArray>()
            
            rawMessages?.let { messages ->
                for (message in messages) {
                    val ndefMessage = message as NdefMessage
                    for (record in ndefMessage.records) {
                        payloads.add(record.payload)
                    }
                }
            }

            val ret = JSObject()
            ret.put("payloads", payloads.toTypedArray())
            session?.invoke?.resolve(ret)
        } catch (e: Exception) {
            session?.invoke?.reject("Failed to read tag: ${e.message}")
        } finally {
            session = null
            disableNFCInForeground()
        }
    }

    private fun enableNFCInForeground() {
        val flag =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE else PendingIntent.FLAG_UPDATE_CURRENT
        val pendingIntent = PendingIntent.getActivity(
            activity, 0,
            Intent(activity, activity.javaClass).addFlags(Intent.FLAG_ACTIVITY_SINGLE_TOP),
            flag
        )

        val intentFilter = IntentFilter(NfcAdapter.ACTION_NDEF_DISCOVERED)
        nfcAdapter?.enableForegroundDispatch(activity, pendingIntent, arrayOf(intentFilter), null)
    }

    private fun disableNFCInForeground() {
        activity.runOnUiThread {
            nfcAdapter?.disableForegroundDispatch(activity)
        }
    }
}
