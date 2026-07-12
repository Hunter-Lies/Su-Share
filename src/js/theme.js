// theme.js — theme, style, titlebar, collapse, tabs
import { invoke, themeDd, themeDdLabel, styleDd, styleDdLabel } from "./state.js";
import { t } from "./i18n/index.js";

function applyTheme() {
  var saved = localStorage.getItem("su-theme");
  var theme = saved || (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light");
  var label = theme === "dark" ? t("dark") : t("light");
  if (!saved) label = t("auto");
  if (themeDdLabel) themeDdLabel.textContent = label;
  var menu = document.querySelectorAll("#theme-dd-menu .set-dd-item");
  menu.forEach(function(item) {
    item.classList.toggle("active", item.dataset.value === (saved || "auto"));
  });
  document.documentElement.classList.toggle("dark", theme === "dark");
}

function applyStyle() {
  var style = localStorage.getItem("su-style") || "classic";
  if (styleDdLabel) styleDdLabel.textContent = style === "classic" ? t("classic") : t("glass");
  var menu = document.querySelectorAll("#style-dd-menu .set-dd-item");
  menu.forEach(function(item) {
    item.classList.toggle("active", item.dataset.value === style);
  });
  document.documentElement.classList.toggle("glass", style === "glass");
}

function setupTitlebar() {
  var minimizeBtn = document.getElementById("tb-minimize");
  var maximizeBtn = document.getElementById("tb-maximize");
  var closeBtn = document.getElementById("tb-close");
  if (minimizeBtn) minimizeBtn.addEventListener("click", function() { invoke("minimize_window"); });
  if (maximizeBtn) maximizeBtn.addEventListener("click", function() { invoke("toggle_maximize"); });
  if (closeBtn) closeBtn.addEventListener("click", async function() {
    var tm = localStorage.getItem("su-tray-mode") === "true";
    var msg = tm ? t("confirm_close_tray") : t("confirm_close");
    if (!confirm(msg)) return;
    if (!tm && localStorage.getItem("su-clear-on-close") === "true") {
      await invoke("clear_received").catch(function() {});
    }
    invoke("close_window");
  });
}

function setupCollapse() {
  var sidebar = document.getElementById("sidebar");
  if (!sidebar) return;
  if (localStorage.getItem("su-collapsed") === "true") sidebar.classList.add("collapsed");
}

function setupTabs() {
  document.querySelectorAll(".s-nav-item").forEach(function(item) {
    item.addEventListener("click", function() {
      var tab = this.dataset.tab;
      if (!tab) return;
      document.querySelectorAll(".s-nav-item").forEach(function(n) { n.classList.remove("active"); });
      this.classList.add("active");
      if (typeof window.loadTab === "function") {
        window.loadTab(tab);
      }
    });
  });
}

export { applyTheme, applyStyle, setupTitlebar, setupCollapse, setupTabs };
