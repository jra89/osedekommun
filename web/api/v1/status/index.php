<?php
// /api/v1/status - statusmeddelande-service.
// Returnerar det systemstatusmeddelande som visas i statusbanneren på
// hemsidans sidor (t.ex. underhållsmeddelanden). Frontend skickar det
// meddelande som visas via GET-parametret msg.
$msg = isset($_GET['msg']) ? $_GET['msg'] : '';
if ($msg === '') {
    $msg = 'Systemunderhåll kommer ske mellan 22:00 och 02:00 på lördag';
}
$accept = isset($_SERVER['HTTP_ACCEPT']) ? $_SERVER['HTTP_ACCEPT'] : '*/*';
if (strpos($accept, 'application/json') !== false && strpos($accept, 'text/html') === false) {
    header('Content-Type: application/json; charset=utf-8');
    echo json_encode(array('status' => 'ok', 'message' => $msg));
} else {
    header('Content-Type: text/html; charset=utf-8');
    echo $msg;
}
