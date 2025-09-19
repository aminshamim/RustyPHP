<?php
// STEP 1: Comments test - Working ✓
/* Block comment */
# Hash comment

// STEP 2: Constants test
define("SITE_NAME", "RustyPHP Website");
define("VERSION", 1.5);
const API_KEY = "abc123";

echo "Welcome to ";
echo SITE_NAME;
echo "\n";
echo "Version: ";
echo VERSION;
echo "\n";
echo "API Key: ";
echo API_KEY;
echo "\n";

$greeting = "Hello from ";
echo $greeting;
echo SITE_NAME;
echo "\n";
?>
