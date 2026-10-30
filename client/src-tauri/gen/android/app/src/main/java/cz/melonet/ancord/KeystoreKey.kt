package cz.melonet.ancord

import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

/**
 * Obálka klíče zařízení v Android Keystore (AES-256-GCM, klíč se z hardwaru nedá vyčíst). Rust (`android_keystore.rs`)
 * přes JNI zabalí náhodný klíč zařízení a uloží jen výsledek; bez klíče v Keystore (přeinstalace, obnova z cloudu) ho nerozbalí.
 * Klíč nevyžaduje autentizaci uživatele ani odemčené zařízení, aby `sync_once` z receiveru fungoval i při zamčeném telefonu;
 * zrušení zámku obrazovky proto klíč nesmaže.
 * Formát: iv (12 B) ‖ šifrotext s tagem. Chyba = `null` (Rust ji hlásí jako nedostupný Keystore).
 */
object KeystoreKey {
  private const val ALIAS = "cz.melonet.ancord.device"
  private const val TRANSFORM = "AES/GCM/NoPadding"
  private const val IV_LEN = 12

  private fun key(create: Boolean): SecretKey? {
    val ks = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
    (ks.getKey(ALIAS, null) as? SecretKey)?.let { return it }
    if (!create) return null
    val spec = KeyGenParameterSpec.Builder(ALIAS, KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT)
      .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
      .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
      .setKeySize(256)
      .build()
    return KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore").run {
      init(spec)
      generateKey()
    }
  }

  @JvmStatic
  fun wrap(plain: ByteArray): ByteArray? = try {
    val c = Cipher.getInstance(TRANSFORM).apply { init(Cipher.ENCRYPT_MODE, key(true)) }
    c.iv + c.doFinal(plain)
  } catch (_: Exception) {
    null
  }

  @JvmStatic
  fun unwrap(blob: ByteArray): ByteArray? = try {
    if (blob.size <= IV_LEN) {
      null
    } else {
      val k = key(false) ?: return null
      val c = Cipher.getInstance(TRANSFORM).apply { init(Cipher.DECRYPT_MODE, k, GCMParameterSpec(128, blob, 0, IV_LEN)) }
      c.doFinal(blob, IV_LEN, blob.size - IV_LEN)
    }
  } catch (_: Exception) {
    null
  }
}
