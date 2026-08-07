<?php // held-out 3 (S-111): hot-loop estratto per forma da WordPress — CONGELATO
$wp_filter = [];
function hf_add(&$t, $tag, $cb, $prio) { $t[$tag][$prio][] = $cb; }
function hf_apply($t, $tag, $value) {
  if (!isset($t[$tag])) return $value;
  foreach ($t[$tag] as $prio => $cbs) {
    foreach ($cbs as $cb) { $value = $cb($value); }
  }
  return $value;
}
function hf_sanitize_key($key) {
  return preg_replace('/[^a-z0-9_\-]/', '', strtolower($key));
}
function hf_esc_attr($text) {
  return str_replace(["&", "<", ">", "\"", "'"],
                     ["&amp;", "&lt;", "&gt;", "&quot;", "&#039;"], $text);
}
hf_add($wp_filter, 'the_title', function($v){ return trim($v); }, 10);
hf_add($wp_filter, 'the_title', function($v){ return ucfirst($v); }, 11);
hf_add($wp_filter, 'the_title', function($v){ return $v . ' - Site'; }, 12);
ksort($wp_filter['the_title']);
$n = 0; $len = 0;
for($i=0;$i<1500000;$i++){
  $t = hf_apply($wp_filter, 'the_title', "  hello world $i <b>&amp;</b>  ");
  $t = hf_esc_attr($t);
  $k = hf_sanitize_key("Option-Name_$i!");
  if (in_array($k, ["option-name_0"], true)) $n++;
  $len += strlen($t) + strlen($k);
}
echo $len, " ", $n, "\n";
