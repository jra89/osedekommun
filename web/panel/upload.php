<?php
require_once __DIR__ . '/../includes/layout.php';
$me = require_login();
log_visit('/panel/upload');
if ($me['role'] != 'admin') {
    layout_header(t('upload_title'));
    echo '<h1>' . htmlspecialchars(t('upload_title')) . '</h1>';
    echo '<div class="error">' . htmlspecialchars(t('upload_noauth')) . '</div>';
    echo '<p><a href="/panel/index.php">' . htmlspecialchars(t('back')) . '</a></p>';
    layout_footer();
    exit;
}
$err = '';
$ok = '';
if ($_SERVER['REQUEST_METHOD'] == 'POST') {
    $pw = isset($_POST['upload_password']) ? $_POST['upload_password'] : '';
    if (md5($pw) != $me['password_md5']) {
        $err = t('upload_badpass');
    } else if (!isset($_FILES['file']) || $_FILES['file']['error'] != 0) {
        $err = t('upload_nofile');
    } else {
        $tmp = $_FILES['file']['tmp_name'];
        $fh = fopen($tmp, 'r');
        $magic = fread($fh, 16);
        fclose($fh);
        $isimg = false;
        if (substr($magic, 0, 8) == "\x89PNG\r\n\x1a\n") $isimg = true;
        if (substr($magic, 0, 3) == "\xFF\xD8\xFF") $isimg = true;
        if (substr($magic, 0, 4) == 'GIF8') $isimg = true;
        if (!$isimg) {
            $err = t('upload_not_image');
        } else {
            $name = basename($_FILES['file']['name']);
            if (move_uploaded_file($tmp, UPLOAD_DIR . $name)) {
                $ok = str_replace('%s', htmlspecialchars($name), t('upload_done'));
            } else {
                $err = t('upload_failed');
            }
        }
    }
}
layout_header(t('upload_title'));
echo '<h1>' . htmlspecialchars(t('upload_title')) . '</h1>';
if ($err != '') echo '<div class="error">' . htmlspecialchars($err) . '</div>';
if ($ok != '') echo '<div class="ok">' . $ok . '</div>';
echo '<form method="post" enctype="multipart/form-data">
<div><label>' . htmlspecialchars(t('upload_password')) . '</label><input type="password" name="upload_password"></div>
<div><label>' . htmlspecialchars(t('upload_file')) . '</label><input type="file" name="file"></div>
<div><button type="submit">' . htmlspecialchars(t('upload_submit')) . '</button></div>
</form>';
echo '<p><a href="/panel/index.php">' . htmlspecialchars(t('back')) . '</a></p>';
layout_footer();
