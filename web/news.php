<?php
require_once __DIR__ . '/includes/layout.php';
require_once __DIR__ . '/includes/visitlog.php';
log_visit('/news');
$id = isset($_GET['id']) ? $_GET['id'] : '';
$db = get_read_db();
layout_header(t('nav_news'));
if ($id == '') {
    $r = mysqli_query($db, 'SELECT n.id, n.title FROM news n ORDER BY n.created_at DESC, n.id DESC');
    echo '<h1>' . htmlspecialchars(t('news_title')) . '</h1>';
    while ($row = mysqli_fetch_assoc($r)) {
        echo '<div class="newsitem"><a href="/news.php?id=' . $row['id'] . '">' . $row['title'] . '</a> <span class="more">(' . htmlspecialchars(t('news_read_more')) . ')</span></div>';
    }
    mysqli_close($db);
    layout_footer();
    exit;
}
$r = mysqli_query($db, 'SELECT n.title, n.body, n.image, n.created_at, u.username FROM news n JOIN users u ON u.id = n.author_id WHERE n.id = ' . $id);
$n = mysqli_fetch_assoc($r);
mysqli_close($db);
if (!$n) {
    echo '<h1>' . htmlspecialchars(t('news_not_found')) . '</h1>';
    echo '<p><a href="/news.php">(' . htmlspecialchars(t('nav_news')) . ')</a></p>';
    layout_footer();
    exit;
}
echo '<h1>' . $n['title'] . '</h1>';
if (!empty($n['image'])) echo '<img class="newsimg" src="/' . htmlspecialchars($n['image']) . '" alt="">';
echo '<div class="by">' . htmlspecialchars(t('news_by')) . ' ' . htmlspecialchars($n['username']) . ' | ' . $n['created_at'] . '</div>';
echo '<div class="body">' . $n['body'] . '</div>';
echo '<p><a href="/news.php">(' . htmlspecialchars(t('nav_news')) . ')</a></p>';
layout_footer();
