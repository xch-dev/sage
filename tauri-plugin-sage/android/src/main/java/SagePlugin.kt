package com.rigidnetwork.sage_plugin

import android.app.Activity
import android.nfc.NdefMessage
import android.nfc.NfcAdapter
import android.nfc.Tag
import android.os.Build
import android.os.Bundle
import android.webkit.WebView
import app.tauri.Logger
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.json.JSONArray

class Session(
    val invoke: Invoke
)

private fun fromU8Array(byteArray: ByteArray): JSONArray {
    val json = JSONArray()
    for (byte in byteArray) {
        json.put(byte)
    }
    return json
}

@TauriPlugin
class SagePlugin(private val activity: Activity) : Plugin(activity), NfcAdapter.ReaderCallback {
    private lateinit var webView: WebView
    private var nfcAdapter: NfcAdapter? = null
    private var session: Session? = null

    override fun load(webView: WebView) {
        super.load(webView)
        this.webView = webView
        this.nfcAdapter = NfcAdapter.getDefaultAdapter(activity.applicationContext)
    }

    override fun onPause() {
        stopNfcReading()
        super.onPause()
        Logger.info("NFC", "onPause")
    }

    override fun onResume() {
        super.onResume()
        Logger.info("NFC", "onResume")
        session?.let {
            startNfcReading()
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

        session = Session(invoke)
        startNfcReading()
    }

    private fun startNfcReading() {
        val flags = NfcAdapter.FLAG_READER_NFC_A or
                NfcAdapter.FLAG_READER_NFC_B or
                NfcAdapter.FLAG_READER_NFC_F or
                NfcAdapter.FLAG_READER_NFC_V or
                NfcAdapter.FLAG_READER_NO_PLATFORM_SOUNDS or
                NfcAdapter.FLAG_READER_NFC_BARCODE

        nfcAdapter?.enableReaderMode(activity, this, flags, Bundle())
    }

    private fun stopNfcReading() {
        nfcAdapter?.disableReaderMode(activity)
        session = null
    }

    override fun onTagDiscovered(tag: Tag) {
        try {
            val ndef = android.nfc.tech.Ndef.get(tag)
            ndef?.connect()
            
            val ndefMessage = ndef?.cachedNdefMessage
            val payloads = JSONArray()
            
            ndefMessage?.records?.forEach { record ->
                payloads.put(fromU8Array(record.payload))
            }

            activity.runOnUiThread {
                val ret = JSObject()
                ret.put("payloads", payloads)
                session?.invoke?.resolve(ret)
                stopNfcReading()
            }
            
            ndef?.close()
        } catch (e: Exception) {
            activity.runOnUiThread {
                session?.invoke?.reject("Failed to read tag: ${e.message}")
                stopNfcReading()
            }
        }
    }
}
