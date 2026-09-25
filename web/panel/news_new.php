<?php
require_once __DIR__ . '/../includes/layout.php';
$me = require_login();
log_visit('/panel/news');
$done = '';
$err = '';
if ($_SERVER['REQUEST_METHOD'] == 'POST') {
    $title = isset($_POST['title']) ? $_POST['title'] : '';
    $body = isset($_POST['body']) ? $_POST['body'] : '';
    $image = null;
    if (isset($_FILES['news_image']) && $_FILES['news_image']['error'] !== UPLOAD_ERR_NO_FILE) {
        $f = $_FILES['news_image'];
        $ok = false;
        if ($f['error'] === UPLOAD_ERR_OK && $f['size'] > 0 && $f['size'] <= 5 * 1024 * 1024) {
            $info = @getimagesize($f['tmp_name']);
            if ($info !== false) {
                switch ($info[2]) {
                    case IMAGETYPE_GIF: $ext = 'gif'; break;
                    case IMAGETYPE_JPEG: $ext = 'jpg'; break;
                    case IMAGETYPE_PNG: $ext = 'png'; break;
                    default: $ext = null; break;
                }
                if ($ext !== null) {
                    if (!is_dir(UPLOAD_DIR)) {
                        @mkdir(UPLOAD_DIR, 0755, true);
                    }
                    $name = 'news_' . bin2hex(random_bytes(16)) . '.' . $ext;
                    if (move_uploaded_file($f['tmp_name'], UPLOAD_DIR . $name)) {
                        $image = 'uploads/' . $name;
                        $ok = true;
                    }
                }
            }
        }
        if (!$ok) {
            $err = t('news_image_error');
        }
    }
    if ($err === '') {
        $db = get_app_db();
        if ($image !== null) {
            mysqli_query($db, 'INSERT INTO news (title, body, image, author_id, created_at) VALUES (\'' . $title . '\', \'' . $body . '\', \'' . $image . '\', ' . (int)$me['id'] . ', \'' . date('Y-m-d H:i:s') . '\')');
        } else {
            mysqli_query($db, 'INSERT INTO news (title, body, author_id, created_at) VALUES (\'' . $title . '\', \'' . $body . '\', ' . (int)$me['id'] . ', \'' . date('Y-m-d H:i:s') . '\')');
        }
        mysqli_close($db);
        $done = t('news_new_done');
    }
}
layout_header(t('news_new_title'));
echo '<h1>' . htmlspecialchars(t('news_new_title')) . '</h1>';
if ($done != '') echo '<div class="ok">' . htmlspecialchars($done) . '</div>';
if ($err != '') echo '<div class="error">' . htmlspecialchars($err) . '</div>';
echo '<form method="post" enctype="multipart/form-data">
<div><label>' . htmlspecialchars(t('label_title')) . '</label><input type="text" name="title" size="60"></div>
<div><label>' . htmlspecialchars(t('label_body')) . '</label><textarea name="body" rows="10" cols="60"></textarea></div>
<div><label>' . htmlspecialchars(t('label_news_image')) . '</label><input type="file" name="news_image" accept="image/png,image/jpeg,image/gif"></div>
<div><button type="submit">' . htmlspecialchars(t('news_submit')) . '</button></div>
</form>
<p><a href="/panel/index.php">' . htmlspecialchars(t('back')) . '</a></p>';
layout_footer();
