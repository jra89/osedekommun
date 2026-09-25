<?php
require_once __DIR__ . '/i18n.php';
require_once __DIR__ . '/auth.php';
require_once __DIR__ . '/visitlog.php';

function layout_header($title) {
    $u = current_user();
    $me = t('org');
    echo '<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>' . htmlspecialchars($title) . ' - ' . htmlspecialchars($me) . '</title>
<link rel="stylesheet" href="/css/style.css">
<link rel="icon" type="image/svg+xml" href="/image.php?file=images/logo.svg">
</head>
<body>
<div class="topbar">
<img class="orglogo" src="/image.php?file=images/logo.svg" alt="">
<div class="org">' . htmlspecialchars($me) . '</div>
<div class="lang"><a href="' . htmlspecialchars(lang_url()) . '"><img class="flag" src="/image.php?file=images/flag_' . (get_lang() == 'sv' ? 'gb' : 'se') . '.svg" alt=""> ' . htmlspecialchars(t('lang_switch')) . '</a></div>
</div>
 <div class="statusbanner" id="statusbanner" data-message="' . htmlspecialchars(t('status_default'), ENT_QUOTES) . '"></div>
  <script>
  (function () {
      var b = document.getElementById("statusbanner");
      if (!b) return;
      var m = new URLSearchParams(window.location.search).get("status");
      if (!m) m = b.getAttribute("data-message");
      if (!m) return;
      fetch("/api/v1/status/?msg=" + encodeURIComponent(m), { headers: { "Accept": "text/html" } })
          .then(function (r) { return r.text(); })
          .then(function (t) { b.innerHTML = t; })
          .catch(function () {});
  })();
  </script>
 <div class="tagline">' . htmlspecialchars(t('tagline')) . '</div>
<div class="menu">
<a href="/" class="mbtn">' . htmlspecialchars(t('nav_start')) . '</a>
<a href="/news.php" class="mbtn">' . htmlspecialchars(t('nav_news')) . '</a>
<a href="/contact.php" class="mbtn">' . htmlspecialchars(t('nav_contact')) . '</a>
<a href="' . ($u ? '/panel/index.php' : '/login.php') . '" class="mbtn">' . htmlspecialchars(t('nav_panel')) . '</a>
' . ($u ? '<a href="/logout.php" class="mbtn">' . htmlspecialchars(t('logout')) . '</a>' : '') . '
</div>
<div class="content">
';
}

function layout_footer() {
    echo '</div>
<div class="footer">
<div class="footer-org">' . htmlspecialchars(t('footer_org')) . '</div>
<div class="footer-address"><img class="ficon" src="/image.php?file=images/pin.svg" alt=""> ' . htmlspecialchars(t('footer_address')) . '<br>' . htmlspecialchars(t('footer_orgnr')) . '</div>
<div class="footer-follow">' . htmlspecialchars(t('footer_follow')) . ': <a href="https://facebook.com/osedekommun">' . htmlspecialchars(t('footer_facebook')) . '</a> / <a href="https://instagram.com/osedekommun">' . htmlspecialchars(t('footer_instagram')) . '</a> / <a href="https://youtube.com/@osedekommun">' . htmlspecialchars(t('footer_youtube')) . '</a></div>
<div>' . htmlspecialchars(t('footer2')) . '</div>
</div>
</body>
</html>
';
}
