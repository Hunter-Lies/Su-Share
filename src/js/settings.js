// settings.js
import { invoke, initDomRefs, downloadsDir, pickDirBtn, autostartToggle, shortcutBtn, settingsIp } from './state.js';
import { t } from './i18n/index.js';
import { applyTheme, applyStyle } from './theme.js';
import { toast } from './utils.js';

var globalDropdownCloseReady = false;

function setupSettings() {
  setupLangDropdown();
  setupMobileLangToggle();
  setupSubPageNavigation();
  if (!globalDropdownCloseReady) {
    globalDropdownCloseReady = true;
    setupGlobalDropdownClose();
  }
}

function setupStyleDropdown() {
  var styleDdBtn = document.getElementById('style-dd-btn');
  if (styleDdBtn) {
    styleDdBtn.addEventListener('click', function(e) {
      e.stopPropagation();
      var sd = document.getElementById('style-dd');
      if (sd) sd.classList.toggle('open');
    });
    var styleDdMenu = document.getElementById('style-dd-menu');
    if (styleDdMenu) {
      styleDdMenu.querySelectorAll('.set-dd-item').forEach(function(item) {
        item.addEventListener('click', function(e) {
          e.stopPropagation();
          localStorage.setItem('su-style', this.dataset.value);
          applyStyle();
          var sd = document.getElementById('style-dd');
          if (sd) sd.classList.remove('open');
          toast(t('toast_style_switched').replace('{v}', this.dataset.value === 'classic' ? t('classic') : t('glass')));
        });
      });
    }
  }
}

function setupThemeDropdown() {
  var themeDdBtn = document.getElementById('theme-dd-btn');
  if (themeDdBtn) {
    themeDdBtn.addEventListener('click', function(e) {
      e.stopPropagation();
      var td = document.getElementById('theme-dd');
      if (td) td.classList.toggle('open');
    });
    var themeDdMenu = document.getElementById('theme-dd-menu');
    if (themeDdMenu) {
      themeDdMenu.querySelectorAll('.set-dd-item').forEach(function(item) {
        item.addEventListener('click', function(e) {
          e.stopPropagation();
          if (this.dataset.value === 'auto') { localStorage.removeItem('su-theme'); }
          else { localStorage.setItem('su-theme', this.dataset.value); }
          applyTheme();
          var td = document.getElementById('theme-dd');
          if (td) td.classList.remove('open');
          toast(t('toast_switched'));
        });
      });
    }
  }
}

function setupSubPageNavigation() {
  var settingsMain = document.getElementById('settings-main');
  var subLock = false;

  if (settingsMain) {
    window.loadSettingsSub = async function(subName) {
      if (subLock) return;
      subLock = true;

      var oldSub = document.querySelector('.settings-sub-dynamic');
      // Start exit animation on old sub-page (if any)
      var exitPromise = Promise.resolve();
      if (oldSub) {
        oldSub.classList.add('hidden');
        exitPromise = new Promise(function(r) { setTimeout(r, 260); });
      }

      // Load new HTML in parallel with exit animation
      var htmlPromise = invoke("read_page", { name: "settings-" + subName });

      // Wait for exit animation to finish
      await exitPromise;
      if (oldSub && oldSub.parentNode) oldSub.remove();

      // Wait for HTML to be ready (should already be done by now)
      var html = await htmlPromise;;

      try {
        var html = await invoke('read_page', { name: 'settings-' + subName });
        var temp = document.createElement('div');
        temp.innerHTML = html;
        var subEl = temp.firstElementChild;
        if (!subEl) { settingsMain.classList.remove('slide-out'); subLock = false; return; }
       subEl.classList.add('settings-sub-dynamic');
        document.getElementById('main-content').appendChild(subEl);
       subEl.offsetHeight;
       subEl.classList.remove('hidden');

        initDomRefs();
        var lang = localStorage.getItem('su-lang') || 'zh-CN';
        (await import('./i18n/index.js')).applyI18n(lang);

        var backBtn = subEl.querySelector('.set-sub-back');
        if (backBtn) backBtn.onclick = function() { closeSubPage(subEl); };

        setupSubPage(subName);
      } catch(e) { console.error('[Su!] loadSettingsSub:', e); settingsMain.classList.remove('slide-out'); }
      subLock = false;
    };

    window.closeSubPage = function(subEl) {
      if (!subEl) return;
      subEl.classList.add('hidden');
      setTimeout(function() {
        if (subEl.parentNode) subEl.remove();
      }, 260);
      settingsMain.classList.remove('slide-out');
    };
  }

  document.querySelectorAll('.set-nav-item').forEach(function(item) {
    item.addEventListener('click', function() {
      var sub = this.dataset.sub;
      if (sub) window.loadSettingsSub(sub);
    });
  });
}

