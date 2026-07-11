// settings.js — settings page: toggles, dropdowns, context menu
import { invoke, downloadsDir, pickDirBtn, themeDd, themeDdLabel, styleDd, styleDdLabel, autostartToggle, shortcutBtn, settingsIp, soundToggle, soundDd, soundDdBtn, soundDdLabel, soundDdMenu, popupToggle, trayToggle, clearReceivedToggle } from './state.js';
import { t } from "./i18n/index.js";
import { applyTheme, applyStyle } from './theme.js';
import { toast } from './utils.js';

function setupSettings() {
  setupLangDropdown();
  setupAutoReceiveToggle();
  setupMobileLangToggle();
  invoke("get_server_info").then(function(info) {
    if (settingsIp) settingsIp.textContent = info.lan_ip + ":" + info.port;
    downloadsDir.textContent = localStorage.getItem("su-downloads-dir") || "C:\\Users\\...\\Downloads\\Su";
    autostartToggle.checked = localStorage.getItem("su-autostart") === "true";
  });

  pickDirBtn.addEventListener("click", async function() {
    try {
      var paths = await invoke("pick_folder");
      if (paths && paths.length) {
        downloadsDir.textContent = paths[0];
        localStorage.setItem("su-downloads-dir", paths[0]);
        invoke("set_download_dir", { path: paths[0] });
        toast(t("toast_dir_updated"));
      }
    } catch (e) { toast(t("toast_failed")); }
  });

  // Style dropdown
  document.getElementById("style-dd-btn").addEventListener("click", function(e) { e.stopPropagation(); styleDd.classList.toggle("open"); });
  document.querySelectorAll("#style-dd-menu .set-dd-item").forEach(function(item) {
    item.addEventListener("click", function(e) {
      e.stopPropagation();
      localStorage.setItem("su-style", this.dataset.value);
      applyStyle();
      styleDd.classList.remove("open");
      toast(t("toast_style_switched").replace("{v}", this.dataset.value === "classic" ? t("classic") : t("glass")));
    });
  });

  // Theme dropdown
  document.getElementById("theme-dd-btn").addEventListener("click", function(e) { e.stopPropagation(); themeDd.classList.toggle("open"); });
  document.querySelectorAll("#theme-dd-menu .set-dd-item").forEach(function(item) {
    item.addEventListener("click", function(e) {
      e.stopPropagation();
      if (this.dataset.value === "auto") { localStorage.removeItem("su-theme"); }
      else { localStorage.setItem("su-theme", this.dataset.value); }
      applyTheme();
      themeDd.classList.remove("open");
      var msgs = { auto: "已设为跟随系统", dark: "已切换到深色主题", light: "已切换到浅色主题" };
      toast(t("toast_switched"));
    });
  });

  // Settings sub-page navigation
  var settingsMain = document.getElementById("settings-main");
  var settingsSubs = document.querySelectorAll(".settings-sub");
  var navItems = document.querySelectorAll(".set-nav-item");
  navItems.forEach(function(item) {
    item.addEventListener("click", function() {
      var sub = this.dataset.sub;
      if (sub) {
        var target = document.getElementById("settings-sub-" + sub);
        if (target) {
          settingsMain.classList.add("hidden");
          settingsSubs.forEach(function(s) { s.classList.add("hidden"); });
          target.classList.remove("hidden");
        }
      }
    });
  });
  // Back buttons
  document.querySelectorAll(".set-sub-back").forEach(function(btn) {
    btn.addEventListener("click", function() {
      settingsSubs.forEach(function(s) { s.classList.add("hidden"); });
      settingsMain.classList.remove("hidden");
    });
  });

  // Global click: close dropdowns when clicking outside
  document.addEventListener("click", function(e) {
    if (!e.target.closest(".set-dd")) {
      themeDd.classList.remove("open");
      styleDd.classList.remove("open");
      if (soundDd) soundDd.classList.remove("open");
    }
  });

  autostartToggle.addEventListener("change", function() {
    localStorage.setItem("su-autostart", this.checked);
    toast(t(this.checked ? "toast_autostart_on" : "toast_autostart_off"));
  });

  shortcutBtn.addEventListener("click", async function() {
    try { await invoke("create_shortcut"); toast(t("toast_shortcut_created")); }
    catch (e) { toast(t("toast_create_failed") + ": " + e); }
  });

  // Reset defaults
  var resetBtn = document.getElementById("reset-defaults-btn");
  if (resetBtn) resetBtn.addEventListener("click", async function() {
    if (!confirm("确定要还原所有默认设置吗？此操作不可恢复！")) return;
    localStorage.clear();
    sessionStorage.clear();
    try { await invoke("reset_defaults"); } catch(e) {}
    location.reload();
  });

  // Context menu register/unregister
  var ctxMenuBtn = document.getElementById("context-menu-btn");
  if (ctxMenuBtn) {
    ctxMenuBtn.textContent = t("ctx_register");
    ctxMenuBtn.addEventListener("click", async function() {
      if (ctxMenuBtn.textContent === "注册") {
        try { await invoke("register_context_menu"); ctxMenuBtn.textContent = t("ctx_registered"); toast(t("toast_context_registered")); }
        catch (e) { toast(t("toast_register_failed") + ": " + e); }
      } else {
        try { await invoke("unregister_context_menu"); ctxMenuBtn.textContent = t("ctx_register"); toast(t("toast_context_unregistered")); }
        catch (e) { toast(t("toast_unregister_failed") + ": " + e); }
      }
    });
  }

  if (typeof window.refreshReceivedGlobal === "function") window.refreshReceivedGlobal();

  setupSoundToggle();
  setupSoundDropdown();
  setupPopupToggle();
  setupTrayToggle();
  setupClearReceivedToggle();
}

