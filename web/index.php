<?php
require_once __DIR__ . '/includes/layout.php';
require_once __DIR__ . '/includes/visitlog.php';
log_visit('/');
layout_header(t('home_title'));
echo '<img class="hero" src="/image.php?file=images/hero.svg" alt="">';
echo '<h1>' . htmlspecialchars(t('org')) . '</h1>';
echo '<p class="welcome">' . htmlspecialchars(t('home_welcome')) . '</p>';
echo '<section class="front-about">
<h2>' . htmlspecialchars(t('handling')) . '</h2>
<p>' . htmlspecialchars(t('handling_text')) . '</p>
<h2>' . htmlspecialchars(t('opettider')) . '</h2>
<p>' . htmlspecialchars(t('opettider_text')) . '</p>
<h2>' . htmlspecialchars(t('fullmaktige')) . '</h2>
<p>' . htmlspecialchars(t('fullmaktige_text')) . '</p>
</section>';
echo '<h2>' . htmlspecialchars(t('news_title')) . '</h2>';
$db = get_app_db();
$r = mysqli_query($db, 'SELECT n.id, n.title, n.body, n.image, n.created_at, u.username FROM news n JOIN users u ON u.id = n.author_id ORDER BY n.created_at DESC');
while ($n = mysqli_fetch_assoc($r)) {
    $excerpt = substr($n['body'], 0, 180);
    echo '<div class="newsitem">';
    echo '<h3><a href="/news.php?id=' . $n['id'] . '">' . $n['title'] . '</a></h3>';
    if (!empty($n['image'])) echo '<img class="newsimg" src="/' . htmlspecialchars($n['image']) . '" alt="">';
    echo '<div class="by">' . htmlspecialchars(t('news_by')) . ' ' . htmlspecialchars($n['username']) . ' | ' . $n['created_at'] . '</div>';
    echo '<div class="excerpt">' . $excerpt . '</div>';
    echo '<a class="more" href="/news.php?id=' . $n['id'] . '">' . htmlspecialchars(t('news_read_more')) . '</a>';
    echo '</div>';
}
mysqli_close($db);
layout_footer();
