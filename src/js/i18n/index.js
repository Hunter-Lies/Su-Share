// Su! i18n — index / loader
import zhCN from "./zh-CN.js";
import zhTW from "./zh-TW.js";
import en from "./en.js";
import ja from "./ja.js";
import ko from "./ko.js";

const I18N = {
  "zh-CN": zhCN,
  "zh-TW": zhTW,
  en,
  ja,
  ko,
};

const LANGS = [
  { code: "zh-CN", name: "简体中文" },
  { code: "zh-TW", name: "繁體中文" },
  { code: "en", name: "English" },
  { code: "ja", name: "日本語" },
  { code: "ko", name: "한국어" },
];

function detectLang(serverLang) {
  const p = new URLSearchParams(location.search);
  const urlLang = p.get("lang");
  if (urlLang && I18N[urlLang]) return urlLang;
  if (serverLang && I18N[serverLang]) return serverLang;
  const b = (navigator.language || "zh-CN").split("-");
  if (I18N[b.join("-")]) return b.join("-");
  const map = { zh: "zh-CN", ja: "ja", ko: "ko", en: "en" };
  if (map[b[0]]) return map[b[0]];
  return "zh-CN";
}

function t(key, lang) {
  lang = lang || window.__su_lang || "zh-CN";
  const d = I18N[lang] || I18N["zh-CN"];
  return d[key] || I18N["zh-CN"][key] || key;
}

function applyI18n(lang) {
  lang = lang || window.__su_lang || "zh-CN";
  document.documentElement.lang = lang;
  const els = document.querySelectorAll("[data-i18n]");
  for (let i = 0; i < els.length; i++) {
    const key = els[i].getAttribute("data-i18n");
    if (I18N[lang] && I18N[lang][key]) {
      els[i].textContent = I18N[lang][key];
    }
  }
  const titles = document.querySelectorAll("[data-i18n-title]");
  for (let i = 0; i < titles.length; i++) {
    const key = titles[i].getAttribute("data-i18n-title");
    if (I18N[lang] && I18N[lang][key]) {
      titles[i].setAttribute("title", I18N[lang][key]);
    }
  }
  const phs = document.querySelectorAll("[data-i18n-placeholder]");
  for (let i = 0; i < phs.length; i++) {
    const key = phs[i].getAttribute("data-i18n-placeholder");
    if (I18N[lang] && I18N[lang][key]) {
      phs[i].placeholder = I18N[lang][key];
    }
  }
}

export { I18N, LANGS, detectLang, t, applyI18n };
