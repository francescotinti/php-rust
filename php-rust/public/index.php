<?php
echo "<!DOCTYPE html><html><head><title>PHP-Rust Test Suite</title>";
echo "<style>body{font-family: monospace; line-height: 1.6; padding: 20px;} h2{color: #2c3e50; border-bottom: 1px solid #eee; padding-bottom: 5px;} .success{color: green;} .test-block{background: #f9f9f9; padding: 15px; margin-bottom: 20px; border-radius: 5px; border: 1px solid #ddd;}</style>";
echo "</head><body>";
echo "<h1>🐘 PHP-Rust Engine: Core Language Tests</h1>";

// ---------------------------------------------------------
echo "<h2>1. Tipi di Base e Variabili</h2>";
echo "<div class='test-block'>";
$int = 42;
$float = 3.14;
$string = "Hello Rust!";
$bool = true;
echo "Intero: $int | Float: $float | Stringa: $string | Booleano: " . ($bool ? 'true' : 'false') . "<br>";
echo "Concatenazione: " . $string . " " . $int . "<br>";
echo "</div>";

// ---------------------------------------------------------
echo "<h2>2. Strutture di Controllo</h2>";
echo "<div class='test-block'>";
$result = "";
for ($i = 1; $i <= 3; $i++) {
    if ($i % 2 == 0) {
        $result .= "$i è pari. ";
    } else {
        $result .= "$i è dispari. ";
    }
}
echo "Risultato Ciclo For/If: $result<br>";

$count = 0;
while ($count < 2) {
    echo "Ciclo While iterazione $count<br>";
    $count++;
}
echo "</div>";

// ---------------------------------------------------------
echo "<h2>3. Array (Associativi e Indicizzati)</h2>";
echo "<div class='test-block'>";
$array_indicizzato = [10, 20, 30];
$array_associativo = ["nome" => "Zend", "motore" => "Rust"];
echo "Array Indicizzato [1]: " . $array_indicizzato[1] . "<br>";
echo "Array Associativo ['motore']: " . $array_associativo['motore'] . "<br>";

echo "<strong>Foreach:</strong><br>";
foreach ($array_associativo as $key => $value) {
    echo "- $key: $value<br>";
}
echo "</div>";

// ---------------------------------------------------------
echo "<h2>4. Destructuring (list)</h2>";
echo "<div class='test-block'>";
$coordinate = [45.4642, 9.1900];
list($lat, $lng) = $coordinate;
echo "Destructuring array: Lat=$lat, Lng=$lng<br>";

$nested = [[1, 2], [3, 4]];
foreach ($nested as list($a, $b)) {
    echo "Destructuring in foreach: $a, $b<br>";
}
echo "</div>";

// ---------------------------------------------------------
echo "<h2>5. Funzioni e Closure</h2>";
echo "<div class='test-block'>";
function calcola_somma($a, $b) {
    return $a + $b;
}
echo "Risultato funzione calcola_somma(5, 7): " . calcola_somma(5, 7) . "<br>";

$moltiplicatore = 10;
$closure = function($x) use ($moltiplicatore) {
    return $x * $moltiplicatore;
};
echo "Risultato Closure con 'use': " . $closure(5) . "<br>";
echo "</div>";

// ---------------------------------------------------------
echo "<h2>6. Object Oriented Programming (OOP)</h2>";
echo "<div class='test-block'>";
class Veicolo {
    public $marca;
    
    // NOTA: In PHP il costruttore è __construct() (non __constructor)
    public function __construct($marca) {
        $this->marca = $marca;
    }
    
    public function descrivi() {
        return "Questo è un veicolo di marca " . $this->marca;
    }
}

$auto = new Veicolo("Ferrari");
echo $auto->descrivi() . "<br>";

echo "<strong>Var Dump dell'Oggetto:</strong><br><pre>";
//ob_start();
var_dump($auto);
//$dump = ob_get_clean();
//echo $dump;
echo "</pre></div>";

// ---------------------------------------------------------
echo "<h2>7. Eccezioni (Try / Catch)</h2>";
echo "<div class='test-block'>";
try {
    throw new Exception("Test eccezione lanciata!");
    echo "Questo non dovrebbe essere stampato.";
} catch (Exception $e) {
    echo "<span class='success'>✅ Eccezione catturata correttamente: " . $e->getMessage() . "</span><br>";
}
echo "</div>";

echo "<h3 class='success'>🎉 Se vedi questo messaggio e nessun Fatal Error, il core language sta funzionando alla grande!</h3>";
echo "</body></html>";


echo "<h3>Test ref</h3>";
$a = "Valore";

$arr = ['Pera', 'Mela', 'Banana', 'Ananas'];
foreach($arr as $chiave => $valore) {
	echo "$chiave = $valore <br>";
}