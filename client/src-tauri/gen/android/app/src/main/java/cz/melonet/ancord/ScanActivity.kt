package cz.melonet.ancord

import android.content.Intent
import android.os.Bundle
import android.view.WindowManager
import androidx.appcompat.app.AppCompatActivity
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.ImageProxy
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.core.content.ContextCompat
import com.google.zxing.BarcodeFormat
import com.google.zxing.BinaryBitmap
import com.google.zxing.DecodeHintType
import com.google.zxing.MultiFormatReader
import com.google.zxing.PlanarYUVLuminanceSource
import com.google.zxing.common.HybridBinarizer
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Skener QR s adresou kontaktu. Vrací jen nalezený text (`EXTRA_TEXT`); co s ním je (je to adresa?), rozhodne Rust.
 * Obraz z kamery se nikam neukládá ani neposílá, analyzuje se v paměti po snímcích.
 */
class ScanActivity : AppCompatActivity() {
  companion object {
    const val EXTRA_TEXT = "text"
  }

  private val executor = Executors.newSingleThreadExecutor()
  private val done = AtomicBoolean(false)
  private val reader = MultiFormatReader().apply {
    setHints(mapOf(DecodeHintType.POSSIBLE_FORMATS to listOf(BarcodeFormat.QR_CODE)))
  }

  override fun onCreate(savedInstanceState: Bundle?) {
    window.setFlags(WindowManager.LayoutParams.FLAG_SECURE, WindowManager.LayoutParams.FLAG_SECURE)
    super.onCreate(savedInstanceState)
    val preview = PreviewView(this)
    setContentView(preview)
    val future = ProcessCameraProvider.getInstance(this)
    future.addListener({
      val provider = future.get()
      val use = Preview.Builder().build().also { it.surfaceProvider = preview.surfaceProvider }
      val analysis = ImageAnalysis.Builder()
        .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
        .build()
        .also { it.setAnalyzer(executor, ::analyze) }
      provider.unbindAll()
      provider.bindToLifecycle(this, CameraSelector.DEFAULT_BACK_CAMERA, use, analysis)
    }, ContextCompat.getMainExecutor(this))
  }

  private fun analyze(image: ImageProxy) {
    image.use {
      if (done.get()) return
      val plane = it.planes[0]
      val buf = plane.buffer
      val data = ByteArray(buf.remaining()).also { b -> buf.get(b) }
      val source = PlanarYUVLuminanceSource(data, plane.rowStride, it.height, 0, 0, it.width, it.height, false)
      val text = try {
        reader.decodeWithState(BinaryBitmap(HybridBinarizer(source))).text
      } catch (_: Exception) {
        null
      } finally {
        reader.reset()
      }
      if (text != null && done.compareAndSet(false, true)) {
        runOnUiThread {
          setResult(RESULT_OK, Intent().putExtra(EXTRA_TEXT, text))
          finish()
        }
      }
    }
  }

  override fun onDestroy() {
    super.onDestroy()
    executor.shutdown()
  }
}
