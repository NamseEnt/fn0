// Forte HMR Client with React Fast Refresh
// This script runs in the browser and handles Hot Module Replacement

(function() {
  'use strict';

  const RECONNECT_DELAY = 1000;
  const MAX_RECONNECT_ATTEMPTS = 10;

  class HMRClient {
    constructor() {
      this.socket = null;
      this.reconnectAttempts = 0;
      this.moduleCallbacks = new Map();
      this.pendingUpdates = [];
      this.isFirstConnect = true;
      this.refreshPending = false;

      this.connect();
    }

    connect() {
      const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
      const wsUrl = `${protocol}//${location.host}/__hmr`;

      console.log('[HMR] Connecting to', wsUrl);
      this.socket = new WebSocket(wsUrl);

      this.socket.onopen = () => {
        console.log('[HMR] Connected');
        this.reconnectAttempts = 0;

        if (!this.isFirstConnect) {
          console.log('[HMR] Reconnected, checking for updates...');
        }
        this.isFirstConnect = false;
      };

      this.socket.onmessage = (event) => {
        try {
          const payload = JSON.parse(event.data);
          this.handleMessage(payload);
        } catch (e) {
          console.error('[HMR] Failed to parse message:', e);
        }
      };

      this.socket.onclose = () => {
        console.log('[HMR] Connection closed');
        this.scheduleReconnect();
      };

      this.socket.onerror = (error) => {
        console.error('[HMR] WebSocket error:', error);
      };
    }

    scheduleReconnect() {
      if (this.reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
        console.error('[HMR] Max reconnect attempts reached. Please refresh the page.');
        return;
      }

      this.reconnectAttempts++;
      const delay = RECONNECT_DELAY * Math.min(this.reconnectAttempts, 5);
      console.log(`[HMR] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${MAX_RECONNECT_ATTEMPTS})`);

      setTimeout(() => this.connect(), delay);
    }

    handleMessage(payload) {
      switch (payload.type) {
        case 'connected':
          console.log('[HMR] Server confirmed connection');
          break;

        case 'reload':
          console.log('[HMR] Full reload requested');
          location.reload();
          break;

        case 'update':
          console.log('[HMR] Update received:', payload.updates);
          if (window.__forte_hideError) {
            window.__forte_hideError();
          }
          this.handleUpdate(payload);
          break;

        case 'error':
          console.error('[HMR] Build error:', payload.message);
          if (payload.file) {
            console.error(`  File: ${payload.file}`);
          }
          if (payload.loc) {
            console.error(`  Location: line ${payload.loc.line}, column ${payload.loc.column}`);
          }
          if (window.__forte_showError) {
            window.__forte_showError({
              message: payload.message,
              file: payload.file,
              loc: payload.loc
            });
          }
          break;

        default:
          console.warn('[HMR] Unknown message type:', payload.type);
      }
    }

    async handleUpdate(payload) {
      const { updates, timestamp } = payload;

      for (const update of updates) {
        const { id, accepted_by } = update;

        const callbacks = this.moduleCallbacks.get(accepted_by);

        if (!callbacks || !callbacks.accept) {
          console.log(`[HMR] No HMR handler for ${accepted_by}, reloading...`);
          location.reload();
          return;
        }

        try {
          if (callbacks.dispose) {
            callbacks.dispose(callbacks.data);
          }

          const moduleUrl = `${id}?t=${timestamp}`;
          console.log(`[HMR] Fetching updated module: ${moduleUrl}`);

          const newModule = await import(moduleUrl);

          if (typeof callbacks.accept === 'function') {
            callbacks.accept(newModule);
          }

          console.log(`[HMR] Successfully updated ${id}`);
        } catch (error) {
          console.error(`[HMR] Failed to update ${id}:`, error);
          console.log('[HMR] Falling back to full reload');
          location.reload();
          return;
        }
      }

      this.scheduleRefresh();
    }

    scheduleRefresh() {
      if (this.refreshPending) return;
      this.refreshPending = true;

      requestAnimationFrame(() => {
        this.refreshPending = false;
        if (window.$RefreshRuntime$) {
          console.log('[HMR] Performing React Refresh');
          window.$RefreshRuntime$.performReactRefresh();
        }
      });
    }

    createHotContext(moduleId) {
      if (!this.moduleCallbacks.has(moduleId)) {
        this.moduleCallbacks.set(moduleId, {
          accept: null,
          dispose: null,
          data: {}
        });
      }

      const callbacks = this.moduleCallbacks.get(moduleId);

      return {
        accept: (deps, callback) => {
          if (typeof deps === 'function' || deps === undefined) {
            callbacks.accept = deps || true;
          } else if (Array.isArray(deps)) {
            callbacks.accept = callback || true;
          }
        },

        dispose: (callback) => {
          callbacks.dispose = callback;
        },

        get data() {
          return callbacks.data;
        },

        decline: () => {
          callbacks.accept = null;
        },

        invalidate: () => {
          console.log(`[HMR] Module ${moduleId} invalidated, reloading...`);
          location.reload();
        },

        prune: (callback) => {
          // TODO: Implement prune handling
        }
      };
    }
  }

  // Create global HMR client instance
  window.__forte_hmr = new HMRClient();

  // Export function to create hot context
  window.__forte_createHotContext = function(moduleId) {
    return window.__forte_hmr.createHotContext(moduleId);
  };

  console.log('[HMR] Client initialized');
})();
