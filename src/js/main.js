// main.js — entry point for Su!
import { applyI18n, t } from "./i18n/index.js";
import { invoke, listen, initDomRefs } from "./state.js";
import { applyTheme, applyStyle, setupTitlebar, setupCollapse, setupTabs } from "./theme.js";
import { setupDragDrop, setupModal, setupClearAll, loadServerInfo, loadSendQr, updateEmpty, renderFileCard, openQrModal } from "./share.js";
import { setupReceived, showReceivedModal } from "./received.js";
import { setupSettings } from "./settings.js";

var currentTab = null;
var tabInitialized = { share: false, received: false, settings: false };
var uploadConfirmReady = false;
var currentConfirmAccept = null;
var currentConfirmReject = null;

function setupUploadConfirmListener() {
  if (uploadConfirmReady) return;
  uploadConfirmReady = true;
  listen("upload-requested", function(event) {
    console.log("[Su!] upload-requested:", event.payload);
    try {
      var d = event.payload;
      var modal = document.getElementById("upload-confirm-modal");
      var body = document.getElementById("confirm-body");
      var acceptBtn = document.getElementById("confirm-accept-btn");
      var rejectBtn = document.getElementById("confirm-reject-btn");
      if (!modal || !body || !acceptBtn || !rejectBtn) {
        console.error("[Su!] upload-confirm-modal elements missing");
        return;
      }
      body.textContent = t("received_from") + " " + (d.device || t("unknown_device")) + " · " + d.count + " " + t("files_count");
      modal.classList.remove("hidden");
      if (currentConfirmAccept) acceptBtn.removeEventListener("click", currentConfirmAccept);
      if (currentConfirmReject) rejectBtn.removeEventListener("click", currentConfirmReject);
      function respond(accepted) {
        console.log("[Su!] respond called, accepted=" + accepted + " id=" + d.id);
        modal.classList.add("hidden");
        if (currentConfirmAccept) acceptBtn.removeEventListener("click", currentConfirmAccept);
        if (currentConfirmReject) rejectBtn.removeEventListener("click", currentConfirmReject);
        currentConfirmAccept = null;
        currentConfirmReject = null;
        console.log("[Su!] calling confirm_upload with id=" + d.id + " accepted=" + accepted);
        invoke("confirm_upload", { id: d.id, accepted: accepted }).then(function(r) {
          console.log("[Su!] confirm_upload success:", r);
        }).catch(function(e) {
          console.error("[Su!] confirm_upload failed:", e);
        });
      }
      currentConfirmAccept = function() { console.log('[Su!] accept clicked!'); respond(true); };
      currentConfirmReject = function() { respond(false); };
      acceptBtn.addEventListener("click", currentConfirmAccept);
      rejectBtn.addEventListener("click", currentConfirmReject);
    } catch(e) { console.error("[Su!] upload-requested error:", e); }
  }).catch(function(e) {
    console.error("[Su!] listen upload-requested failed:", e);
  });
}

window.loadTab = async function(name) {
  var mc = document.getElementById("main-content");
  if (!mc) return;
  currentTab = name;
  try {
    var html = await invoke("read_page", { name: name });
    mc.innerHTML = html;
    var first = mc.firstElementChild;
    if (first) first.classList.remove("hidden");
    initDomRefs();
    var lang = localStorage.getItem("su-lang") || "zh-CN";
   applyI18n(lang);
    if (name === "share") {
      setupDragDrop();
      if (!tabInitialized.share) {
        setupModal();
        tabInitialized.share = true;
      }
      setupClearAll();
      loadServerInfo();
      loadSendQr();
      loadActiveShares();
      updateEmpty();
    } else if (name === "received") {
     setupReceived();
   } else if (name === "settings") {
      setupSettings();
    }
  } catch(e) {
    console.error("[Su!] loadTab failed:", e);
  }
};

function loadActiveShares() {
  invoke("get_active_shares").then(function(shares) {
    var fl = document.getElementById("file-list");
    if (!fl) return;
    fl.innerHTML = "";
    console.log("[Su!] loadActiveShares:", shares.length);
    shares.forEach(function(file) { renderFileCard(file); });
    updateEmpty();
  }).catch(function(e) { console.error("[Su!] loadActiveShares:", e); });
}

function listenForCliShares() {
  listen("share-added", function(event) {
    var payload = event.payload;
    if (!payload) return;
    var mc = document.getElementById("main-content");
    var onShareTab = mc && mc.querySelector("#tab-share");
    if (!onShareTab) {
    var shareTab = document.querySelector('[data-tab="share"]');
    if (shareTab && typeof shareTab.click === "function") shareTab.click();
    }
    renderFileCard(payload);
    openQrModal(payload);
    updateEmpty();
  });
}

window.addEventListener("DOMContentLoaded", function() {
  initDomRefs();
  (function(){var l=localStorage.getItem("su-lang")||"zh-CN";applyI18n(l);invoke("set_lang",{lang:l}).catch(function(){})})();
  applyTheme();
  applyStyle();
  setupTitlebar();
  setupCollapse();
  setupTabs();
  setupUploadConfirmListener();
  // Setup batch-complete listener for received popup (always active)
  listen("batch-complete", function(event) {
    console.log("[Su!] batch-complete event:", event.payload);
    var d = event.payload;
    if (d && localStorage.getItem("su-popup-enabled") !== "false") {
      showReceivedModal(d);
    }
  }).catch(function(e) { console.error("[Su!] listen batch-complete failed:", e); });
  listenForCliShares();
  // Listen for share auto-destroy events
  listen("share-destroyed", function(event) {
    console.log("[Su!] share-destroyed:", event.payload);
    loadActiveShares();
  }).catch(function(e) { console.error("[Su!] listen share-destroyed failed:", e); });
  window.loadTab("share");
  // Check for pending popup from CLI cold start
  invoke("get_popup_data").then(function(data) {
    if (data && data.url) {
      renderFileCard({ id: data.bundle_id || "cli", name: data.name, url: data.url, size: 0, files: [] });
      openQrModal({ name: data.name, url: data.url, size: 0, files: [] });
      updateEmpty();
      invoke("store_popup_data", { url: "", name: "" }).catch(function() {});
    }
  }).catch(function() {});
  (function() {
    var sn = localStorage.getItem("su-sound") || "\u6295\u9012";
    var se = localStorage.getItem("su-sound-enabled") !== "false";
    invoke("set_sound_settings", { enabled: se, name: sn }).catch(function() {});
  })();
});
