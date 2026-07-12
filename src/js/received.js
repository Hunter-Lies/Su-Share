// received.js - received files list and modal
import { invoke, listen } from './state.js';
import { fmtSize, toast, esc } from './utils.js';
import { t } from './i18n/index.js';

var receivedInitialized = false;
var currentConfirmAccept = null;
var currentConfirmReject = null;
var receivedObserver = null;

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
  var bg = modal.querySelector(".modal-bg");
  if (bg) bg.onclick = close;
}

function formatTime(ts) {
  var d = new Date(ts);
  if (!isNaN(d.getTime())) {
    return ("0" + d.getHours()).slice(-2) + ":" + ("0" + d.getMinutes()).slice(-2);
  }
  if (typeof ts === "string" && ts.length >= 16) return ts.substring(11, 16);
  return ts || "";
}

function renderReceivedItem(item, isNew, index) {
  var row = document.createElement("div");
  row.className = "rcard" + (isNew ? " rcard-new" : "");
  row.style.animationDelay = (index * 30) + "ms";

  var icon = document.createElement("div");
  icon.className = "rcard-icon";
  icon.innerHTML = '<i class="fa-solid fa-file"></i>';

  var info = document.createElement("div");
  info.className = "rcard-info";
  var name = document.createElement("div");
  name.className = "rcard-name";
  name.title = item.name;
  name.textContent = item.name;
  var meta = document.createElement("div");
  meta.className = "rcard-meta";
  meta.textContent = fmtSize(item.size);
  info.appendChild(name);
  info.appendChild(meta);

  var time = document.createElement("div");
  time.className = "rcard-time";
  time.textContent = formatTime(item.time);

  var openBtn = document.createElement("button");
  openBtn.className = "rcard-btn";
  openBtn.title = t("view_file");
  openBtn.innerHTML = '<i class="fa-solid fa-folder-open"></i>';
  openBtn.addEventListener('click', function(e) {
    e.stopPropagation();
    if (item.path) invoke('open_folder', { path: item.path }).catch(function() {});
  });

  row.appendChild(icon);
  row.appendChild(info);
  row.appendChild(time);
  row.appendChild(openBtn);

  row.addEventListener('dblclick', function() {
    if (item.path) invoke('open_path', { path: item.path }).catch(function() {});
  });
  row.style.cursor = 'pointer';
  return row;
}

function renderReceivedHeader(time, device, isLatest, index) {
  var hdr = document.createElement("div");
  hdr.className = "rcard-header" + (isLatest ? " rcard-header-latest" : "");
  hdr.style.animationDelay = (index * 30) + "ms";
  var label = time ? time.substring(0, 16) : "";
  if (device) label += " " + t("received_from") + " " + device;
  hdr.textContent = label;
  return hdr;
}

function setupReceived() {
  var receivedList = document.getElementById("received-list");
  if (!receivedList) { console.log("[Su!] received-list not found"); return; }

  window.refreshReceivedGlobal = async function refreshReceived() {
    try {
      var items = await invoke("get_received_files");
      if (!receivedList) return;
      if (items.length === 0) {
        receivedList.innerHTML = '';
        var empty = document.createElement("div");
        empty.className = "received-empty";
        empty.dataset.i18n = "no_received";
        empty.textContent = t("no_received");
        receivedList.appendChild(empty);
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

      var idx = 0;
      batchNums.forEach(function(bn) {
        var isLatest = bn === maxBatch;
        var files = batches[bn];
        var hdrTime = (files[0].time || "").substring(0, 16);
        var deviceLabel = files[0].device || "";
        receivedList.appendChild(renderReceivedHeader(hdrTime, deviceLabel, isLatest, idx++));
        files.forEach(function(item) { receivedList.appendChild(renderReceivedItem(item, isLatest, idx++)); });
      });
    } catch (e) { console.error("[Su!] refreshReceived error:", e); }
  };

  if (!receivedInitialized) {
    receivedInitialized = true;

    listen("file-received", function(event) {
      console.log("[Su!] file-received event:", event.payload);
      if (typeof window.refreshReceivedGlobal === "function") window.refreshReceivedGlobal();
    }).catch(function(e) {
      console.error("[Su!] listen file-received failed:", e);
    });

    listen("batch-complete", function(event) {
      console.log("[Su!] batch-complete event:", event.payload);
      var d = event.payload;
      if (d && localStorage.getItem("su-popup-enabled") !== "false") {
        showReceivedModal(d);
      }
      if (typeof window.refreshReceivedGlobal === "function") window.refreshReceivedGlobal();
    }).catch(function(e) {
      console.error("[Su!] listen batch-complete failed:", e);
    });
  }

  var receivedTab = document.querySelector('[data-tab="received"]');
  if (receivedTab && !receivedObserver) {
    receivedObserver = new MutationObserver(function() {
     if (receivedTab.classList.contains("active")) {
       if (typeof window.refreshReceivedGlobal === "function") window.refreshReceivedGlobal();
     }
   });
    receivedObserver.observe(receivedTab, { attributes: true, attributeFilter: ["class"] });
 }

  if (typeof window.refreshReceivedGlobal === "function") window.refreshReceivedGlobal();
}

export { setupReceived, showReceivedModal };
