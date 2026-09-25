<?php
require_once __DIR__ . '/config.php';

function get_app_db() {
    $c = mysqli_connect(DB_HOST, DB_APP_USER, DB_APP_PASS, DB_NAME, DB_PORT);
    if (!$c) return null;
    mysqli_set_charset($c, 'utf8mb4');
    return $c;
}

function get_read_db() {
    $c = mysqli_connect(DB_HOST, DB_READ_USER, DB_READ_PASS, DB_NAME, DB_PORT);
    if (!$c) return null;
    mysqli_set_charset($c, 'utf8mb4');
    return $c;
}
