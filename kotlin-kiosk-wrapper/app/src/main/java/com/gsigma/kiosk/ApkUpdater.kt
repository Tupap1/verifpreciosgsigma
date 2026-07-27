package com.gsigma.kiosk

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import androidx.core.content.FileProvider
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.net.HttpURLConnection
import java.net.URL
import kotlin.concurrent.thread

object ApkUpdater {

    fun checkAndInstallUpdate(context: Context, repoOwner: String = "Tupap1", repoName: String = "verifpreciosgsigma") {
        thread {
            try {
                val apiUrl = "https://api.github.com/repos/$repoOwner/$repoName/releases/latest"
                val url = URL(apiUrl)
                val conn = url.openConnection() as HttpURLConnection
                conn.requestMethod = "GET"
                conn.setRequestProperty("User-Agent", "VerificadorGsigmaApp")
                conn.connectTimeout = 5000
                conn.readTimeout = 5000

                if (conn.responseCode == 200) {
                    val responseText = conn.inputStream.bufferedReader().use { it.readText() }
                    val json = JSONObject(responseText)
                    val latestTag = json.optString("tag_name", "")

                    // Obtener versión instalada actual
                    val packageInfo = context.packageManager.getPackageInfo(context.packageName, 0)
                    val currentVersion = packageInfo.versionName ?: "1.0.0"

                    if (isNewerVersion(latestTag, currentVersion)) {
                        // Buscar el asset APK en la release
                        val assets = json.optJSONArray("assets")
                        var apkDownloadUrl = ""
                        if (assets != null) {
                            for (i in 0 until assets.length()) {
                                val asset = assets.getJSONObject(i)
                                val name = asset.optString("name", "")
                                if (name.endsWith(".apk")) {
                                    apkDownloadUrl = asset.optString("browser_download_url", "")
                                    break
                                }
                            }
                        }

                        if (apkDownloadUrl.isNotEmpty()) {
                            downloadAndPromptInstall(context, apkDownloadUrl)
                        }
                    }
                }
            } catch (e: Exception) {
                e.printStackTrace()
            }
        }
    }

    private fun isNewerVersion(latestTag: String, currentVersion: String): Boolean {
        val cleanTag = latestTag.trim().removePrefix("v")
        val cleanCurrent = currentVersion.trim().removePrefix("v")
        return cleanTag.isNotEmpty() && cleanTag != cleanCurrent
    }

    private fun downloadAndPromptInstall(context: Context, apkUrl: String) {
        try {
            val url = URL(apkUrl)
            val conn = url.openConnection() as HttpURLConnection
            conn.connectTimeout = 10000
            conn.readTimeout = 10000
            conn.connect()

            val apkFile = File(context.cacheDir, "update.apk")
            if (apkFile.exists()) apkFile.delete()

            conn.inputStream.use { input ->
                FileOutputStream(apkFile).use { output ->
                    input.copyTo(output)
                }
            }

            // Prompt install via Intent
            val intent = Intent(Intent.ACTION_VIEW).apply {
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_GRANT_READ_URI_PERMISSION
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                    val contentUri = FileProvider.getUriForFile(context, "${context.packageName}.fileprovider", apkFile)
                    setDataAndType(contentUri, "application/vnd.android.package-archive")
                } else {
                    setDataAndType(Uri.fromFile(apkFile), "application/vnd.android.package-archive")
                }
            }
            context.startActivity(intent)
        } catch (e: Exception) {
            e.printStackTrace()
        }
    }
}
