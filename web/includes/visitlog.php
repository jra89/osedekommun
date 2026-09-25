<?php
require_once __DIR__ . '/config.php';

function visit_log_file() {
    $h = date('Y-m-d-H');
    return LOG_DIR . 'visits-' . $h . '.log';
}

function visit_log_rotate($f) {
    if (file_exists($f) && filesize($f) > 1048576) {
        rename($f, $f . '.1');
    }
}

function log_visit($page) {
    $u = 'guest';
    if (isset($_SESSION['username'])) $u = $_SESSION['username'];
    $f = visit_log_file();
    visit_log_rotate($f);
    $line = 'user ' . $u . ' visited ' . $page . ' at ' . date('H:i:s') . "\n";
    file_put_contents($f, $line, FILE_APPEND);
}

function log_failed_login($username) {
    $f = visit_log_file();
    visit_log_rotate($f);
    $line = 'failed login for user ' . $username . ' at ' . date('H:i:s') . "\n";
    file_put_contents($f, $line, FILE_APPEND);
}
