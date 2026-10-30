package cz.melonet.ancord

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.util.Log
import kotlin.concurrent.thread

/** Prototyp 13.10: probuzení spustí `sync_once` v Rustu bez Activity a WebView. */
class SyncReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        val pending = goAsync()
        val dir = context.dataDir.absolutePath
        thread {
            try {
                System.loadLibrary("ancord_client_lib")
                Log.i("ancord", "sync_once vrátil ${nativeSync(dir)}")
            } catch (e: Throwable) {
                Log.e("ancord", "sync_once selhal", e)
            } finally {
                pending.finish()
            }
        }
    }

    companion object {
        @JvmStatic external fun nativeSync(dir: String): Int
    }
}
