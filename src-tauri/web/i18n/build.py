# Auto-generated: concatenate mobile i18n files into i18n.js
import os

base = os.path.dirname(__file__)
langs = ["zh-CN", "zh-TW", "en", "ja", "ko"]

# Read per-language files
blocks = {}
for code in langs:
    with open(os.path.join(base, code + ".js"), "r", encoding="utf-8") as f:
        lines = f.readlines()
    # Skip comment line, take everything else
    inner = "".join(lines[1:]).strip()
    blocks[code] = inner

js = r"""// Su! i18n — mobile pages (non-module)
var SU = SU || {};
SU.I18N = {
"""

for code in langs:
    js += f'"{code}":{{{blocks[code]}}},\n'

js += """};
SU.t=function(k,l){l=l||window.__SU_LANG||"zh-CN";var d=SU.I18N[l]||SU.I18N["zh-CN"];return d[k]||SU.I18N["zh-CN"][k]||k;};
SU.applyI18n=function(l){l=l||window.__SU_LANG||"zh-CN";document.documentElement.lang=l;var els=document.querySelectorAll("[data-i18n]");for(var i=0;i<els.length;i++){var k=els[i].getAttribute("data-i18n");var d=SU.I18N[l]||SU.I18N["zh-CN"];if(d[k])els[i].textContent=d[k];}};
SU.detectLang=function(sl){var p=new URLSearchParams(location.search).get("lang");if(p&&SU.I18N[p])return p;var mode=window.__SU_LANG_MODE||"auto";if(mode==="server"&&sl&&SU.I18N[sl])return sl;var b=navigator.language||navigator.browserLanguage||"";var map={"zh-CN":"zh-CN","zh-TW":"zh-TW","zh":"zh-CN",ja:"ja",ko:"ko",en:"en"};if(SU.I18N[b])return b;var base=b.split("-")[0];if(map[base]&&SU.I18N[map[base]])return map[base];if(sl&&SU.I18N[sl])return sl;return"zh-CN";};
"""

with open(os.path.join(os.path.dirname(base), "i18n.js"), "w", encoding="utf-8", newline="") as f:
    f.write(js)
print("Mobile i18n.js rebuilt from per-language files")
