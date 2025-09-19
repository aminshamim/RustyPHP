<?php
// Single-line comment
# Another single-line comment
/*
   Multi-line comment
*/

// ----- VARIABLES -----
$name = "Amin";
$age = 30;
$height = 5.9;
$isDeveloper = true;

// ----- CONSTANTS -----
define("APP_NAME", "My Simple PHP App");
const VERSION = "1.0.0";

// ----- OUTPUT -----
echo "<h1>Welcome to " . APP_NAME . " (v" . VERSION . ")</h1>";
echo "Hello, my name is $name and I am $age years old.<br>";

// ----- ARRAYS -----
$colors = ["red", "green", "blue"];
$person = [
    "name" => $name,
    "age" => $age,
    "developer" => $isDeveloper
];

// Indexed array loop
echo "<h3>Colors:</h3>";
foreach ($colors as $color) {
    echo "$color<br>";
}

// Associative array loop
echo "<h3>Person Details:</h3>";
foreach ($person as $key => $value) {
    echo ucfirst($key) . ": $value<br>";
}

// ----- CONDITIONALS -----
if ($age < 18) {
    echo "You are a minor.<br>";
} elseif ($age >= 18 && $age < 60) {
    echo "You are an adult.<br>";
} else {
    echo "You are a senior.<br>";
}

// ----- FUNCTIONS -----
function greet($who = "Guest") {
    return "Hello, $who!<br>";
}
echo greet($name);
echo greet();

// ----- LOOPS -----
echo "<h3>Numbers:</h3>";
for ($i = 1; $i <= 5; $i++) {
    echo "$i ";
}

// While loop
echo "<br><h3>Countdown:</h3>";
$count = 5;
while ($count > 0) {
    echo "$count ";
    $count--;
}

// ----- CLASSES & OBJECTS -----
class Animal {
    public $type;
    public function __construct($type) {
        $this->type = $type;
    }
    public function speak() {
        return "The $this->type makes a sound.<br>";
    }
}

class Dog extends Animal {
    public function speak() {
        return "The $this->type barks!<br>";
    }
}

$animal = new Animal("animal");
$dog = new Dog("dog");
echo $animal->speak();
echo $dog->speak();

// ----- SUPERGLOBALS -----
echo "<h3>Superglobals:</h3>";
echo "Server name: " . $_SERVER['SERVER_NAME'] . "<br>";
echo "Script filename: " . $_SERVER['SCRIPT_FILENAME'] . "<br>";

?>
