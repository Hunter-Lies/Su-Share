// share.js - file sharing UI: drag-drop, QR modal, file cards
import { invoke, listen, dropZone, fileList, emptyHint, clearAllBtn, qrModal, qrModalImg, qrModalName, qrModalUrl, qrModalCopy, sendQrWrap, settingsIp } from './state.js';
import { fmtSize, toast, copyText, esc, getQrDataUrl } from './utils.js';
import { t } from './i18n/index.js';

var dragListenersReady = false;
var modalListenersReady = false;

function setupDragDrop() {
  var browseLink = document.getElementById("browse-link");
  if (browseLink) browseLink.addEventListener("click", async function(e) {
   e.stopPropagation();
   try { var paths = await invoke("pick_files"); if (paths && paths.length) shareNow(paths); } catch (e) { toast(t("toast_pick_failed")); }
 });
 if (dropZone) dropZone.addEventListener("click", async function() {
   try { var paths = await invoke("pick_files"); if (paths && paths.length) shareNow(paths); } catch (e) { toast(t("toast_pick_failed")); }
 });
  if (dragListenersReady) return;
  dragListenersReady = true;
 listen("tauri://drag-enter", function() { if (dropZone) dropZone.classList.add("drag-over"); });
 listen("tauri://drag-leave", function() { if (dropZone) dropZone.classList.remove("drag-over"); });
 listen("tauri://drag-drop", function(event) {
    if (dropZone) dropZone.classList.remove("drag-over");
    var paths = event.payload.paths || [];
    if (paths.length) shareNow(paths);
  });
}

function setupModal() {
  if (!qrModal || modalListenersReady) return;
  modalListenersReady = true;
 var _bg = qrModal.querySelector(".modal-bg"); if (_bg) _bg.addEventListener("click", closeModal);
  var _x = qrModal.querySelector(".modal-x"); if (_x) _x.addEventListener("click", closeModal);
  if (qrModalCopy) qrModalCopy.addEventListener("click", function() { copyText(qrModalUrl.textContent).finally(function(){closeModal()}); });
}

function closeModal() { qrModal.classList.add("hidden"); }

async function openQrModal(file) {
  qrModalImg.innerHTML = '<div style="color:var(--t2);padding:40px;font-size:13px">' + t("generating") + '</div>';
  qrModalName.textContent = file.name;
  qrModalUrl.textContent = file.url;
  qrModal.classList.remove("hidden");
  var qrDataUrl = await getQrDataUrl(file.url, 512);
  var hasMultiple = file.files && file.files.length > 1;
  if (hasMultiple) { file.name = file.files.length + ' ' + t('files_count'); }
  var modalCard = qrModal.querySelector(".modal-card");
  if (hasMultiple) {
    modalCard.classList.add("modal-multi");
    qrModalName.style.display = 'none';
    qrModalUrl.style.display = 'none';
    qrModalCopy.style.display = 'none';
    var leftHtml = '<div class="modal-panel"><div class="modal-flist">';
    for (var i = 0; i < file.files.length; i++) {
      leftHtml += '<div class="modal-fitem"><span class="modal-fn">' + esc(file.files[i].name) + '</span><span class="modal-fs">' + fmtSize(file.files[i].size) + '</span></div>';
    }
    leftHtml += '</div></div>';
    var rightHtml = '<div class="modal-panel modal-panel-right"><img class="modal-qr-single" src="' + qrDataUrl + '" alt="QR" />';
    rightHtml += '<h3 class="modal-right-name">' + esc(file.name) + '</h3>';
    rightHtml += '<code class="modal-right-url">' + esc(file.url) + '</code>';
    rightHtml += '<button class="modal-right-copy" id="modal-right-copy-btn">'+t('copy_link')+'</button>';
    rightHtml += '</div>';
    qrModalImg.innerHTML = leftHtml + rightHtml;
    setTimeout(function() {
      var btn = document.getElementById("modal-right-copy-btn");
      if (btn) btn.addEventListener("click", function(e) { e.stopPropagation(); copyText(file.url).finally(function(){closeModal()}); });
    }, 100);
  } else {
    modalCard.classList.remove("modal-multi");
    qrModalImg.innerHTML = '<img src="' + qrDataUrl + '" alt="QR" style="width:188px;height:188px;image-rendering:pixelated;border-radius:10px" />';
    qrModalName.style.display = '';
    qrModalUrl.style.display = '';
    qrModalCopy.style.display = '';
  }
}

