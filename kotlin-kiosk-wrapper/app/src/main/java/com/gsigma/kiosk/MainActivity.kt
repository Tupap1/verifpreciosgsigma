package com.gsigma.kiosk

import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.view.Gravity
import android.view.View
import android.view.WindowManager
import android.webkit.WebResourceError
import android.webkit.WebResourceRequest
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    private lateinit var mainLayout: LinearLayout
    private lateinit var webView: WebView
    private lateinit var errorLayout: LinearLayout
    private lateinit var txtErrorMsg: TextView
    private lateinit var inputServerIp: EditText
    private lateinit var btnRetry: Button

    private var currentServerUrl: String = "http://192.168.1.9:8080"

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

        // Contenedor principal
        mainLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.MATCH_PARENT
            )
        }

        webView = WebView(this).apply {
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.MATCH_PARENT
            )
        }

        setupErrorLayout()

        mainLayout.addView(webView)
        mainLayout.addView(errorLayout)
        setContentView(mainLayout)

        setupWebView()
        hideSystemUI()

        // Cargar IP de SharedPreferences
        val prefs = getSharedPreferences("GsigmaKiosk", Context.MODE_PRIVATE)
        currentServerUrl = prefs.getString("server_url", "http://192.168.1.9:8080") ?: "http://192.168.1.9:8080"

        loadUrlInWebView(currentServerUrl)

        // Iniciar comprobación de auto-actualizaciones en segundo plano
        ApkUpdater.checkAndInstallUpdate(this)
    }

    private fun setupErrorLayout() {
        errorLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(60, 60, 60, 60)
            gravity = Gravity.CENTER
            visibility = View.GONE
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.MATCH_PARENT
            )
        }

        val title = TextView(this).apply {
            text = "⚠️ Error de Conexión"
            textSize = 24f
            gravity = Gravity.CENTER
            setPadding(0, 0, 0, 20)
        }

        txtErrorMsg = TextView(this).apply {
            text = "No se pudo conectar al servidor."
            textSize = 16f
            gravity = Gravity.CENTER
            setPadding(0, 0, 0, 30)
        }

        inputServerIp = EditText(this).apply {
            hint = "http://192.168.1.9:8080"
            setPadding(30, 30, 30, 30)
        }

        btnRetry = Button(this).apply {
            text = "🔄 Guardar IP y Conectar"
            setPadding(30, 30, 30, 30)
            setOnClickListener {
                val newUrl = inputServerIp.text.toString().trim()
                if (newUrl.isNotEmpty()) {
                    currentServerUrl = newUrl
                    val prefs = getSharedPreferences("GsigmaKiosk", Context.MODE_PRIVATE)
                    prefs.edit().putString("server_url", newUrl).apply()
                    
                    showWebView()
                    loadUrlInWebView(currentServerUrl)
                } else {
                    Toast.makeText(this@MainActivity, "Ingrese una URL/IP válida", Toast.LENGTH_SHORT).show()
                }
            }
        }

        errorLayout.addView(title)
        errorLayout.addView(txtErrorMsg)
        errorLayout.addView(inputServerIp)
        errorLayout.addView(btnRetry)
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

            override fun onReceivedError(
                view: WebView?,
                request: WebResourceRequest?,
                error: WebResourceError?
            ) {
                super.onReceivedError(view, request, error)
                if (request?.isForMainFrame == true) {
                    showErrorScreen("No se pudo conectar a $currentServerUrl")
                }
            }

            @Suppress("DEPRECATION")
            override fun onReceivedError(
                view: WebView?,
                errorCode: Int,
                description: String?,
                failingUrl: String?
            ) {
                super.onReceivedError(view, errorCode, description, failingUrl)
                showErrorScreen("No se pudo conectar a $currentServerUrl")
            }
        }
    }

    private fun loadUrlInWebView(url: String) {
        inputServerIp.setText(url)
        webView.loadUrl(url)
    }

    private fun showWebView() {
        webView.visibility = View.VISIBLE
        errorLayout.visibility = View.GONE
    }

    private fun showErrorScreen(msg: String) {
        webView.visibility = View.GONE
        errorLayout.visibility = View.VISIBLE
        txtErrorMsg.text = msg
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

    private fun hideSystemUI() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT) {
            @Suppress("DEPRECATION")
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
