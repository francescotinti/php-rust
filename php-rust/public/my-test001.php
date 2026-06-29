<?php
echo "<!DOCTYPE html><html><head><title>PHP-Rust Test Suite</title>";
echo "<style>body{font-family: monospace; line-height: 1.6; padding: 20px;} h2{color: #2c3e50; border-bottom: 1px solid #eee; padding-bottom: 5px;} .success{color: green;} .test-block{background: #f9f9f9; padding: 15px; margin-bottom: 20px; border-radius: 5px; border: 1px solid #ddd;}</style>";
echo "</head><body>";
echo "<h1>🐘 PHP-Rust Engine: Core Language My Test</h1>";

function potenza($x) {
	if (is_numeric($x) == false) {
		return -1;
	}
	
	return $x*$x;
}

function trim_stringa($stringa, $inizio, $durata) {
		
	$lunghezza = strlen($stringa);
	
	if ($inizio > $lunghezza) {
		return -1;
	}
	
	if ($durata > $lunghezza) {
		return -1;
	}

	$result = substr($stringa, $inizio, $durata);
	return $result;
}

$valore = potenza(10);
echo "10 $valore<br>";

$valore = potenza('A');
echo "A $valore<br>";


echo trim_stringa("Buongiorno signori", 0, 10);

echo trim_stringa("Buongiorno signori", 11, -5);


echo "</body></html>";
?>