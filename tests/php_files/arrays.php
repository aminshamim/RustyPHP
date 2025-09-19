<?php
// STEP 3: Arrays Test

$indexed = [1, 2, 3];
$assoc = ["name" => "RustyPHP", "version" => 2, "active" => true];
$nested = ["numbers" => $indexed, "meta" => $assoc];

// Element access
echo $indexed[0]; // 1
echo "\n";
echo $indexed[2]; // 3
echo "\n";

echo $assoc["name"]; // RustyPHP
echo "\n";

echo $nested["numbers"][1]; // 2
echo "\n";

echo "Array literal printing: ";
echo $indexed; // prints Array

echo "\nDone arrays test\n";
?>
