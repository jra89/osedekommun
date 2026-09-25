<?php
require_once __DIR__ . '/../includes/layout.php';
$me = require_login();
log_visit('/panel');
layout_header(t('panel_title'));
echo '<h1>' . htmlspecialchars(t('panel_title')) . '</h1>';
echo '<div class="panelmenu">
<a href="/panel/news_new.php">' . htmlspecialchars(t('panel_new_news')) . '</a>
<a href="/panel/notes.php">' . htmlspecialchars(t('panel_notes')) . '</a>
<a href="/panel/messages.php">' . htmlspecialchars(t('panel_messages')) . '</a>
<a href="/panel/logs.php">' . htmlspecialchars(t('panel_logs')) . '</a>
<a href="/panel/upload.php">' . htmlspecialchars(t('panel_upload')) . '</a>
<a href="/panel/uplist.php">' . htmlspecialchars(t('panel_uplist')) . '</a>
</div>';
layout_footer();
