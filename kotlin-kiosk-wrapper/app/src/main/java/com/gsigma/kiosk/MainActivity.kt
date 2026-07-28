package com.gsigma.kiosk

import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
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
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import java.net.HttpURLConnection
import java.net.URL
import kotlin.concurrent.thread

class MainActivity : AppCompatActivity() {

    private lateinit var rootLayout: FrameLayout
    private lateinit var webView: WebView
    private lateinit var errorLayout: LinearLayout
    private lateinit var txtErrorMsg: TextView
    private lateinit var txtTestResult: TextView
    private lateinit var inputServerIp: EditText
    private lateinit var btnTestConn: Button
    private lateinit var btnSaveAndConnect: Button

    private var currentServerUrl: String = "http://192.168.1.9:8080"

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        @Suppress("DEPRECATION")
        window.setFlags(
            WindowManager.LayoutParams.FLAG_FULLSCREEN,
            WindowManager.LayoutParams.FLAG_FULLSCREEN
        )
        window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON)

        rootLayout = FrameLayout(this).apply {
            layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
            setBackgroundColor(Color.WHITE)
        }

        webView = WebView(this).apply {
            layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
        }

        setupErrorLayout()

        rootLayout.addView(webView)
        rootLayout.addView(errorLayout)
        setContentView(rootLayout)

        setupWebView()
        hideSystemUI()

        // Cargar IP guardada
        val prefs = getSharedPreferences("GsigmaKiosk", Context.MODE_PRIVATE)
        val savedUrl = prefs.getString("server_url", null)

        if (savedUrl.isNullOrEmpty()) {
            showErrorScreen("Bienvenido al Verificador Gsigma.\nIngrese la IP del servidor local:")
        } else {
            currentServerUrl = savedUrl
            inputServerIp.setText(currentServerUrl)
            testAndConnect(currentServerUrl, autoLoad = true)
        }

        // Iniciar comprobación de auto-actualizaciones en segundo plano
        ApkUpdater.checkAndInstallUpdate(this)
    }

    private fun setupErrorLayout() {
        errorLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(60, 60, 60, 60)
            gravity = Gravity.CENTER
            visibility = View.GONE
            setBackgroundColor(Color.WHITE) // Fondo blanco nítido
            layoutParams = FrameLayout.LayoutParams(
                FrameLayout.LayoutParams.MATCH_PARENT,
                FrameLayout.LayoutParams.MATCH_PARENT
            )
        }

        val title = TextView(this).apply {
            text = "⚙️ Configuración del Servidor"
            textSize = 24f
            setTypeface(null, Typeface.BOLD)
            setTextColor(Color.parseColor("#0F172A")) // Dark Slate
            gravity = Gravity.CENTER
            setPadding(0, 0, 0, 15)
        }

        txtErrorMsg = TextView(this).apply {
            text = "Ingrese la dirección IP del servidor local:"
            textSize = 15f
            setTextColor(Color.parseColor("#475569")) // Medium Slate
            gravity = Gravity.CENTER
            setPadding(0, 0, 0, 20)
        }

        inputServerIp = EditText(this).apply {
            hint = "http://192.168.1.9:8080"
            setPadding(40, 30, 40, 30)
            setText("http://192.168.1.9:8080")
            setTextColor(Color.parseColor("#0F172A"))
            setHintTextColor(Color.parseColor("#94A3B8"))
            textSize = 16f

            // Background & Border styling
            val bg = GradientDrawable().apply {
                setColor(Color.parseColor("#F8FAFC"))
                setStroke(3, Color.parseColor("#CBD5E1"))
                cornerRadius = 16f
            }
            background = bg
        }

        txtTestResult = TextView(this).apply {
            text = ""
            textSize = 14f
            setTypeface(null, Typeface.BOLD)
            gravity = Gravity.CENTER
            setPadding(0, 20, 0, 25)
        }

        val btnContainer = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(0, 10, 0, 0)
        }

        btnTestConn = Button(this).apply {
            text = "🧪 Probar Conexión"
            setTextColor(Color.parseColor("#0F172A"))
            textSize = 14f
            setTypeface(null, Typeface.BOLD)
            val bg = GradientDrawable().apply {
                setColor(Color.parseColor("#E2E8F0"))
                cornerRadius = 16f
            }
            background = bg
            setPadding(30, 25, 30, 25)
            setOnClickListener {
                val url = inputServerIp.text.toString().trim()
                if (url.isNotEmpty()) {
                    testAndConnect(url, autoLoad = false)
                } else {
                    Toast.makeText(this@MainActivity, "Ingrese una IP válida", Toast.LENGTH_SHORT).show()
                }
            }
        }

        btnSaveAndConnect = Button(this).apply {
            text = "💾 Guardar y Conectar"
            setTextColor(Color.WHITE)
            textSize = 14f
            setTypeface(null, Typeface.BOLD)
            val bg = GradientDrawable().apply {
                setColor(Color.parseColor("#059669")) // Emerald Green
                cornerRadius = 16f
            }
            background = bg
            setPadding(30, 25, 30, 25)
            setOnClickListener {
                val url = inputServerIp.text.toString().trim()
                if (url.isNotEmpty()) {
                    testAndConnect(url, autoLoad = true)
                } else {
                    Toast.makeText(this@MainActivity, "Ingrese una IP válida", Toast.LENGTH_SHORT).show()
                }
            }
        }

        val lp1 = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f).apply {
            setMargins(0, 0, 15, 0)
        }
        val lp2 = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f).apply {
            setMargins(15, 0, 0, 0)
        }

        btnContainer.addView(btnTestConn, lp1)
        btnContainer.addView(btnSaveAndConnect, lp2)

        errorLayout.addView(title)
        errorLayout.addView(txtErrorMsg)
        errorLayout.addView(inputServerIp)
        errorLayout.addView(txtTestResult)
        errorLayout.addView(btnContainer)
    }

    private fun testAndConnect(targetUrl: String, autoLoad: Boolean) {
        val cleanUrl = if (!targetUrl.startsWith("http://") && !targetUrl.startsWith("https://")) {
            "http://$targetUrl"
        } else {
            targetUrl
        }

        runOnUiThread {
            txtTestResult.text = "⌛ Probando conexión a $cleanUrl..."
            txtTestResult.setTextColor(Color.parseColor("#2563EB"))
        }

        thread {
            var success = false
            var errorDetails = ""

            try {
                val healthUrl = URL("${cleanUrl.trimEnd('/')}/api/health")
                val conn = healthUrl.openConnection() as HttpURLConnection
                conn.connectTimeout = 3000
                conn.readTimeout = 3000
                conn.requestMethod = "GET"

                if (conn.responseCode == 200) {
                    success = true
                } else {
                    errorDetails = "HTTP Status ${conn.responseCode}"
                }
            } catch (e: Exception) {
                errorDetails = e.localizedMessage ?: "Timeout / Red inaccesible"
            }

            runOnUiThread {
                if (success) {
                    txtTestResult.text = "🟢 ✅ Conexión Exitosa con el Servidor"
                    txtTestResult.setTextColor(Color.parseColor("#059669"))

                    currentServerUrl = cleanUrl
                    val prefs = getSharedPreferences("GsigmaKiosk", Context.MODE_PRIVATE)
                    prefs.edit().putString("server_url", cleanUrl).apply()

                    if (autoLoad) {
                        showWebView()
                        loadUrlInWebView(cleanUrl)
                    }
                } else {
                    txtTestResult.text = "🔴 ❌ Falló la conexión ($errorDetails)\nVerifique la IP y que el servidor esté encendido."
                    txtTestResult.setTextColor(Color.parseColor("#DC2626"))
                    showErrorScreen("No se pudo conectar a $cleanUrl ($errorDetails)")
                }
            }
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

            override fun onReceivedError(
                view: WebView?,
                request: WebResourceRequest?,
                error: WebResourceError?
            ) {
                super.onReceivedError(view, request, error)
                if (request?.isForMainFrame == true) {
                    view?.stopLoading()
                    view?.loadUrl("about:blank")
                    showErrorScreen("Página web no disponible en $currentServerUrl")
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
                view?.stopLoading()
                view?.loadUrl("about:blank")
                showErrorScreen("Página web no disponible en $currentServerUrl ($description)")
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
        } else {
            closeSystemDialogs()
        }
    }

    private fun closeSystemDialogs() {
        try {
            @Suppress("DEPRECATION")
            val closeIntent = Intent(Intent.ACTION_CLOSE_SYSTEM_DIALOGS)
            sendBroadcast(closeIntent)
        } catch (e: Exception) {
            // Ignore
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
