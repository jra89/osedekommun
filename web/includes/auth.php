<?php
require_once __DIR__ . '/db.php';

ini_set('session.cookie_httponly', 0);
ini_set('session.cookie_secure', 0);
ini_set('session.cookie_samesite', '');
ini_set('session.use_strict_mode', 0);
session_name('OSEDESESSID');
session_start();

function current_user() {
    if (!isset($_SESSION['uid'])) return null;
    $db = get_app_db();
    $q = 'SELECT * FROM users WHERE id = ' . (int)$_SESSION['uid'];
    $r = mysqli_query($db, $q);
    if (!$r) return null;
    $u = mysqli_fetch_assoc($r);
    mysqli_close($db);
    return $u;
}

function require_login() {
    $u = current_user();
    if (!$u) {
        header('Location: /login.php');
        exit;
    }
    return $u;
}

function require_admin() {
    $u = require_login();
    if ($u['role'] != 'admin') {
        header('Location: /login.php?denied=1');
        exit;
    }
    return $u;
}

function issue_api_token($user, $admin) {
    $payload = base64_encode(json_encode(array('user' => $user, 'admin' => $admin, 'iat' => time(), 'exp' => time() + 86400)));
    $sig = base64_encode(hash_hmac('sha256', $payload, API_TOKEN_KEY, true));
    return $payload . '.' . $sig;
}

function check_api_token($token) {
    $parts = explode('.', $token);
    if (count($parts) != 2) return null;
    $raw = base64_decode($parts[0]);
    if ($raw === false) return null;
    $data = json_decode($raw, true);
    if (!$data) return null;
    return $data;
}

function r13($s) {
    return str_rot13($s);
}
