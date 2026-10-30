package cz.melonet.ancord

import android.Manifest
import android.app.AlertDialog
import android.content.Intent
import android.content.pm.PackageManager
import android.webkit.JavascriptInterface
import androidx.activity.result.contract.ActivityResultContracts
import org.json.JSONObject
import android.os.Build
import android.os.Bundle
import android.view.View
import android.view.WindowManager
import android.webkit.WebView
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  private var webView: WebView? = null

  private val scanResult = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { r ->
    deliverScan(r.data?.getStringExtra(ScanActivity.EXTRA_TEXT))
  }
  private val cameraPermission = registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
    if (granted) scanResult.launch(Intent(this, ScanActivity::class.java)) else deliverScan(null)
  }

  /** Most pro UI: `window.AncordScan.scan()`, výsledek (text nebo null) přijde do `window.__ancordScan`. Vrací jen text QR. */
  private inner class ScanBridge {
    @JavascriptInterface
    fun scan() {
      runOnUiThread {
        if (checkSelfPermission(Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED) {
          scanResult.launch(Intent(this@MainActivity, ScanActivity::class.java))
        } else {
          cameraPermission.launch(Manifest.permission.CAMERA)
        }
      }
    }
  }

  private fun deliverScan(text: String?) {
    val arg = if (text == null) "null" else JSONObject.quote(text)
    webView?.evaluateJavascript("window.__ancordScan && window.__ancordScan($arg)", null)
  }

  companion object {
    // Nejnižší hlavní verze Android System WebView (Chromium); UI se překládá na ES2022 (Chrome 94+), rezerva na CSS a výkon.
    const val MIN_WEBVIEW_MAJOR = 110
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    // Musí být před super.onCreate: okno se vytvoří s příznakem hned, bez jediného snímku bez ochrany.
    window.setFlags(WindowManager.LayoutParams.FLAG_SECURE, WindowManager.LayoutParams.FLAG_SECURE)
    super.onCreate(savedInstanceState)
    hardenWindow()
  }

  // Autofill a ukládání formulářů se u WebView vypíná celé: hesla ani text zpráv nemají skončit u cizí služby.
  // Klávesnici „bez učení“ (IME_FLAG_NO_PERSONALIZED_LEARNING) tu nenastavíme, WebView vytváří wry; viz ARCHITECTURE.md §13.14.
  override fun onWebViewCreate(webView: WebView) {
    this.webView = webView
    webView.addJavascriptInterface(ScanBridge(), "AncordScan")
    checkWebViewVersion()
    webView.importantForAutofill = View.IMPORTANT_FOR_AUTOFILL_NO_EXCLUDE_DESCENDANTS
    @Suppress("DEPRECATION")
    webView.settings.saveFormData = false
  }

  // Starý WebView aplikaci nespustí spolehlivě (§13.1 slabiny Tauri): uživatel se to dozví, ne bílá obrazovka.
  private fun checkWebViewVersion() {
    val name = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) WebView.getCurrentWebViewPackage()?.versionName else null
    val major = name?.substringBefore('.')?.toIntOrNull() ?: return
    if (major >= MIN_WEBVIEW_MAJOR) return
    AlertDialog.Builder(this)
      .setTitle("Zastaralý WebView")
      .setMessage("Ancord potřebuje Android System WebView ve verzi $MIN_WEBVIEW_MAJOR nebo novější, máte $name. Aktualizujte ho v obchodě s aplikacemi.")
      .setCancelable(false)
      .setPositiveButton("Zavřít") { _, _ -> finish() }
      .show()
  }

  // Ochrana obsahu nad rámec FLAG_SECURE (ARCHITECTURE.md §9 bod 14). Každé API je na starších verzích
  // nedostupné, proto jsou volání podmíněná verzí; FLAG_SECURE platí vždy.
  private fun hardenWindow() {
    val decor = window.decorView
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
      setRecentsScreenshotEnabled(false)
    }
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
      try {
        window.setHideOverlayWindows(true)
      } catch (_: SecurityException) {
        // oprávnění HIDE_OVERLAY_WINDOWS odepřeno: FLAG_SECURE stále platí, aplikace nesmí spadnout
      }
    }
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
      decor.setAccessibilityDataSensitive(View.ACCESSIBILITY_DATA_SENSITIVE_YES)
    }
    if (Build.VERSION.SDK_INT >= 35) {
      decor.setContentSensitivity(View.CONTENT_SENSITIVITY_SENSITIVE)
    }
  }
}
