package `in`.cinny.app

import android.app.Activity
import android.webkit.WebView
import app.tauri.annotation.Command
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import org.unifiedpush.android.connector.UnifiedPush

@TauriPlugin
class UnifiedPushPlugin(private val activity: Activity) : Plugin(activity) {

    companion object {
        var instance: UnifiedPushPlugin? = null
        private const val PREFS_NAME = "cinny_push_prefs"
        private const val PREF_ENDPOINT = "up_endpoint"
        private const val INSTANCE_TAG = "cinny"
    }

    override fun load(webView: WebView) {
        instance = this
        super.load(webView)
    }

    @Command
    fun register(invoke: Invoke) {
        try {
            val distributors = UnifiedPush.getDistributors(activity)
            if (distributors.isEmpty()) {
                val result = JSObject()
                result.put("success", false)
                result.put("reason", "no_distributor")
                invoke.resolveObject(result)
                return
            }

            val savedDistributor = UnifiedPush.getSavedDistributor(activity)
            if (savedDistributor.isNullOrEmpty()) {
                UnifiedPush.saveDistributor(activity, distributors[0])
            }

            UnifiedPush.registerApp(activity, INSTANCE_TAG)

            val result = JSObject()
            result.put("success", true)
            invoke.resolveObject(result)
        } catch (e: Exception) {
            invoke.reject("Registration failed: ${e.message}")
        }
    }

    @Command
    fun unregister(invoke: Invoke) {
        try {
            UnifiedPush.unregisterApp(activity, INSTANCE_TAG)
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject("Unregistration failed: ${e.message}")
        }
    }

    @Command
    fun getEndpoint(invoke: Invoke) {
        try {
            val prefs = activity.getSharedPreferences(PREFS_NAME, android.content.Context.MODE_PRIVATE)
            val endpoint = prefs.getString(PREF_ENDPOINT, null)
            val result = JSObject()
            result.put("endpoint", endpoint)
            invoke.resolveObject(result)
        } catch (e: Exception) {
            invoke.reject("getEndpoint failed: ${e.message}")
        }
    }

    @Command
    fun getDistributors(invoke: Invoke) {
        try {
            val distributors: List<String> = UnifiedPush.getDistributors(activity)
            invoke.resolveObject(distributors)
        } catch (e: Exception) {
            invoke.reject("getDistributors failed: ${e.message}")
        }
    }

    @Command
    fun saveDistributor(invoke: Invoke) {
        try {
            val distributor = invoke.getArgs().getString("distributor")
            if (distributor.isNullOrEmpty()) {
                invoke.reject("distributor is required")
                return
            }
            UnifiedPush.saveDistributor(activity, distributor)
            invoke.resolve()
        } catch (e: Exception) {
            invoke.reject("saveDistributor failed: ${e.message}")
        }
    }
}
