<?php
// Fixture restapi-shaped della sentinella estesa (A-PE-101-3): header JSON
// + corpo json_encode + eco del parametro (payload INTERLEAVED via query).
header('Content-Type: application/json; charset=UTF-8');
header('X-Sentinel: s100');
$route = $_GET['rest_route'] ?? '/none';
static $seen = 0; $seen++;
echo json_encode([
  'route' => $route,
  'seen' => $seen,
  'data' => ['a' => 1, 'b' => [2, 3], 'c' => str_repeat('x', 32)],
], JSON_UNESCAPED_SLASHES), "\n";
