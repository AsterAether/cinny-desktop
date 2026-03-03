(function() {
  'use strict';

  console.log('[Polyfill] IIFE started - polyfill script is executing');

  // Don't overwrite if Notification already exists natively
  if ('Notification' in window && window.Notification.toString().indexOf('native') !== -1) {
    console.log('[Polyfill] Native Notification API detected, skipping polyfill');
    return;
  }

  console.log('[Polyfill] Installing Web Notification API polyfill');

  const { invoke, Channel } = window.__TAURI__.core;

  // Store notification instances
  const notifications = new Map();
  let notificationCounter = 0;

  // Permission state
  let permissionState = 'default';

  class NotificationPolyfill {
    constructor(title, options = {}) {
      if (!title) {
        throw new TypeError('Failed to construct \'Notification\': 1 argument required, but only 0 present.');
      }

      console.log('[Polyfill] Creating notification with options:', JSON.stringify(options));

      this.title = title;
      this.body = options.body || '';
      this.icon = options.icon || '';
      this.tag = options.tag || '';
      this.data = options.data || null;

      console.log('[Polyfill] Stored data field:', JSON.stringify(this.data));

      // Generate unique ID
      this.id = options.tag || `notif_${++notificationCounter}_${Date.now()}`;

      // Event handlers
      this.onclick = null;
      this.onshow = null;
      this.onclose = null;
      this.onerror = null;

      // Store reference
      notifications.set(this.id, this);

      // Show notification
      this._show(options);
    }

    async _show(options) {
      try {
        console.log('[Polyfill] _show called with options:', JSON.stringify(options));
        console.log('[Polyfill] this.data:', JSON.stringify(this.data));

        await invoke('show_notification', {
          id: this.id,
          title: this.title,
          body: this.body,
          options: {
            body: this.body,
            icon: this.icon,
            tag: this.tag,
            data: this.data,
            room_id: options.roomId || options.room_id || (this.data && this.data.roomId) || (this.data && this.data.room_id),
            event_id: options.eventId || options.event_id || (this.data && this.data.eventId) || (this.data && this.data.event_id),
            room_type: options.roomType || options.room_type || (this.data && this.data.roomType) || (this.data && this.data.room_type),
            has_mention: options.hasMention || options.has_mention || (this.data && this.data.hasMention) || (this.data && this.data.has_mention),
          }
        });

        // Emit onshow event
        if (this.onshow) {
          this.onshow();
        }
      } catch (error) {
        console.error('[Polyfill] Failed to show notification:', error);
        if (this.onerror) {
          this.onerror(error);
        }
      }
    }

    close() {
      invoke('close_notification', { id: this.id })
        .then(() => {
          notifications.delete(this.id);
          if (this.onclose) {
            this.onclose();
          }
        })
        .catch(error => {
          console.error('[Polyfill] Failed to close notification:', error);
        });
    }

    static get permission() {
      return permissionState;
    }

    static async requestPermission() {
      try {
        const result = await invoke('request_notification_permission');
        permissionState = result;
        return result;
      } catch (error) {
        console.error('[Polyfill] Failed to request permission:', error);
        permissionState = 'denied';
        return 'denied';
      }
    }
  }

  // Initialize permission state
  invoke('get_notification_permission')
    .then(result => {
      permissionState = result;
    })
    .catch(error => {
      console.error('[Polyfill] Failed to get permission:', error);
      permissionState = 'default';
    });

  // Install polyfill
  window.Notification = NotificationPolyfill;

  // Listen for notification click events (Android)
  // Register plugin listener for actionPerformed events
  (async function registerNotificationListener() {
    try {
      const channel = new Channel();
      channel.onmessage = (payload) => {
        console.log('[Polyfill] Notification action performed:', JSON.stringify(payload));

        try {
          // According to plugin source, payload structure is:
          // { actionId, inputValue, notification: { id, title, body, extra: {...} } }
          let notificationId = null;

          // Try payload.notification.extra.notification_id (from plugin's sourceJson)
          if (payload.notification && payload.notification.extra && payload.notification.extra.notification_id) {
            notificationId = payload.notification.extra.notification_id;
            console.log('[Polyfill] Got notification ID from payload.notification.extra:', notificationId);
          }
          // Fallback to payload.notification.id (Android notification ID)
          else if (payload.notification && payload.notification.id) {
            notificationId = payload.notification.id.toString();
            console.log('[Polyfill] Got notification ID from payload.notification.id:', notificationId);
          }

          if (!notificationId) {
            console.warn('[Polyfill] No notification ID in payload');
            console.log('[Polyfill] Payload keys:', Object.keys(payload));
            return;
          }

          // Find the notification instance
          const notification = notifications.get(notificationId);
          if (notification && notification.onclick) {
            console.log('[Polyfill] Calling onclick handler for notification:', notificationId);
            notification.onclick.call(notification);
          } else {
            console.warn('[Polyfill] No onclick handler found for notification:', notificationId);
            console.log('[Polyfill] Available notifications:', Array.from(notifications.keys()));
          }
        } catch (error) {
          console.error('[Polyfill] Error handling notification action:', error);
        }
      };

      // Try snake_case first (newer versions)
      try {
        await invoke('plugin:notification|register_listener', {
          event: 'actionPerformed',
          handler: channel
        });
        console.log('[Polyfill] Registered notification action listener');
      } catch (error) {
        // Fallback to camelCase (older versions)
        await invoke('plugin:notification|registerListener', {
          event: 'actionPerformed',
          handler: channel
        });
        console.log('[Polyfill] Registered notification action listener (camelCase)');
      }
    } catch (error) {
      console.error('[Polyfill] Failed to register notification action listener:', error);
    }
  })();

  console.log('[Polyfill] Web Notification API polyfill installed successfully');

  // ============================================================
  // UnifiedPush Registration Module
  // ============================================================
  // Exposed as window.__cinny_register_push(accessToken, homeserverUrl)
  // Called by Cinny web app after login when running on Android.

  window.__cinny_register_push = async function(accessToken, homeserverUrl) {
    try {
      console.log('[Polyfill] Starting UnifiedPush registration...');
      window.__cinny_push_credentials = { accessToken, homeserverUrl };

      // Check if any UP distributor is installed
      let distributors;
      try {
        distributors = await invoke('get_push_distributors');
      } catch (e) {
        console.warn('[Polyfill] get_push_distributors failed:', e);
        return;
      }

      if (!distributors || distributors.length === 0) {
        console.log('[Polyfill] No UnifiedPush distributor installed, skipping background push setup');
        return;
      }

      console.log('[Polyfill] Found UP distributors:', distributors);

      // Save first distributor and register
      await invoke('save_push_distributor', { distributor: distributors[0] });
      await invoke('register_unifiedpush');

      console.log('[Polyfill] Registered with UP distributor:', distributors[0]);

      // Poll for endpoint (distributor responds asynchronously)
      const endpoint = await waitForEndpoint(30000); // 30s timeout
      if (!endpoint) {
        console.warn('[Polyfill] Timed out waiting for UP endpoint');
        return;
      }

      console.log('[Polyfill] Got UP endpoint:', endpoint);
      await registerMatrixPusher(accessToken, homeserverUrl, endpoint);

    } catch (error) {
      console.error('[Polyfill] UnifiedPush registration error:', error);
    }
  };

  async function waitForEndpoint(timeoutMs) {
    const start = Date.now();
    const pollInterval = 1000;

    while (Date.now() - start < timeoutMs) {
      try {
        const result = await invoke('get_push_endpoint');
        if (result && result !== '') {
          return result;
        }
      } catch (e) {
        // ignore
      }
      await new Promise(resolve => setTimeout(resolve, pollInterval));
    }
    return null;
  }

  async function registerMatrixPusher(accessToken, homeserverUrl, endpoint) {
    try {
      // Derive gateway URL from endpoint base URL
      // e.g. https://ntfy.sh/up_abc123 -> https://ntfy.sh/_matrix/push/v1/notify
      const url = new URL(endpoint);
      const gatewayUrl = `${url.protocol}//${url.host}/_matrix/push/v1/notify`;

      const hsUrl = homeserverUrl.replace(/\/$/, '');
      const response = await fetch(`${hsUrl}/_matrix/client/v3/pushers/set`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${accessToken}`,
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          kind: 'http',
          app_id: 'in.cinny.app',
          app_display_name: 'Cinny',
          device_display_name: 'Android',
          pushkey: endpoint,
          lang: 'en',
          data: {
            url: gatewayUrl,
          },
        }),
      });

      if (response.ok) {
        console.log('[Polyfill] Matrix pusher registered successfully');
      } else {
        const body = await response.text();
        console.error('[Polyfill] Failed to register Matrix pusher:', response.status, body);
      }
    } catch (error) {
      console.error('[Polyfill] Error registering Matrix pusher:', error);
    }
  }

  // Listen for endpoint changes (distributor may update the endpoint)
  (async function listenForEndpointChanges() {
    try {
      const channel = new Channel();
      channel.onmessage = async (payload) => {
        if (payload && payload.endpoint) {
          console.log('[Polyfill] UP endpoint changed:', payload.endpoint);
          if (window.__cinny_push_credentials) {
            const { accessToken, homeserverUrl } = window.__cinny_push_credentials;
            await registerMatrixPusher(accessToken, homeserverUrl, payload.endpoint);
          }
        }
      };
      await invoke('plugin:unified-push|register_listener', {
        event: 'newEndpoint',
        handler: channel,
      });
    } catch (e) {
      // Plugin event listener not available, endpoint changes won't auto-re-register
    }
  })();
})();
