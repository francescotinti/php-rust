<?php
$n = (int)$argv[1]; $h = '';
for ($i = 0; $i < $n; $i++) { $h = password_hash("secret$i", PASSWORD_BCRYPT, ['cost' => (int)$argv[2]]); }
for ($i = 0; $i < $n; $i++) { password_verify("secret", $h); }
echo strlen($h), "\n";
