<?php
// S-189 gate-loopback fixture: L SEQUENTIAL blocking loopbacks to pong.php on
// the same server (curl, not file_get_contents: the http:// stream wrapper has
// no timeout, see PHPR_DIVERGENCES §3.39). ms = the curl call only.
$L = (int)($_GET['L'] ?? 1);
$port = (int)($_GET['port'] ?? 0);
for ($i = 1; $i <= $L; $i++) {
    $ch = curl_init("http://127.0.0.1:$port/pong.php");
    curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
    curl_setopt($ch, CURLOPT_TIMEOUT, 3);
    curl_setopt($ch, CURLOPT_CONNECTTIMEOUT, 3);
    $t0 = microtime(true);
    $body = curl_exec($ch);
    $ms = (int) round((microtime(true) - $t0) * 1000);
    $e = curl_errno($ch);
    unset($ch); // curl_close() is deprecated since 8.5
    if ($e === 0 && $body === 'pong') {
        echo "i=$i OK ms=$ms\n";
    } else {
        echo "i=$i ERR errno=$e ms=$ms\n";
    }
}
