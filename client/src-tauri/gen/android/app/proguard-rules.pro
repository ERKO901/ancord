# Add project specific ProGuard rules here.
# You can control the set of applied configuration files using the
# proguardFiles setting in build.gradle.
#
# For more details, see
#   http://developer.android.com/guide/developing/tools/proguard.html

# If your project uses WebView with JS, uncomment the following
# and specify the fully qualified class name to the JavaScript interface
# class:
#-keepclassmembers class fqcn.of.javascript.interface.for.webview {
#   public *;
#}

# Uncomment this to preserve the line number information for
# debugging stack traces.
#-keepattributes SourceFile,LineNumberTable

# If you keep the line number information, uncomment this to
# hide the original source file name.
#-renamesourcefileattribute SourceFile
# Release (R8) maže, co nevidí volané z Kotlinu/Javy. Tyto metody volá Rust přes JNI, resp. JavaScript z WebView.
-keep class cz.melonet.ancord.KeystoreKey {
  public static byte[] wrap(byte[]);
  public static byte[] unwrap(byte[]);
}
-keepclassmembers class cz.melonet.ancord.MainActivity$ScanBridge {
  @android.webkit.JavascriptInterface public <methods>;
}
-keep class cz.melonet.ancord.SyncReceiver { *; }
-keep class cz.melonet.ancord.BiometricKey {
  public static boolean available();
  public static void forget();
  public static byte[] wrap(byte[]);
  public static byte[] unwrap(byte[]);
}
