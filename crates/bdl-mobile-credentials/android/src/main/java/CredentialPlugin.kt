package com.yueli.bdl.credentials

import android.app.Activity
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties
import android.util.Base64
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.nio.charset.StandardCharsets
import java.security.KeyStore
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec

private const val STORE_NAME = "bdl.credentials"
private const val KEY_ALIAS = "bdl.account.cookie"
private const val COOKIE_PAYLOAD = "account_cookie_payload"
private const val COOKIE_IV = "account_cookie_iv"
private const val TRANSFORMATION = "AES/GCM/NoPadding"

@InvokeArg
class SaveCookieArgs {
  lateinit var cookie: String
}

@TauriPlugin
class CredentialPlugin(private val activity: Activity) : Plugin(activity) {
  private val preferences by lazy { activity.getSharedPreferences(STORE_NAME, Activity.MODE_PRIVATE) }

  @Command
  fun loadCookie(invoke: Invoke) {
    try {
      val payload = preferences.getString(COOKIE_PAYLOAD, null)
      val iv = preferences.getString(COOKIE_IV, null)
      if (payload == null || iv == null) {
        resolveCookie(invoke, null)
        return
      }

      val keyStore = loadKeyStore()
      val key = keyStore.getKey(KEY_ALIAS, null) as? SecretKey
      if (key == null) {
        clearStoredPayload()
        resolveCookie(invoke, null)
        return
      }

      val cipher = Cipher.getInstance(TRANSFORMATION)
      cipher.init(
        Cipher.DECRYPT_MODE,
        key,
        GCMParameterSpec(128, Base64.decode(iv, Base64.NO_WRAP)),
      )
      val plain = cipher.doFinal(Base64.decode(payload, Base64.NO_WRAP))
      resolveCookie(invoke, String(plain, StandardCharsets.UTF_8))
    } catch (error: Exception) {
      clearStoredPayload()
      invoke.reject("Unable to read stored account credential")
    }
  }

  @Command
  fun saveCookie(invoke: Invoke) {
    try {
      val args = invoke.parseArgs(SaveCookieArgs::class.java)
      val cipher = Cipher.getInstance(TRANSFORMATION)
      cipher.init(Cipher.ENCRYPT_MODE, getOrCreateKey())
      val encrypted = cipher.doFinal(args.cookie.toByteArray(StandardCharsets.UTF_8))

      preferences.edit()
        .putString(COOKIE_PAYLOAD, Base64.encodeToString(encrypted, Base64.NO_WRAP))
        .putString(COOKIE_IV, Base64.encodeToString(cipher.iv, Base64.NO_WRAP))
        .apply()
      invoke.resolve(JSObject())
    } catch (error: Exception) {
      invoke.reject("Unable to store account credential")
    }
  }

  @Command
  fun clearCookie(invoke: Invoke) {
    try {
      clearStoredPayload()
      val keyStore = loadKeyStore()
      if (keyStore.containsAlias(KEY_ALIAS)) {
        keyStore.deleteEntry(KEY_ALIAS)
      }
      invoke.resolve(JSObject())
    } catch (error: Exception) {
      invoke.reject("Unable to clear account credential")
    }
  }

  private fun resolveCookie(invoke: Invoke, cookie: String?) {
    val response = JSObject()
    response.put("cookie", cookie)
    invoke.resolve(response)
  }

  private fun clearStoredPayload() {
    preferences.edit().remove(COOKIE_PAYLOAD).remove(COOKIE_IV).apply()
  }

  private fun loadKeyStore(): KeyStore = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }

  private fun getOrCreateKey(): SecretKey {
    val keyStore = loadKeyStore()
    (keyStore.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }

    val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, "AndroidKeyStore")
    generator.init(
      KeyGenParameterSpec.Builder(
        KEY_ALIAS,
        KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
      )
        .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
        .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
        .build(),
    )
    return generator.generateKey()
  }
}
