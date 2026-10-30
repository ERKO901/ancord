package cz.melonet.ancord

import android.os.Build
import android.os.Bundle
import android.view.View
import android.view.WindowManager
import androidx.activity.enableEdgeToEdge

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    // Musí být před super.onCreate: okno se vytvoří s příznakem hned, bez jediného snímku bez ochrany.
    window.setFlags(WindowManager.LayoutParams.FLAG_SECURE, WindowManager.LayoutParams.FLAG_SECURE)
    super.onCreate(savedInstanceState)
    hardenWindow()
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
