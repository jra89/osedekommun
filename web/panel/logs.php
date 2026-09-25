<?php
require_once __DIR__ . '/../includes/layout.php';
$me = require_login();
log_visit('/panel/logs');
layout_header(t('logs_title'));
echo '<h1>' . htmlspecialchars(t('logs_title')) . '</h1>';
$f = visit_log_file();
if (file_exists($f)) {
    $lines = file($f, FILE_IGNORE_NEW_LINES);
    $n = count($lines);
    $start = $n > 200 ? $n - 200 : 0;
    echo '<div class="by">' . str_replace(array('%count%', '%file%'), array($n - $start, basename($f)), t('logs_recent')) . '</div>';
    echo '<pre class="logview">';
    for ($i = $start; $i < $n; $i++) {
        echo htmlspecialchars($lines[$i]) . "\n";
    }
    echo '</pre>';
} else {
    echo '<p>' . htmlspecialchars(t('logs_empty')) . '</p>';
}
echo '<p><a href="/panel/index.php">' . htmlspecialchars(t('back')) . '</a></p>';
layout_footer();
