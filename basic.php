<?php
// Single-line comment
# Another single-line comment
/*
   Multi-line comment
*/

// Variables
$name = "Amin";
$age = 30;
$isAdmin = true;
$price = 99.99;

// Constants
define("SITE_NAME", "My PHP Site");
const VERSION = "1.0.0";

// Arrays
$colors = ["red", "green", "blue"];
$person = [
    "name" => "Shamim",
    "age"  => 28,
];

// Functions
function greet($username) {
    return "Hello, $username!";
}

// Control Structures
if ($age >= 18) {
    echo "You are an adult.\n";
} elseif ($age > 12) {
    echo "You are a teenager.\n";
} else {
    echo "You are a child.\n";
}

// Loops
for ($i = 0; $i < 3; $i++) {
    echo "For loop count: $i\n";
}

$counter = 0;
while ($counter < 2) {
    echo "While loop count: $counter\n";
    $counter++;
}

foreach ($colors as $color) {
    echo "Color: $color\n";
}

// String Interpolation
echo "My name is $name and I am $age years old.\n";

// Function call
echo greet($name) . "\n";

// Working with arrays
echo "First color is " . $colors[0] . "\n";
echo "Person’s name: " . $person['name'] . "\n";

// Null coalescing operator
$user = $_GET['user'] ?? "Guest";
echo "Welcome, $user\n";

// Switch case
switch ($name) {
    case "Amin":
        echo "Name is Amin\n";
        break;
    case "Shamim":
        echo "Name is Shamim\n";
        break;
    default:
        echo "Unknown name\n";
}
?>