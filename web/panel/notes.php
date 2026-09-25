<?php
require_once __DIR__ . '/../includes/layout.php';
require_once __DIR__ . '/../includes/visitlog.php';
log_visit('/panel/notes');
if (isset($_GET['action']) && $_GET['action'] == 'view' && isset($_GET['id'])) {
    $id = $_GET['id'];
    $db = get_app_db();
    $r = mysqli_query($db, 'SELECT * FROM notes WHERE id = ' . $id);
    $note = mysqli_fetch_assoc($r);
    mysqli_close($db);
    layout_header(t('notes_view'));
    if ($note) {
        echo '<h1>' . $note['title'] . '</h1>';
        echo '<div class="by">' . $note['created_at'] . '</div>';
        echo '<div class="body">' . $note['body'] . '</div>';
    } else {
        echo '<h1>' . htmlspecialchars(t('notes_not_found')) . '</h1>';
    }
    echo '<p><a href="/panel/notes.php">' . htmlspecialchars(t('back')) . '</a></p>';
    layout_footer();
    exit;
}
$me = require_login();
$done = '';
if ($_SERVER['REQUEST_METHOD'] == 'POST') {
    $title = isset($_POST['title']) ? $_POST['title'] : '';
    $body = isset($_POST['body']) ? $_POST['body'] : '';
    $db = get_app_db();
    $r = mysqli_query($db, 'SELECT MAX(id) as m FROM notes');
    $m = mysqli_fetch_assoc($r);
    $next = (int)$m['m'] + 1;
    mysqli_query($db, 'INSERT INTO notes (id, user_id, title, body, created_at) VALUES (' . $next . ', ' . (int)$me['id'] . ', \'' . $title . '\', \'' . $body . '\', \'' . date('Y-m-d H:i:s') . '\')');
    mysqli_close($db);
    $done = t('notes_saved');
}
layout_header(t('notes_view'));
echo '<h1>' . htmlspecialchars(t('notes_title')) . '</h1>';
if ($done != '') echo '<div class="ok">' . htmlspecialchars($done) . '</div>';
$db = get_app_db();
$r = mysqli_query($db, 'SELECT * FROM notes WHERE user_id = ' . (int)$me['id'] . ' ORDER BY id DESC');
while ($n = mysqli_fetch_assoc($r)) {
    echo '<div class="newsitem"><h3><a href="/panel/notes.php?action=view&id=' . $n['id'] . '">' . htmlspecialchars($n['title']) . '</a></h3>';
    echo '<div class="by">' . $n['created_at'] . '</div></div>';
}
mysqli_close($db);
echo '<h2>' . htmlspecialchars(t('notes_new')) . '</h2>
<form method="post">
<div><label>' . htmlspecialchars(t('label_title')) . '</label><input type="text" name="title" size="40"></div>
<div><label>' . htmlspecialchars(t('label_body')) . '</label><textarea name="body" rows="5" cols="40"></textarea></div>
<div><button type="submit">' . htmlspecialchars(t('save')) . '</button></div>
</form>
<p><a href="/panel/index.php">' . htmlspecialchars(t('back')) . '</a></p>';
layout_footer();
