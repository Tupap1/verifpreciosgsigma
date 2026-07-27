package com.gsigma.kiosk

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity

class ConfigActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        val context = this
        val prefs = getSharedPreferences("GsigmaKiosk", Context.MODE_PRIVATE)

        val layout = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(50, 50, 50, 50)
        }

        val inputUrl = EditText(context).apply {
            hint = "http://192.168.1.4:8080"
            setText(prefs.getString("server_url", "http://192.168.1.4:8080"))
        }

        val btnSave = Button(context).apply {
            text = "Guardar IP y Abrir Verificador"
            setOnClickListener {
                val url = inputUrl.text.toString().trim()
                if (url.isNotEmpty()) {
                    prefs.edit().putString("server_url", url).apply()
                    Toast.makeText(context, "IP Guardada", Toast.LENGTH_SHORT).show()
                    startActivity(Intent(context, MainActivity::class.java))
                    finish()
                } else {
                    Toast.makeText(context, "Ingrese una IP válida", Toast.LENGTH_SHORT).show()
                }
            }
        }

        layout.addView(inputUrl)
        layout.addView(btnSave)
        setContentView(layout)
    }
}
