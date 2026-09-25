<?php
$f = isset($_GET['file']) ? $_GET['file'] : '';
$ext = strtolower(pathinfo($f, PATHINFO_EXTENSION));
$types = array('png' => 'image/png', 'jpg' => 'image/jpeg', 'jpeg' => 'image/jpeg', 'gif' => 'image/gif', 'ico' => 'image/x-icon', 'svg' => 'image/svg+xml');
if (isset($types[$ext])) header('Content-Type: ' . $types[$ext]);
readfile($f);
