package cz.melonet.ancord

import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import java.security.KeyStore
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * Obálka hlavního klíče úložiště klíčem z Android Keystore, který vydá jen biometrika (silná třída, při každém použití).
 * Rust (`android_keystore.rs`, `AndroidBiometric`) volá `wrap`/`unwrap` přes JNI z vlákna mimo UI; volání čeká na potvrzení
 * v systémovém dotazu. Klíč se zneplatní přidáním nového otisku (`setInvalidatedByBiometricEnrollment`), pak zůstává heslo.
 * Hlavní klíč nikdy nejde do WebView. Formát: iv (12 B) ‖ šifrotext s tagem. Chyba nebo zrušení = `null`.
 */
object BiometricKey {
  private const val ALIAS = "cz.melonet.ancord.bio"
  private const val TRANSFORM = "AES/GCM/NoPadding"
  private const val IV_LEN = 12
  private const val TIMEOUT_S = 120L

  private fun store(): KeyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

  @JvmStatic
  fun available(): Boolean {
    val activity = MainActivity.current?.get() ?: return false
    return BiometricManager.from(activity).canAuthenticate(BiometricManager.Authenticators.BIOMETRIC_STRONG) == BiometricManager.BIOMETRIC_SUCCESS
  }

  @JvmStatic
  fun forget() {
    try {
      store().deleteEntry(ALIAS)
    } catch (_: Exception) {
    }
  }

  private fun newKey(): SecretKey {
    forget()
    val b = KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
      .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
      .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
      .setKeySize(256)
      .setUserAuthenticationRequired(true)
      .setInvalidatedByBiometricEnrollment(true)
    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
      b.setUserAuthenticationParameters(0, KeyProperties.AUTH_BIOMETRIC_STRONG)
    }
    return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").run {
      init(b.build())
      generateKey()
    }
  }

  /** Zobrazí systémový dotaz a po úspěchu vrátí odemčený `Cipher`; zrušení, chyba i vypršení = `null`. */
  private fun authenticate(cipher: Cipher, title: String): Cipher? {
    val activity = MainActivity.current?.get() ?: return null
    val latch = CountDownLatch(1)
    var result: Cipher? = null
    activity.runOnUiThread {
      val prompt = BiometricPrompt(
        activity,
        ContextCompat.getMainExecutor(activity),
        object : BiometricPrompt.AuthenticationCallback() {
          override fun onAuthenticationSucceeded(r: BiometricPrompt.AuthenticationResult) {
            result = r.cryptoObject?.cipher
            latch.countDown()
          }

          override fun onAuthenticationError(code: Int, msg: CharSequence) {
            latch.countDown()
          }
        },
      )
      val info = BiometricPrompt.PromptInfo.Builder()
        .setTitle(title)
        .setNegativeButtonText("Použít heslo")
        .setAllowedAuthenticators(BiometricManager.Authenticators.BIOMETRIC_STRONG)
        .setConfirmationRequired(false)
        .build()
      prompt.authenticate(info, BiometricPrompt.CryptoObject(cipher))
    }
    return if (latch.await(TIMEOUT_S, TimeUnit.SECONDS)) result else null
  }

  @JvmStatic
  fun wrap(plain: ByteArray): ByteArray? = try {
    val c = Cipher.getInstance(TRANSFORM).apply { init(Cipher.ENCRYPT_MODE, newKey()) }
    val ready = authenticate(c, "Zapnout odemykání biometrikou")
    if (ready == null) {
      forget()
      null
    } else {
      ready.iv + ready.doFinal(plain)
    }
  } catch (_: Exception) {
    forget()
    null
  }

  @JvmStatic
  fun unwrap(blob: ByteArray): ByteArray? = try {
    if (blob.size <= IV_LEN) {
      null
    } else {
      val key = store().getKey(ALIAS, null) as? SecretKey
      if (key == null) {
        null
      } else {
        val c = Cipher.getInstance(TRANSFORM).apply { init(Cipher.DECRYPT_MODE, key, GCMParameterSpec(128, blob, 0, IV_LEN)) }
        authenticate(c, "Odemknout ancord")?.doFinal(blob, IV_LEN, blob.size - IV_LEN)
      }
    }
  } catch (_: Exception) {
    null
  }
}
