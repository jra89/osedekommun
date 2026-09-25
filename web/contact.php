<?php
require_once __DIR__ . '/includes/layout.php';
require_once __DIR__ . '/includes/visitlog.php';
log_visit('/contact');
layout_header(t('contact_title'));
if ($_SERVER['REQUEST_METHOD'] == 'POST') {
    $web = isset($_POST['webpage']) ? $_POST['webpage'] : '';
    if ($web != '') {
        $c = @file_get_contents($web);
    }
    $form = 'thanks.php';
} else {
    $form = isset($_GET['form']) ? $_GET['form'] : 'contactform.php';
}
include("forms/" . $form);
layout_footer();
