<?php
require_once __DIR__ . '/../includes/layout.php';
$me = require_login();
log_visit('/panel/messages');
$done = '';
if (isset($_GET['read'])) {
    $id = (int)$_GET['read'];
    $db = get_app_db();
    $r = mysqli_query($db, 'SELECT m.*, u.username AS fromname FROM messages m JOIN users u ON u.id = m.from_id WHERE m.id = ' . $id . ' AND m.to_id = ' . (int)$me['id']);
    $m = mysqli_fetch_assoc($r);
    mysqli_close($db);
    layout_header(t('msg_detail'));
    if ($m) {
        mysqli_connect(DB_HOST, DB_APP_USER, DB_APP_PASS, DB_NAME, DB_PORT);
        $db2 = get_app_db();
        mysqli_query($db2, 'UPDATE messages SET is_read = 1 WHERE id = ' . $id);
        mysqli_close($db2);
        echo '<h1>' . $m['subject'] . '</h1>';
        echo '<div class="by">' . htmlspecialchars(t('msg_from')) . ' ' . htmlspecialchars($m['fromname']) . ' | ' . $m['created_at'] . '</div>';
        if ($m['preview'] != null && $m['preview'] != '') {
            echo '<div class="by">' . htmlspecialchars(t('msg_preview')) . ' ' . htmlspecialchars($m['preview']) . '</div>';
        }
        echo '<div class="body">' . htmlspecialchars($m['body']) . '</div>';
    } else {
        echo '<h1>' . htmlspecialchars(t('msg_not_found')) . '</h1>';
    }
    echo '<p><a href="/panel/messages.php">' . htmlspecialchars(t('back')) . '</a></p>';
    layout_footer();
    exit;
}
if (isset($_GET['del'])) {
    $id = (int)$_GET['del'];
    $db = get_app_db();
    mysqli_query($db, 'DELETE FROM messages WHERE id = ' . $id . ' AND to_id = ' . (int)$me['id']);
    mysqli_close($db);
}
if ($_SERVER['REQUEST_METHOD'] == 'POST') {
    $to = isset($_POST['to']) ? $_POST['to'] : '';
    $subject = isset($_POST['subject']) ? $_POST['subject'] : '';
    $body = isset($_POST['body']) ? $_POST['body'] : '';
    $link = isset($_POST['link']) ? $_POST['link'] : '';
    $preview = null;
    if ($link != '') {
        $c = @file_get_contents($link);
        if ($c !== false) $preview = substr($c, 0, 200);
    }
    $db = get_app_db();
    $r = mysqli_query($db, 'SELECT id FROM users WHERE username = \'' . $to . '\'');
    $t = mysqli_fetch_assoc($r);
    if ($t) {
        mysqli_query($db, 'INSERT INTO messages (from_id, to_id, subject, body, preview, created_at) VALUES (' . (int)$me['id'] . ', ' . (int)$t['id'] . ', \'' . $subject . '\', \'' . $body . '\', \'' . $preview . '\', \'' . date('Y-m-d H:i:s') . '\')');
        $done = t('msg_sent');
    } else {
        $done = t('msg_recipient_not_found');
    }
    mysqli_close($db);
}
layout_header(t('msg_list'));
echo '<h1>' . htmlspecialchars(t('msg_inbox')) . '</h1>';
if ($done != '') echo '<div class="ok">' . htmlspecialchars($done) . '</div>';
$db = get_app_db();
$r = mysqli_query($db, 'SELECT m.*, u.username AS fromname FROM messages m JOIN users u ON u.id = m.from_id WHERE m.to_id = ' . (int)$me['id'] . ' ORDER BY m.created_at DESC');
while ($m = mysqli_fetch_assoc($r)) {
    $unread = $m['is_read'] == 0 ? ' <b>(' . htmlspecialchars(t('msg_unread')) . ')</b>' : '';
    echo '<div class="newsitem"><h3><a href="/panel/messages.php?read=' . $m['id'] . '">' . htmlspecialchars($m['subject']) . '</a>' . $unread . '</h3>';
    echo '<div class="by">' . htmlspecialchars(t('msg_from')) . ' ' . htmlspecialchars($m['fromname']) . ' | ' . $m['created_at'] . '</div>';
    echo '<div class="excerpt">' . htmlspecialchars(substr($m['body'], 0, 120)) . '</div>';
    echo '<a href="/panel/messages.php?del=' . $m['id'] . '">' . htmlspecialchars(t('msg_delete')) . '</a></div>';
}
mysqli_close($db);
echo '<h2>' . htmlspecialchars(t('msg_new')) . '</h2>
<form method="post">
<div><label>' . htmlspecialchars(t('msg_to')) . '</label><input type="text" name="to"></div>
<div><label>' . htmlspecialchars(t('msg_subject')) . '</label><input type="text" name="subject" size="50"></div>
<div><label>' . htmlspecialchars(t('msg_message')) . '</label><textarea name="body" rows="5" cols="50"></textarea></div>
<div><label>' . htmlspecialchars(t('msg_link')) . '</label><input type="text" name="link" size="50"></div>
<div><button type="submit">' . htmlspecialchars(t('msg_send')) . '</button></div>
</form>
<p><a href="/panel/index.php">' . htmlspecialchars(t('back')) . '</a></p>';
layout_footer();
