// utils.js — utility functions for Su!
import { invoke, toastEl, qrCache } from './state.js';
var _toastTimer = null;
import { t } from './i18n/index.js';

function fmtSize(bytes) {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1048576) return (bytes/1024).toFixed(1) + " KB";
  if (bytes < 1073741824) return (bytes/1048576).toFixed(1) + " MB";
  return (bytes/1073741824).toFixed(2) + " GB";
}

function toast(msg) {
  if (_toastTimer) clearTimeout(_toastTimer);
  toastEl.textContent = msg;
  toastEl.classList.remove("hidden");
  _toastTimer = setTimeout(function() { toastEl.classList.add("hidden"); }, 2200);
}

async function copyText(text) {
  // Show toast immediately
  try { toast(t("toast_copied")); } catch(e) {}
  // Then do clipboard copy
  try {
    await navigator.clipboard.writeText(text);
  } catch(e) {
    var ta = document.createElement("textarea");
    ta.value = text;
    ta.style.cssText = "position:fixed;opacity:0";
    document.body.appendChild(ta);
    ta.select();
    document.execCommand("copy");
    document.body.removeChild(ta);
  }
}

function esc(s) {
  var d = document.createElement("div");
  d.appendChild(document.createTextNode(s));
  return d.innerHTML;
}

async function getQrDataUrl(text, size) {
  var key = text + "|" + (size || 256);
  if (qrCache[key]) return qrCache[key];
  try {
    qrCache[key] = await invoke("generate_qr", { text: text, size: size || 256 });
    return qrCache[key];
  } catch (e) {
    return "";
  }
}

export { fmtSize, toast, copyText, esc, getQrDataUrl };
