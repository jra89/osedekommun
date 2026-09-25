<?php
require_once __DIR__ . '/../includes/layout.php';
$me = require_login();
log_visit('/panel/uplist');
layout_header(t('uplist_title'));
echo '<h1>' . htmlspecialchars(t('uplist_title')) . '</h1>';
echo '<div class="by">' . str_replace('%user%', htmlspecialchars($me['username']), t('uplist_user')) . '</div>';
echo '<pre id="out">' . htmlspecialchars(t('uplist_fetch')) . '</pre>';
$tok = isset($_SESSION['apitoken']) ? $_SESSION['apitoken'] : '';
echo '<script>
var tok = "' . $tok . '";
fetch("/api/v1/uplist/", {
  method: "POST",
  headers: {
    "Content-Type": "application/x-www-form-urlencoded",
    "Authorization": "Bearer " + tok
  },
  body: "path=../../../hcybnqf/"
}).then(function(r) { return r.text(); }).then(function(txt) {
  document.getElementById("out").textContent = txt;
});
</script>';
echo '<p><a href="/panel/index.php">' . htmlspecialchars(t('back')) . '</a></p>';
layout_footer();