function setupClearAll() {
  if (clearAllBtn) clearAllBtn.addEventListener("click", async function() {
    await invoke("clear_all_shares");
    if (fileList) fileList.innerHTML = "";
    updateEmpty();
    toast(t("toast_cleared"));
  });
}

function createSvg(html) {
  var wrap = document.createElement("span");
  wrap.innerHTML = html;
  return wrap.firstChild;
}

async function renderFileCard(file) {
  var c = document.createElement("div");
  c.className = "fcard";
  c.dataset.id = file.id;
  c.style.animationDelay = "0ms";
  var showName = (file.files && file.files.length > 1) ? (file.files.length + " " + t("files_count")) : file.name;

  var fqr = document.createElement("div");
  fqr.className = "fqr fqr-icon";
  fqr.innerHTML = '<i class="fa-solid fa-qrcode"></i>';

  var fi = document.createElement("div");
  fi.className = "fi";
  var fn = document.createElement("div");
  fn.className = "fn";
  fn.textContent = showName;
  fn.title = showName;
  var fs = document.createElement("div");
  fs.className = "fs";
  fs.textContent = fmtSize(file.size);
  var fu = document.createElement("div");
  fu.className = "fu";
  fu.title = t("copy_link");
  fu.textContent = file.url;
  fu.addEventListener("click", function(e) { e.stopPropagation(); copyText(file.url); });
  fi.appendChild(fn);
  fi.appendChild(fs);
  fi.appendChild(fu);

  var fa = document.createElement("div");
  fa.className = "fa";

  function makeBtn(svgHtml, cls, titleKey, onClick) {
    var b = document.createElement("button");
    if (cls) b.className = cls;
    b.title = t(titleKey);
    b.appendChild(createSvg(svgHtml));
    b.addEventListener("click", function(e) { e.stopPropagation(); onClick(); });
    return b;
  }

  var enlargeSvg = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/><path d="M11 8v6M8 11h6"/></svg>';
  var copySvg = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/></svg>';
  var stopSvg = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>';

  fa.appendChild(makeBtn(enlargeSvg, "", "view_qr", function() { openQrModal(file); }));
  fa.appendChild(makeBtn(copySvg, "", "copy_link", function() { copyText(file.url); }));
  fa.appendChild(makeBtn(stopSvg, "danger", "stop_share", function() {
    invoke("stop_share", { id: file.id }).then(function() { c.remove(); updateEmpty(); });
  }));

  c.appendChild(fqr);
  c.appendChild(fi);
  c.appendChild(fa);

  c.addEventListener("click", function(e) {
    if (e.target.closest(".fa button") || e.target.closest(".fu")) return;
    openQrModal(file);
  });

  var fl = fileList || document.getElementById("file-list");
  if (fl) fl.appendChild(c);
}

function updateEmpty() {
  if (emptyHint) emptyHint.style.display = (fileList && fileList.children.length === 0) ? "block" : "none";
}

async function shareNow(paths) {
  if (!paths || paths.length === 0) { toast(t("toast_no_files")); return; }
  try {
    var bundle = await invoke("share_files", { paths: paths });
    renderFileCard(bundle);
    openQrModal(bundle);
    updateEmpty();
  } catch (e) { toast(t("toast_share_failed") + ": " + e); }
}

async function loadServerInfo() {
  try {
    var info = await invoke("get_server_info");
    var addr = "http://" + info.lan_ip + ":" + info.port;
    if (settingsIp) settingsIp.textContent = info.lan_ip + ":" + info.port;
    var shareUrl = document.getElementById("share-url");
    if (shareUrl) {
      shareUrl.textContent = addr;
      shareUrl.onclick = function() {
        navigator.clipboard.writeText(addr).then(function() { toast(t("toast_copied")); });
      };
    }
  } catch (e) { if (settingsIp) settingsIp.textContent = t("not_ready"); }
}

async function loadSendQr() {
  try {
    var dataUrl = await invoke("get_send_qr", { size: 256 });
    if (sendQrWrap) sendQrWrap.innerHTML = '<img src="' + dataUrl + '" alt="QR" />';
  } catch (e) { if (sendQrWrap) sendQrWrap.innerHTML = '<span class="qrl">' + t("load_failed") + '</span>'; }
}

export { setupDragDrop, setupModal, closeModal, openQrModal, setupClearAll, renderFileCard, updateEmpty, shareNow, loadServerInfo, loadSendQr };
