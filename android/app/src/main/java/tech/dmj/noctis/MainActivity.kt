package tech.dmj.noctis

import android.app.Activity
import android.os.Bundle
import android.graphics.Typeface
import android.view.Gravity
import android.view.ViewGroup
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import android.text.InputType

class MainActivity : Activity() {
    private var nativeLibraryLoaded = false

    private external fun nativeParseHtml(source: String): String
    private external fun nativeEngineVersion(): String

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        nativeLibraryLoaded = try {
            System.loadLibrary("noctis_android_bridge")
            true
        } catch (error: UnsatisfiedLinkError) {
            false
        }

        val root = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(24, 28, 24, 24)
            setBackgroundColor(0xFFF4F6FA.toInt())
        }

        val heading = TextView(this).apply {
            text = "NOCTIS"
            textSize = 28f
            setTextColor(0xFF17233A.toInt())
            typeface = Typeface.DEFAULT_BOLD
        }
        root.addView(heading)

        val subtitle = TextView(this).apply {
            text = "Independent HTML and DOM Engine Lab"
            textSize = 14f
            setTextColor(0xFF53627A.toInt())
        }
        root.addView(subtitle)

        val version = TextView(this).apply {
            text = if (nativeLibraryLoaded) nativeEngineVersion()
            else "Native parser library unavailable"
            textSize = 12f
            setTextColor(0xFF667085.toInt())
            setPadding(0, 10, 0, 18)
        }
        root.addView(version)

        val input = EditText(this).apply {
            hint = "Enter an HTML document"
            minLines = 8
            maxLines = 14
            gravity = Gravity.TOP or Gravity.START
            inputType = InputType.TYPE_CLASS_TEXT or
                InputType.TYPE_TEXT_FLAG_MULTI_LINE or
                InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
            typeface = Typeface.MONOSPACE
            setText(
                "<!doctype html>\n" +
                    "<title>NOCTIS demo</title>\n" +
                    "<main id='app' class='panel'>\n" +
                    "  <h1>Hello from NOCTIS</h1>\n" +
                    "  <p>Our own HTML parser and DOM.</p>\n" +
                    "</main>"
            )
        }
        root.addView(
            input,
            LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
            )
        )

        val parseButton = Button(this).apply {
            text = "Parse with NOCTIS"
            isEnabled = nativeLibraryLoaded
        }
        root.addView(
            parseButton,
            LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
            )
        )

        val output = TextView(this).apply {
            text = if (nativeLibraryLoaded) {
                "The parsed DOM tree will appear here."
            } else {
                "This APK was built without the Rust native library. Rebuild it using the Android release workflow."
            }
            textSize = 12f
            typeface = Typeface.MONOSPACE
            setTextColor(0xFF17233A.toInt())
            setPadding(16, 16, 16, 16)
            setBackgroundColor(0xFFFFFFFF.toInt())
            textIsSelectable = true
        }
        val outputScroll = ScrollView(this).apply {
            addView(output)
        }
        root.addView(
            outputScroll,
            LinearLayout.LayoutParams(
                ViewGroup.LayoutParams.MATCH_PARENT,
                0,
                1f
            ).apply {
                topMargin = 16
            }
        )

        parseButton.setOnClickListener {
            output.text = try {
                nativeParseHtml(input.text.toString())
            } catch (error: Exception) {
                "NOCTIS parser error: ${error.message ?: error.javaClass.simpleName}"
            }
        }

        setContentView(root)
    }
}
