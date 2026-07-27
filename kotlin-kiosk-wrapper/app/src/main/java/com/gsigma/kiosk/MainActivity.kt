package com.gsigma.kiosk

import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.view.View
import android.view.WindowManager
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    private lateinit var webView: WebView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Mantener pantalla encendida permanentemente
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

        webView = WebView(this)
        setContentView(webView)

        setupWebView()
        hideSystemUI()

        // Cargar URL guardada o ir a Configuración por defecto
        val prefs = getSharedPreferences("GsigmaKiosk", Context.MODE_PRIVATE)
        val serverUrl = prefs.getString("server_url", "http://192.168.1.4:8080") ?: "http://192.168.1.4:8080"

        if (serverUrl.trim().isEmpty()) {
            startActivity(Intent(this, ConfigActivity::class.java))
        } else {
            webView.loadUrl(serverUrl)
        }
    }

    private fun setupWebView() {
        val settings = webView.settings
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.databaseEnabled = true
        settings.allowFileAccess = true
        settings.cacheMode = WebSettings.LOAD_DEFAULT

        webView.webViewClient = object : WebViewClient() {
            override fun onPageFinished(view: WebView?, url: String?) {
                super.onPageFinished(view, url)
                hideSystemUI()
            }
        }
    }

    override fun onResume() {
        super.onResume()
        hideSystemUI()
        startKioskLock()
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        if (hasFocus) {
            hideSystemUI()
        }
    }

    // Modo Inmersivo Nativo Absoluto (Oculta barra de estado y botones de navegación)
    private fun hideSystemUI() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT) {
            window.decorView.systemUiVisibility = (
                View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY
                or View.SYSTEM_UI_FLAG_LAYOUT_STABLE
                or View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION
                or View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
                or View.SYSTEM_UI_FLAG_HIDE_NAVIGATION
                or View.SYSTEM_UI_FLAG_FULLSCREEN
            )
        }
    }

    // Activa la API nativa de fijación de pantalla (Lock Task Mode)
    private fun startKioskLock() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
            try {
                startLockTask()
            } catch (e: Exception) {
                // Modo Lock Task no disponible o sin permiso
            }
        }
    }
}