function setupGlobalDropdownClose() {
  if (globalDropdownCloseReady) return;
  globalDropdownCloseReady = true;
  document.addEventListener('click', function(e) {
    if (!e.target.closest('.set-dd')) {
      document.querySelectorAll('.set-dd.open').forEach(function(d) {
        d.classList.remove('open');
      });
    }
  });
}

function setupSubPage(subName) {
  if (subName === 'appearance') { setupStyleDropdown(); setupThemeDropdown(); applyTheme(); applyStyle(); }
  if (subName === 'notification') { setupSoundToggle(); setupSoundDropdown(); setupPopupToggle(); }
  if (subName === 'software') { setupAutostart(); setupTrayToggle(); setupShortcutBtn(); setupContextMenuBtn(); setupResetDefaultsBtn(); }
  if (subName === 'receive') { setupDownloadsDir(); setupClearReceivedToggle(); }
  if (subName === 'security') { setupAutoReceiveToggle(); }
}

function setupAutostart() {
  var toggle = document.getElementById('autostart-toggle');
  if (!toggle) return;
  invoke('get_autostart').then(function(v) {
    toggle.checked = v;
    localStorage.setItem('su-autostart', v ? 'true' : 'false');
  }).catch(function() {
    toggle.checked = localStorage.getItem('su-autostart') === 'true';
  });
  toggle.addEventListener('change', async function() {
    localStorage.setItem('su-autostart', this.checked);
    try {
      await invoke('set_autostart', { enable: this.checked });
      toast(t(this.checked ? 'toast_autostart_on' : 'toast_autostart_off'));
    } catch (e) {
      toast(t('toast_failed') + ': ' + e);
    }
  });
}

function setupShortcutBtn() {
  var btn = document.getElementById('shortcut-btn');
  if (!btn) return;
  btn.addEventListener('click', async function() {
    try { await invoke('create_shortcut'); toast(t('toast_shortcut_created')); }
    catch (e) { toast(t('toast_create_failed') + ': ' + e); }
  });
}

function setupContextMenuBtn() {
  var btn = document.getElementById('context-menu-btn');
  if (!btn) return;
  function updateText() {
    var registered = btn.dataset.registered === 'true';
    btn.textContent = t(registered ? 'ctx_registered' : 'ctx_register');
  }
  updateText();
  btn.addEventListener('click', async function() {
    var registered = btn.dataset.registered === 'true';
    try {
      if (!registered) {
        await invoke('register_context_menu');
        btn.dataset.registered = 'true';
        toast(t('toast_context_registered'));
      } else {
        await invoke('unregister_context_menu');
        btn.dataset.registered = 'false';
        toast(t('toast_context_unregistered'));
      }
      updateText();
    } catch (e) { toast(t(registered ? 'toast_unregister_failed' : 'toast_register_failed') + ': ' + e); }
  });
}

