// main.js — entry point for Su!
import { applyI18n } from './i18n/index.js';
import { invoke, initDomRefs } from './state.js';
import { applyTheme, applyStyle, setupTitlebar, setupCollapse, setupTabs } from './theme.js';
import { setupDragDrop, setupModal, setupClearAll, loadServerInfo, loadSendQr, updateEmpty } from './share.js';
import { setupReceived } from './received.js';
import { setupSettings } from './settings.js';

window.addEventListener("DOMContentLoaded", function () {
  initDomRefs();
  (function(){var l=localStorage.getItem("su-lang")||"zh-CN";applyI18n(l);invoke("set_lang",{lang:l}).catch(function(){})})();
  applyTheme();
  applyStyle();
  setupTitlebar();
  setupCollapse();
  setupTabs();
  setupDragDrop();
  setupModal();
  setupClearAll();
  setupReceived();
  setupSettings();
  loadServerInfo();
  loadSendQr();
  loadSendQr();
  (function() {
    var sn = localStorage.getItem("su-sound") || "投递";
    var se = localStorage.getItem("su-sound-enabled") !== "false";
    invoke("set_sound_settings", { enabled: se, name: sn }).catch(function() {});
  })();
  updateEmpty();
});
