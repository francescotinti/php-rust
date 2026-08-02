<?php
class J implements JsonSerializable { #[\ReturnTypeWillChange] public function jsonSerialize() { return ["a" => 1]; } }
echo json_encode(new J);
