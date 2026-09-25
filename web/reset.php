<?php
require_once __DIR__ . '/includes/layout.php';
require_once __DIR__ . '/includes/visitlog.php';
log_visit('/reset');
$step = isset($_POST['step']) ? $_POST['step'] : '1';
$msg = '';
$err = '';
$u = '';
if ($_SERVER['REQUEST_METHOD'] == 'POST') {
    if ($step == '1') {
        $in = isset($_POST['who']) ? $_POST['who'] : '';
        if ($in == 'admin') {
            $err = t('reset_admin_err');
        } else {
            $db = get_app_db();
            $q = 'SELECT * FROM users WHERE username = \'' . $in . '\' OR email = \'' . $in . '\'';
            $r = mysqli_query($db, $q);
            $row = mysqli_fetch_assoc($r);
            if ($row && $row['username'] == 'admin') {
                mysqli_close($db);
                $err = t('reset_admin_err');
            } else if ($row) {
                $code = sprintf('%05d', mt_rand(0, 99999));
                $now = date('Y-m-d H:i:s');
                $exp = date('Y-m-d H:i:s', time() + 900);
                $st = mysqli_query($db, 'INSERT INTO reset_codes (username, code, created_at, expires_at) VALUES (\'' . $row['username'] . '\', \'' . $code . '\', \'' . $now . '\', \'' . $exp . '\')');
                mysqli_close($db);
                $lf = ROOT_DIR . '/data/logs/app.log';
                file_put_contents($lf, 'reset code for ' . $row['username'] . ': ' . $code . ' at ' . $now . "\n", FILE_APPEND);
                $msg = str_replace('%s', $row['email'], t('reset_sent'));
                $step = '2';
            } else {
                mysqli_close($db);
                $err = t('reset_notfound');
            }
        }
    } else if ($step == '2') {
        $u = isset($_POST['username']) ? $_POST['username'] : '';
        $c = isset($_POST['code']) ? $_POST['code'] : '';
        $db = get_app_db();
        $q = 'SELECT * FROM reset_codes WHERE username = \'' . $u . '\' AND code = \'' . $c . '\'';
        $r = mysqli_query($db, $q);
        $row = mysqli_fetch_assoc($r);
        mysqli_close($db);
        if (!$row) {
            $err = t('reset_badcode');
        } else if ($row['used'] == 1) {
            $err = t('reset_used');
        } else if ($row['expires_at'] < date('Y-m-d H:i:s')) {
            $err = t('reset_expired');
        } else {
            $step = '3';
        }
    } else if ($step == '3') {
        $u = isset($_POST['username']) ? $_POST['username'] : '';
        $np = isset($_POST['newpass']) ? $_POST['newpass'] : '';
        $db = get_app_db();
        mysqli_query($db, 'UPDATE users SET password_md5 = \'' . md5($np) . '\' WHERE username = \'' . $u . '\'');
        mysqli_query($db, 'UPDATE reset_codes SET used = 1 WHERE username = \'' . $u . '\' AND used = 0');
        mysqli_close($db);
        $msg = t('reset_done');
        $step = 'done';
    }
}
layout_header(t('reset_title'));
echo '<h1>' . htmlspecialchars(t('reset_title')) . '</h1>';
if ($msg != '') echo '<div class="ok">' . htmlspecialchars($msg) . '</div>';
if ($err != '') echo '<div class="error">' . htmlspecialchars($err) . '</div>';
if ($step == '1') {
    echo '<p>' . htmlspecialchars(t('reset_step1_text')) . '</p>
<form method="post">
<input type="hidden" name="step" value="1">
<div><input type="text" name="who"></div>
<div><button type="submit">' . htmlspecialchars(t('reset_submit1')) . '</button></div>
</form>';
} else if ($step == '2') {
    echo '<p>' . htmlspecialchars(t('reset_step2_text')) . '</p>
<form method="post">
<input type="hidden" name="step" value="2">
<input type="hidden" name="username" value="' . htmlspecialchars($u) . '">
<div><input type="text" name="code" maxlength="5"></div>
<div><button type="submit">' . htmlspecialchars(t('reset_verify')) . '</button></div>
</form>';
} else if ($step == '3') {
    echo '<p>' . htmlspecialchars(t('reset_step3_text')) . '</p>
<form method="post">
<input type="hidden" name="step" value="3">
<input type="hidden" name="username" value="' . htmlspecialchars($u) . '">
<div><input type="password" name="newpass"></div>
<div><button type="submit">' . htmlspecialchars(t('reset_save')) . '</button></div>
</form>';
}
echo '<p><a href="/login.php">(' . htmlspecialchars(t('login')) . ')</a></p>';
layout_footer();