function setupResetDefaultsBtn() {
  var btn = document.getElementById('reset-defaults-btn');
  if (!btn) return;
  btn.addEventListener('click', async function() {
    if (!confirm(t('confirm_reset'))) return;
    localStorage.clear();
    sessionStorage.clear();
    try { await invoke('reset_defaults'); } catch(e) {}
    location.reload();
  });
}

function setupDownloadsDir() {
  var dirEl = document.getElementById('downloads-dir');
  var pickBtn = document.getElementById('pick-dir-btn');
  if (dirEl) {
    invoke('get_download_dir').then(function(path) {
      dirEl.textContent = path || localStorage.getItem('su-downloads-dir') || 'C:/Users/.../Downloads/Su';
    }).catch(function() {
      dirEl.textContent = localStorage.getItem('su-downloads-dir') || 'C:/Users/.../Downloads/Su';
    });
  }
  if (pickBtn) {
    pickBtn.addEventListener('click', async function() {
      try {
        var paths = await invoke('pick_folder');
        if (paths && paths.length) {
          if (dirEl) dirEl.textContent = paths[0];
          localStorage.setItem('su-downloads-dir', paths[0]);
          invoke('set_download_dir', { path: paths[0] });
          toast(t('toast_dir_updated'));
        }
      } catch (e) { toast(t('toast_failed')); }
    });
  }
}

function setupSoundToggle() {
  var st = document.getElementById('sound-toggle');
  if (!st) return;
  st.checked = localStorage.getItem('su-sound-enabled') !== 'false';
  st.addEventListener('change', function() {
    localStorage.setItem('su-sound-enabled', this.checked);
    invoke('set_sound_settings', {
      enabled: this.checked,
      name: localStorage.getItem('su-sound') || '投递'
    });
    toast(t(this.checked ? 'toast_sound_on' : 'toast_sound_off'));
  });
}

function setupSoundDropdown() {
  var sdB = document.getElementById('sound-dd-btn');
  var sdM = document.getElementById('sound-dd-menu');
  if (!sdB || !sdM) return;
  var sdL = document.getElementById('sound-dd-label');
  if (sdL) sdL.textContent = localStorage.getItem('su-sound') || '投递';
  sdB.addEventListener('click', function(e) {
    e.stopPropagation();
    var sd = document.getElementById('sound-dd');
    if (sd) sd.classList.toggle('open');
  });
  sdM.querySelectorAll('.set-dd-item').forEach(function(item) {
    item.addEventListener('click', function(e) {
      e.stopPropagation();
      var val = this.dataset.value;
      if (sdL) sdL.textContent = val;
      localStorage.setItem('su-sound', val);
      var st = document.getElementById('sound-toggle');
      invoke('set_sound_settings', { enabled: st ? st.checked : true, name: val });
      var sd = document.getElementById('sound-dd');
      if (sd) sd.classList.remove('open');
      toast(t('toast_sound_changed') + ' ' + val);
    });
  });
}

function setupPopupToggle() {
  var pt = document.getElementById('popup-toggle');
  if (!pt) return;
  pt.checked = localStorage.getItem('su-popup-enabled') !== 'false';
  pt.addEventListener('change', function() {
    localStorage.setItem('su-popup-enabled', this.checked);
    toast(t(this.checked ? 'toast_popup_on' : 'toast_popup_off'));
  });
}

function setupTrayToggle() {
  var tt = document.getElementById('tray-toggle');
  if (!tt) return;
  tt.checked = localStorage.getItem('su-tray-mode') === 'true';
  invoke('set_tray_mode', { enabled: tt.checked }).catch(function() {});
  tt.addEventListener('change', function() {
    localStorage.setItem('su-tray-mode', this.checked);
    invoke('set_tray_mode', { enabled: this.checked });
    toast(t(this.checked ? 'toast_tray_on' : 'toast_tray_off'));
  });
}

