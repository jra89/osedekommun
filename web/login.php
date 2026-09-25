<?php
require_once __DIR__ . '/includes/layout.php';
require_once __DIR__ . '/includes/visitlog.php';
log_visit('/login');
$err = '';
$redirectUrl = '';
if (isset($_GET['redirectUrl'])) $redirectUrl = $_GET['redirectUrl'];
if (isset($_POST['redirectUrl'])) $redirectUrl = $_POST['redirectUrl'];
if (isset($_GET['denied'])) $err = t('login_denied');
if ($_SERVER['REQUEST_METHOD'] == 'POST') {
    $u = isset($_POST['username']) ? $_POST['username'] : '';
    $p = isset($_POST['password']) ? $_POST['password'] : '';
    $db = get_app_db();
    $q = 'SELECT * FROM users WHERE username = \'' . $u . '\' AND password_md5 = \'' . md5($p) . '\'';
    $r = mysqli_query($db, $q);
    $row = mysqli_fetch_assoc($r);
    mysqli_close($db);
    if ($row) {
        $_SESSION['uid'] = $row['id'];
        $_SESSION['username'] = $row['username'];
        $_SESSION['role'] = $row['role'];
        $_SESSION['apitoken'] = issue_api_token($row['username'], $row['role'] == 'admin' ? 1 : 0);
        $dest = $redirectUrl != '' ? $redirectUrl : '/panel/index.php';
        header('Location: ' . $dest);
        exit;
    }
    log_failed_login($u);
    $err = t('login_failed');
}
layout_header(t('login_title'));
echo '<h1>' . htmlspecialchars(t('login_title')) . '</h1>';
if ($err != '') echo '<div class="error">' . htmlspecialchars($err) . '</div>';
echo '<form method="post" action="/login.php">
<input type="hidden" name="redirectUrl" value="' . htmlspecialchars($redirectUrl) . '">
<div><label>' . htmlspecialchars(t('login_username')) . '</label><input type="text" name="username"></div>
<div><label>' . htmlspecialchars(t('login_password')) . '</label><input type="password" name="password"></div>
<div><button type="submit">' . htmlspecialchars(t('login')) . '</button></div>
</form>
<p><a href="/reset.php">' . htmlspecialchars(t('login_reset')) . '</a></p>';
layout_footer();