function setupSoundToggle() {
  if (!soundToggle) return;
  soundToggle.checked = localStorage.getItem("su-sound-enabled") !== "false";
  soundToggle.addEventListener("change", function() {
    localStorage.setItem("su-sound-enabled", this.checked);
    var sn = soundDdLabel ? soundDdLabel.textContent : "投递";
    invoke("set_sound_settings", { enabled: this.checked, name: sn });
    toast(t(this.checked ? "toast_sound_on" : "toast_sound_off"));
  });
}

function setupSoundDropdown() {
  if (!soundDdBtn || !soundDdMenu) return;
  var currentSound = localStorage.getItem("su-sound") || "投递";
  if (soundDdLabel) soundDdLabel.textContent = currentSound;
  soundDdBtn.addEventListener("click", function(e) { e.stopPropagation(); soundDd.classList.toggle("open"); });
  soundDdMenu.querySelectorAll(".set-dd-item").forEach(function(item) {
    item.addEventListener("click", function(e) {
      e.stopPropagation();
      var val = this.dataset.value;
      soundDdLabel.textContent = val;
      localStorage.setItem("su-sound", val);
      var se = soundToggle ? soundToggle.checked : true;
      invoke("set_sound_settings", { enabled: se, name: val });
      soundDd.classList.remove("open");
      toast(t("toast_sound_changed") + " " + val);
    });
  });
}

function setupPopupToggle() {
  if (!popupToggle) return;
  popupToggle.checked = localStorage.getItem("su-popup-enabled") !== "false";
  popupToggle.addEventListener("change", function() {
    localStorage.setItem("su-popup-enabled", this.checked);
    toast(t(this.checked ? "toast_popup_on" : "toast_popup_off"));
  });
}

function setupTrayToggle() {
  if (!trayToggle) return;
  trayToggle.checked = localStorage.getItem("su-tray-mode") === "true";
  invoke("set_tray_mode", { enabled: trayToggle.checked }).catch(function() {});
  trayToggle.addEventListener("change", function() {
    localStorage.setItem("su-tray-mode", this.checked);
    invoke("set_tray_mode", { enabled: this.checked });
    toast(t(this.checked ? "toast_tray_on" : "toast_tray_off"));
  });
}

function setupClearReceivedToggle() {
  if (!clearReceivedToggle) return;
  clearReceivedToggle.checked = localStorage.getItem("su-clear-on-close") === "true";
  invoke("set_clear_on_close", { enabled: clearReceivedToggle.checked }).catch(function() {});
  clearReceivedToggle.addEventListener("change", function() {
    localStorage.setItem("su-clear-on-close", this.checked);
    invoke("set_clear_on_close", { enabled: this.checked });
    toast(t(this.checked ? "toast_clear_on" : "toast_clear_off"));
  });
}



function setupAutoReceiveToggle() {
  var toggle = document.getElementById("auto-receive-toggle");
  if (!toggle) return;
  invoke("get_auto_receive").then(function(v) { toggle.checked = v; }).catch(function() {});
  toggle.addEventListener("change", function() {
    invoke("set_auto_receive", { enable: toggle.checked });
  });
}

function setupMobileLangToggle() {
  var toggle = document.getElementById("mobile-lang-mode-toggle");
  if (!toggle) return;
  toggle.checked = localStorage.getItem("su-mobile-lang-mode") !== "auto";
  invoke("set_mobile_lang_mode", { mode: toggle.checked ? "server" : "auto" }).catch(function() {});
  toggle.addEventListener("change", function() {
    var mode = this.checked ? "server" : "auto";
    localStorage.setItem("su-mobile-lang-mode", mode);
    invoke("set_mobile_lang_mode", { mode: mode }); toast(mode==="server"?t("toast_mobile_lang_server"):t("toast_mobile_lang_device"));
  });
}
function setupLangDropdown(){var dd=document.getElementById("lang-dd");if(!dd)return;var btn=document.getElementById("lang-dd-btn");var label=document.getElementById("lang-dd-label");var menu=document.getElementById("lang-dd-menu");var cur=localStorage.getItem("su-lang")||"zh-CN";var names={"zh-CN":"简体中文","zh-TW":"繁體中文","en":"English","ja":"日本語","ko":"한국어"};label.textContent=names[cur]||cur;btn.addEventListener("click",function(e){e.stopPropagation();dd.classList.toggle("open")});menu.querySelectorAll(".set-dd-item").forEach(function(it){it.addEventListener("click",function(e){e.stopPropagation();var v=this.dataset.value;localStorage.setItem("su-lang",v);label.textContent=names[v]||v;dd.classList.remove("open");invoke("set_lang",{lang:v}).then(function(){location.reload()}).catch(function(){location.reload()})})})}

export { setupSettings };