function setupClearReceivedToggle() {
  var crt = document.getElementById('clear-received-toggle');
  if (!crt) return;
  crt.checked = localStorage.getItem('su-clear-on-close') === 'true';
  invoke('set_clear_on_close', { enabled: crt.checked }).catch(function() {});
  crt.addEventListener('change', function() {
    localStorage.setItem('su-clear-on-close', this.checked);
    invoke('set_clear_on_close', { enabled: this.checked });
    toast(t(this.checked ? 'toast_clear_on' : 'toast_clear_off'));
  });
}

function setupAutoReceiveToggle() {
  var toggle = document.getElementById('auto-receive-toggle');
  if (!toggle) return;
  if (toggle._suListener) return;
  toggle._suListener = true;
  invoke('get_auto_receive').then(function(v) { toggle.checked = v; }).catch(function() {});
  toggle.addEventListener('change', function() {
    invoke('set_auto_receive', { enable: toggle.checked })
      .then(function() {
        toast(t(toggle.checked ? 'toast_auto_receive_on' : 'toast_auto_receive_off'));
      })
      .catch(function(e) {
        toast(t('toast_failed') + ': ' + e);
        toggle.checked = !toggle.checked;
      });
  });
}


function setupMobileLangToggle() {
  var toggle = document.getElementById('mobile-lang-mode-toggle');
  if (!toggle) return;
  toggle.checked = localStorage.getItem('su-mobile-lang-mode') !== 'auto';
  invoke('set_mobile_lang_mode', { mode: toggle.checked ? 'server' : 'auto' }).catch(function() {});
  toggle.addEventListener('change', function() {
    var mode = this.checked ? 'server' : 'auto';
    localStorage.setItem('su-mobile-lang-mode', mode);
    invoke('set_mobile_lang_mode', { mode: mode });
    toast(mode === 'server' ? t('toast_mobile_lang_server') : t('toast_mobile_lang_device'));
  });
}

function setupLangDropdown() {
  var dd = document.getElementById('lang-dd');
  if (!dd) return;
  var btn = document.getElementById('lang-dd-btn');
  var label = document.getElementById('lang-dd-label');
  var menu = document.getElementById('lang-dd-menu');
  if (!btn || !label || !menu) return;
  var cur = localStorage.getItem('su-lang') || 'zh-CN';
  var names = {
    'zh-CN': '简体中文',
    'zh-TW': '繁體中文',
    'en': 'English',
    'ja': '日本語',
    'ko': '한국어'
  };
  label.textContent = names[cur] || cur;
  btn.addEventListener('click', function(e) {
    e.stopPropagation();
    dd.classList.toggle('open');
  });
  menu.querySelectorAll('.set-dd-item').forEach(function(it) {
    it.addEventListener('click', function(e) {
      e.stopPropagation();
      var v = this.dataset.value;
      localStorage.setItem('su-lang', v);
      label.textContent = names[v] || v;
      dd.classList.remove('open');
      invoke('set_lang', { lang: v }).then(function() {
        location.reload();
      }).catch(function() {
        location.reload();
      });
    });
  });
}

function setupCtxMenuBtn() {
  var btn = document.getElementById("ctx-menu-btn");
  if (!btn) return;
  if (localStorage.getItem("su-ctx-registered") === "true") {
    btn.textContent = t("ctx_unregister");
  }
  btn.addEventListener("click", async function() {
    if (btn.textContent === t("ctx_register")) {
      try {
        await invoke("register_context_menu");
        localStorage.setItem("su-ctx-registered", "true");
        btn.textContent = t("ctx_unregister");
        toast(t("toast_context_registered"));
      } catch (e) { toast(t("toast_register_failed") + ": " + e); }
    } else {
      try {
        await invoke("unregister_context_menu");
        localStorage.removeItem("su-ctx-registered");
        btn.textContent = t("ctx_register");
        toast(t("toast_context_unregistered"));
      } catch (e) { toast(t("toast_unregister_failed") + ": " + e); }
    }
  });
}

export { setupSettings };
