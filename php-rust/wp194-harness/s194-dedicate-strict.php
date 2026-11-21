<?php
declare(strict_types=1);
function strict_probe() {
  try { echo strlen(5); } catch (TypeError $e) { echo "ST:", $e->getMessage(), "\n"; }
  try { echo strlen(null); } catch (TypeError $e) { echo "ST:", $e->getMessage(), "\n"; }
  echo strlen("abc"), "|", count([1, 2]), "\n";
}
