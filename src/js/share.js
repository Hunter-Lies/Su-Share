// share.js — file sharing UI: drag-drop, QR modal, file cards
import { invoke, listen, dropZone, fileList, emptyHint, clearAllBtn, qrModal, qrModalImg, qrModalName, qrModalUrl, qrModalCopy, sendQrWrap, settingsIp } from './state.js';
import { fmtSize, toast, copyText, esc, getQrDataUrl } from './utils.js';
import { t } from './i18n.js';

function setupDragDrop() {
  document.getElementById("browse-link").addEventListener("click", async function(e) {
    e.stopPropagation();
    try { var paths = await invoke("pick_files"); if (paths && paths.length) shareNow(paths); } catch (e) { toast(t("toast_pick_failed")); }
  });
  dropZone.addEventListener("click", async function() {
    try { var paths = await invoke("pick_files"); if (paths && paths.length) shareNow(paths); } catch (e) { toast(t("toast_pick_failed")); }
  });
  listen("tauri://drag-enter", function() { dropZone.classList.add("drag-over"); });
  listen("tauri://drag-leave", function() { dropZone.classList.remove("drag-over"); });
  listen("tauri://drag-drop", function(event) {
    dropZone.classList.remove("drag-over");
    var paths = event.payload.paths || [];
    if (paths.length) shareNow(paths);
  });
}

function setupModal() {
  qrModal.querySelector(".modal-bg").addEventListener("click", closeModal);
  qrModal.querySelector(".modal-x").addEventListener("click", closeModal);
  qrModalCopy.addEventListener("click", function() { copyText(qrModalUrl.textContent).finally(function(){closeModal()}); });
}

function closeModal() { qrModal.classList.add("hidden"); }

async function openQrModal(file) {
  qrModalImg.innerHTML = '<div style="color:var(--t2);padding:40px;font-size:13px">生成中...</div>';
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
  clearAllBtn.addEventListener("click", async function() {
    await invoke("clear_all_shares");
    fileList.innerHTML = "";
    updateEmpty();
    toast(t("toast_cleared"));
  });
}

async function renderFileCard(file) {
  var c = document.createElement("div");
  c.className = "fcard";
  c.dataset.id = file.id;
  var showName = (file.files && file.files.length > 1) ? (file.files.length + " " + t("files_count")) : file.name;
  c.innerHTML = '<div class="fqr fqr-icon"><i class="fa-solid fa-qrcode"></i></div><div class="fi"><div class="fn">' + esc(showName) + '</div><div class="fs">' + fmtSize(file.size) + '</div><div class="fu" title="点击复制">' + esc(file.url) + '</div></div><div class="fa"><button title="放大"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/><path d="M11 8v6M8 11h6"/></svg></button><button title="复制"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"/></svg></button><button class="danger" title="停止"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg></button></div>';
  c.addEventListener("click", function(e) {
    if (e.target.closest(".fa button") || e.target.closest(".fu")) return;
    openQrModal(file);
  });
  c.querySelector(".fu").addEventListener("click", function(e) { e.stopPropagation(); copyText(file.url); });
  c.querySelector(".fa button:nth-child(1)").addEventListener("click", function(e) { e.stopPropagation(); openQrModal(file); });
  c.querySelector(".fa button:nth-child(2)").addEventListener("click", function(e) { e.stopPropagation(); copyText(file.url); });
  c.querySelector(".fa button.danger").addEventListener("click", function(e) {
    e.stopPropagation();
    invoke("stop_share", { id: file.id }).then(function() { c.remove(); updateEmpty(); });
  });
  fileList.appendChild(c);
}

function updateEmpty() {
  emptyHint.style.display = fileList.children.length === 0 ? "block" : "none";
}

async function shareNow(paths) {
  if (!paths || paths.length === 0) { toast(t("toast_no_files")); return; }
  try {
    var bundle = await invoke("share_files", { paths: paths });
    await renderFileCard(bundle);
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
  } catch (e) { if (settingsIp) settingsIp.textContent = "未就绪"; }
}

async function loadSendQr() {
  try {
    var dataUrl = await invoke("get_send_qr", { size: 256 });
    sendQrWrap.innerHTML = '<img src="' + dataUrl + '" alt="QR" />';
  } catch (e) { sendQrWrap.innerHTML = '<span class="qrl">加载失败</span>'; }
}

export { setupDragDrop, setupModal, closeModal, openQrModal, setupClearAll, renderFileCard, updateEmpty, shareNow, loadServerInfo, loadSendQr };
