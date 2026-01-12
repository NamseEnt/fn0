// Forte Error Overlay
// Displays build errors in the browser

(function() {
  'use strict';

  const overlayId = '__forte_error_overlay__';

  const styles = `
    #${overlayId} {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0, 0, 0, 0.85);
      color: #fff;
      font-family: 'SF Mono', Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
      font-size: 14px;
      z-index: 999999;
      overflow: auto;
      padding: 32px;
      box-sizing: border-box;
    }

    #${overlayId} .error-container {
      max-width: 900px;
      margin: 0 auto;
    }

    #${overlayId} .error-header {
      display: flex;
      align-items: center;
      gap: 12px;
      margin-bottom: 24px;
    }

    #${overlayId} .error-icon {
      width: 32px;
      height: 32px;
      background: #ff5555;
      border-radius: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
      font-size: 18px;
      font-weight: bold;
    }

    #${overlayId} .error-title {
      font-size: 20px;
      font-weight: 600;
      color: #ff5555;
    }

    #${overlayId} .error-file {
      color: #8be9fd;
      margin-bottom: 16px;
      font-size: 13px;
    }

    #${overlayId} .error-file a {
      color: #8be9fd;
      text-decoration: none;
    }

    #${overlayId} .error-file a:hover {
      text-decoration: underline;
    }

    #${overlayId} .error-message {
      background: rgba(255, 85, 85, 0.1);
      border-left: 4px solid #ff5555;
      padding: 16px;
      margin-bottom: 16px;
      white-space: pre-wrap;
      word-break: break-word;
      line-height: 1.5;
    }

    #${overlayId} .error-stack {
      color: #aaa;
      font-size: 12px;
      white-space: pre-wrap;
      word-break: break-word;
      line-height: 1.6;
    }

    #${overlayId} .close-button {
      position: absolute;
      top: 16px;
      right: 16px;
      background: transparent;
      border: 1px solid #666;
      color: #fff;
      padding: 8px 16px;
      cursor: pointer;
      font-size: 13px;
      border-radius: 4px;
      transition: background 0.2s;
    }

    #${overlayId} .close-button:hover {
      background: rgba(255, 255, 255, 0.1);
    }

    #${overlayId} .dismiss-hint {
      position: absolute;
      bottom: 16px;
      left: 50%;
      transform: translateX(-50%);
      color: #666;
      font-size: 12px;
    }
  `;

  function createOverlay() {
    let overlay = document.getElementById(overlayId);
    if (!overlay) {
      overlay = document.createElement('div');
      overlay.id = overlayId;

      const styleEl = document.createElement('style');
      styleEl.textContent = styles;
      overlay.appendChild(styleEl);

      document.body.appendChild(overlay);
    }
    return overlay;
  }

  function showError(error) {
    const overlay = createOverlay();

    const { message, file, loc, stack } = error;

    let fileInfo = '';
    if (file) {
      const location = loc ? `:${loc.line}:${loc.column}` : '';
      fileInfo = `<div class="error-file">${file}${location}</div>`;
    }

    let stackInfo = '';
    if (stack) {
      stackInfo = `<div class="error-stack">${escapeHtml(stack)}</div>`;
    }

    overlay.innerHTML = `
      <style>${styles}</style>
      <button class="close-button" onclick="window.__forte_dismissError()">Dismiss</button>
      <div class="error-container">
        <div class="error-header">
          <div class="error-icon">!</div>
          <div class="error-title">Build Error</div>
        </div>
        ${fileInfo}
        <div class="error-message">${escapeHtml(message)}</div>
        ${stackInfo}
      </div>
      <div class="dismiss-hint">Press Escape to dismiss</div>
    `;

    overlay.style.display = 'block';
  }

  function hideError() {
    const overlay = document.getElementById(overlayId);
    if (overlay) {
      overlay.style.display = 'none';
    }
  }

  function escapeHtml(str) {
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      hideError();
    }
  });

  window.__forte_showError = showError;
  window.__forte_hideError = hideError;
  window.__forte_dismissError = hideError;

  console.log('[Error Overlay] Initialized');
})();
