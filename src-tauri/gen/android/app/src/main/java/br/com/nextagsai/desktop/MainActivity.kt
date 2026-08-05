package br.com.nextagsai.desktop

import android.os.Bundle
import android.view.View
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)

    // Com targetSdk 36 o edge-to-edge e imposto pelo sistema e nao ha opt-out, entao a WebView
    // desenha atras da status bar e da barra de navegacao. O site tem uma barra fixa de 48px em
    // top:0 que ficaria encoberta.
    //
    // Tratar os insets aqui tambem resolve o teclado: o manifest declara adjustResize, mas em
    // Android 15+ com edge-to-edge isso nao basta, e window.visualViewport nao atualiza quando o
    // IME abre (tauri#10631), logo o proprio site nao consegue se defender. Aplicamos o maior
    // entre o inset das barras e o do IME no padding inferior.
    val root = findViewById<View>(android.R.id.content)
    ViewCompat.setOnApplyWindowInsetsListener(root) { view, insets ->
      val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars())
      val ime = insets.getInsets(WindowInsetsCompat.Type.ime())
      view.setPadding(bars.left, bars.top, bars.right, maxOf(bars.bottom, ime.bottom))
      insets
    }
  }
}
