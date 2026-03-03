package `in`.cinny.app

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import app.tauri.plugin.JSObject
import kotlin.math.abs
import org.json.JSONObject
import org.unifiedpush.android.connector.FailedReason
import org.unifiedpush.android.connector.MessagingReceiver
import org.unifiedpush.android.connector.data.PushEndpoint
import org.unifiedpush.android.connector.data.PushMessage

class UnifiedPushReceiver : MessagingReceiver() {

    companion object {
        private const val TAG = "CinnyUP"
        private const val PREFS_NAME = "cinny_push_prefs"
        private const val PREF_ENDPOINT = "up_endpoint"
        private const val PREF_TOKEN = "up_token"
        private const val PREF_FAILURE_REASON = "up_failure_reason"
        private const val CHANNEL_ID = "default"
    }

    override fun onNewEndpoint(context: Context, endpoint: PushEndpoint, instance: String) {
        android.util.Log.i(TAG, "New UnifiedPush endpoint: ${endpoint.url}")

        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE).edit().apply {
            putString(PREF_ENDPOINT, endpoint.url)
            putString(PREF_TOKEN, instance)
            apply()
        }

        try {
            val data = JSObject()
            data.put("endpoint", endpoint.url)
            UnifiedPushPlugin.instance?.trigger("newEndpoint", data)
        } catch (e: Exception) {
            android.util.Log.w(TAG, "Could not trigger newEndpoint event: ${e.message}")
        }
    }

    override fun onMessage(context: Context, message: PushMessage, instance: String) {
        val messageStr = String(message.content, Charsets.UTF_8)
        android.util.Log.i(TAG, "Received UnifiedPush message")

        try {
            val json = JSONObject(messageStr)
            val notification = json.optJSONObject("notification") ?: json

            val roomId = notification.optString("room_id", "")
            val eventId = notification.optString("event_id", "")
            val senderDisplayName = notification.optString(
                "sender_display_name",
                notification.optString("sender", "Unknown")
            )
            val roomName = notification.optString("room_name", senderDisplayName)
            val content = notification.optJSONObject("content")
            val body = content?.optString("body", "New message") ?: "New message"

            showNotification(context, roomId, eventId, roomName, body)

        } catch (e: Exception) {
            android.util.Log.e(TAG, "Failed to parse push message: ${e.message}")
            showNotification(context, "", "", "Cinny", "New message")
        }
    }

    override fun onRegistrationFailed(context: Context, reason: FailedReason, instance: String) {
        android.util.Log.e(TAG, "UnifiedPush registration failed: $reason")
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE).edit().apply {
            putString(PREF_FAILURE_REASON, reason.toString())
            apply()
        }
    }

    override fun onUnregistered(context: Context, instance: String) {
        android.util.Log.i(TAG, "UnifiedPush unregistered")
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE).edit().apply {
            remove(PREF_ENDPOINT)
            remove(PREF_TOKEN)
            apply()
        }
        try {
            UnifiedPushPlugin.instance?.trigger("unregistered", JSObject())
        } catch (e: Exception) {
            android.util.Log.w(TAG, "Could not trigger unregistered event: ${e.message}")
        }
    }

    private fun showNotification(
        context: Context,
        roomId: String,
        eventId: String,
        title: String,
        body: String
    ) {
        val notificationId = (roomId + eventId).hashCode().let {
            if (it == Int.MIN_VALUE) 0 else abs(it)
        }

        ensureNotificationChannel(context)

        // Build sourceJson — must match TauriNotificationManager.buildIntent() format
        // so NotificationPlugin.onIntent() can handle the tap click event correctly.
        val extra = JSONObject()
        if (roomId.isNotEmpty()) extra.put("roomId", roomId)
        if (eventId.isNotEmpty()) extra.put("eventId", eventId)
        val sourceJson = JSONObject().apply {
            put("id", notificationId)
            put("title", title)
            put("body", body)
            put("extra", extra)
        }.toString()

        val tapIntent = context.packageManager.getLaunchIntentForPackage(context.packageName)
        if (tapIntent == null) {
            android.util.Log.e(TAG, "getLaunchIntentForPackage returned null for ${context.packageName}")
            return
        }
        tapIntent.apply {
            action = Intent.ACTION_MAIN
            addCategory(Intent.CATEGORY_LAUNCHER)
            flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
            putExtra("NotificationId", notificationId)
            putExtra("NotificationUserAction", "tap")
            putExtra("LocalNotficationObject", sourceJson)
            putExtra("NotificationRepeating", false)
        }

        var flags = PendingIntent.FLAG_CANCEL_CURRENT
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            flags = flags or PendingIntent.FLAG_IMMUTABLE
        }
        val pendingIntent = PendingIntent.getActivity(context, notificationId, tapIntent, flags)

        val notification = NotificationCompat.Builder(context, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle(title)
            .setContentText(body)
            .setAutoCancel(true)
            .setContentIntent(pendingIntent)
            .setPriority(NotificationCompat.PRIORITY_DEFAULT)
            .build()

        try {
            NotificationManagerCompat.from(context).notify(notificationId, notification)
        } catch (e: SecurityException) {
            android.util.Log.e(TAG, "No notification permission: ${e.message}")
        }
    }

    private fun ensureNotificationChannel(context: Context) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                CHANNEL_ID,
                "Messages",
                NotificationManager.IMPORTANCE_DEFAULT
            ).apply {
                description = "Matrix message notifications"
            }
            val manager = context.getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(channel)
        }
    }
}
