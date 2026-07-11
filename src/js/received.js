// received.js — received files list and modal
import { invoke, listen } from './state.js';
import { fmtSize, toast, esc } from './utils.js';
import { t } from './i18n/index.js';

function showReceivedModal(d) {
  var modal = document.getElementById("received-modal");
  var title = document.getElementById("received-modal-title");
  var detail = document.getElementById("received-modal-detail");
  var btn = document.getElementById("received-modal-btn");
  var x = document.getElementById("received-modal-x");
  if (!modal) return;
  title.textContent = t("received_from") + " " + (d.device || t("unknown_device"));
  var count = d.count || 1;
  detail.textContent = count + " " + t("files_count");
  modal.classList.remove("hidden");
  function close() { modal.classList.add("hidden"); }
  btn.onclick = function() {
    close();
    var rt = document.querySelector('[data-tab="received"]');
    if (rt) rt.click();
  };
  x.onclick = close;
  modal.querySelector(".modal-bg").onclick = close;
}

function setupReceived() {
  var receivedList = document.getElementById("received-list");
  if (!receivedList) { console.log("[Su!] received-list not found"); return; }

  window.refreshReceivedGlobal = async function refreshReceived() {
    try {
      var items = await invoke("get_received_files");
      if (!receivedList) return;
      if (items.length === 0) {
        receivedList.innerHTML = '<div class="received-empty" data-i18n="no_received">' + t("no_received") + '</div>';
        return;
      }
      receivedList.innerHTML = "";
      var rev = items.slice().reverse();
      var batches = {};
      var maxBatch = 0;
      rev.forEach(function(item) {
        var b = item.batch || 0;
        if (b > maxBatch) maxBatch = b;
        if (!batches[b]) batches[b] = [];
        batches[b].push(item);
      });
      var batchNums = Object.keys(batches).map(Number).sort(function(a, b) { return b - a; });

      function renderItem(item, isNew) {
        var ts = item.time || "";
        var d = new Date(ts);
        var valid = !isNaN(d.getTime());
        var shortTime = valid ? ("0" + d.getHours()).slice(-2) + ":" + ("0" + d.getMinutes()).slice(-2) : (ts.substring(11, 16) || ts);
        var row = document.createElement("div");
        row.className = "rcard" + (isNew ? " rcard-new" : "");
        row.innerHTML =
          '<div class="rcard-icon"><i class="fa-solid fa-file"></i></div>' +
          '<div class="rcard-info">' +
            '<div class="rcard-name" title="' + esc(item.name) + '">' + esc(item.name) + '</div>' +
            '<div class="rcard-meta">' + fmtSize(item.size) + '</div>' +
          '</div>' +
          '<div class="rcard-time">' + shortTime + '</div>' +
          '<button class="rcard-btn" data-path="' + esc(item.path || '') + '" title="打开文件夹"><i class="fa-solid fa-folder-open"></i></button>';
        receivedList.appendChild(row);
        row.querySelector('.rcard-btn').addEventListener('click', function(e) {
          e.stopPropagation();
          var p = this.dataset.path;
          if (!p) return;
          invoke('open_folder', { path: p }).catch(function() {});
        });
        row.addEventListener('dblclick', function(e) {
          var p = this.querySelector('.rcard-btn');
          if (p && p.dataset.path) invoke('open_path', { path: p.dataset.path }).catch(function() {});
        });
        row.style.cursor = 'pointer';
      }

      batchNums.forEach(function(bn) {
        var isLatest = bn === maxBatch;
        var files = batches[bn];
        var hdrTime = (files[0].time || "").substring(0, 16);
        var hdr = document.createElement("div");
        hdr.className = "rcard-header" + (isLatest ? " rcard-header-latest" : "");
        var deviceLabel = (files[0].device || "");
        if (deviceLabel) hdrTime += " " + t("received_from") + " " + deviceLabel;
        hdr.textContent = hdrTime;
        receivedList.appendChild(hdr);
        files.forEach(function(item) { renderItem(item, isLatest); });
      });
    } catch (e) { console.error("[Su!] refreshReceived error:", e); }
  };

  listen("file-received", function(event) {
    console.log("[Su!] file-received event:", event.payload);
    window.refreshReceivedGlobal();
  }).catch(function(e) {
    console.error("[Su!] listen file-received failed:", e);
  });

  listen("upload-requested", function(event) {
    console.log("[Su!] upload-requested:", event.payload);
    try {
      var d = event.payload;
      var modal = document.getElementById("upload-confirm-modal");
      var body = document.getElementById("confirm-body");
      var acceptBtn = document.getElementById("confirm-accept-btn");
      var rejectBtn = document.getElementById("confirm-reject-btn");
      if (!modal) return;
      body.textContent = t("received_from") + " " + (d.device || t("unknown_device")) + " \u00b7 " + d.count + " " + t("files_count");
      modal.classList.remove("hidden");
      function respond(accepted) {
        modal.classList.add("hidden");
        acceptBtn.onclick = null;
        rejectBtn.onclick = null;
        invoke("confirm_upload", { id: d.id, accepted: accepted }).catch(function(){});
      }
      acceptBtn.onclick = function() { respond(true); };
      rejectBtn.onclick = function() { respond(false); };
    } catch(e) { console.error("[Su!] upload-requested error:", e); }
  });

  listen("batch-complete", function(event) {
    console.log("[Su!] batch-complete event:", event.payload);
    var d = event.payload;
    if (d && localStorage.getItem("su-popup-enabled") !== "false") {
      showReceivedModal(d);
    }
  }).catch(function(e) {
    console.error("[Su!] listen batch-complete failed:", e);
  });

  var receivedTab = document.querySelector('[data-tab="received"]');
  if (receivedTab) {
    var observer = new MutationObserver(function() {
      if (receivedTab.classList.contains("active")) {
        window.refreshReceivedGlobal();
      }
    });
    observer.observe(receivedTab, { attributes: true, attributeFilter: ["class"] });
  }

  if (typeof window.refreshReceivedGlobal === "function") window.refreshReceivedGlobal();
}

export { setupReceived, showReceivedModal };
