<?php
declare(strict_types=1);

// 1) constants, variables
const APP_NAME = 'DemoApp';
define('APP_ENV', 'local');

$greeting = "Hello";
$pi = 3.14159;
$count = 0;
$isOk  = true;
$maybe = null;

// 2) cli args & env
$rawQuery = $argv[1] ?? '';
parse_str($rawQuery, $cliGet);
$_GET = array_merge($_GET, $cliGet);

$user = $_GET['name'] ?? 'Guest';
$n    = (int)($_GET['n'] ?? 3);
$home = getenv('HOME') ?: '(no HOME)';

// 3) strings
$who = "$greeting, $user";
$info = <<<TEXT
App:    {APP_NAME}
Env:    {APP_ENV}
User:   $user
HOME:   $home
TEXT;

$literal = <<<'NOWDOC'
This is a nowdoc block: $variables won't expand here.
NOWDOC;

// 4) arrays, spread, destructuring, references
$nums = [1, 2, 3];
$more = [4, 5];
$all  = [...$nums, 0, ...$more];

[$first, $second] = $nums;
['name' => $name2] = ['name' => 'Shamim'];

$a = 10;
$b =& $a;
$b++;

// 5) functions
function sum(int|float ...$values): int|float {
    return array_sum($values);
}
function box(string $text, int $pad = 1): string {
    $s = str_repeat(' ', $pad);
    return "[$s$text$s]";
}
function bumpCounter(): int {
    static $c = 0;
    return ++$c;
}

$total = sum(...$all);
$boxed = box(text: $who, pad: 2);

// 6) closures, arrow fns, first-class callables
$add = fn(int|float $x, int|float $y) => $x + $y;
function apply(callable $fn, mixed $v): mixed { return $fn($v); }
$timesTwo = function(int $x): int { return $x * 2; };
$applied  = apply($timesTwo(...), 21);

// 7) sorting
$words = ['pear', 'banana', 'apple'];
usort($words, fn($l, $r) => $l <=> $r);

// 8) control flow
$age = (int)($_GET['age'] ?? 20);
$stage = $age >= 18 ? 'adult' : 'minor';

$code = (int)($_GET['code'] ?? 200);
$statusText = match (true) {
    $code >= 200 && $code < 300 => 'OK',
    $code >= 400 && $code < 500 => 'Client Error',
    $code >= 500                => 'Server Error',
    default                     => 'Other',
};

// 9) loops
$sumLoop = 0;
for ($i = 0; $i < $n; $i++) { $sumLoop += $i; }

$j = 0;
while ($j < 2) { $j++; }

$colors = ['red', 'green', 'blue'];
$colorStr = '';
foreach ($colors as $c) { $colorStr .= "$c "; }

// 10) generators
function ints(int $start, int $end): Generator {
    for ($i = $start; $i <= $end; $i++) yield $i;
}
function ints2(): Generator { yield from ints(5, 7); }
$genSum = array_sum(iterator_to_array(ints2()));

// 11) json, try/catch
$data = ['user' => $user, 'n' => $n, 'ok' => $isOk];
$json = json_encode($data, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE);

try {
    $decoded = json_decode($json, true, flags: JSON_THROW_ON_ERROR);
} catch (JsonException $e) {
    $decoded = ['error' => 'bad_json', 'message' => $e->getMessage()];
} finally {
    $touchedFinally = true;
}

// error handler demo
set_error_handler(function(int $errno, string $errstr) {
    echo "[php-error:$errno] $errstr\n";
    return true;
});

// 12) regex + filter
$email = $_GET['email'] ?? 'not@an.email';
$isEmail = (bool)preg_match('/^[^\s@]+@[^\s@]+\.[^\s@]+$/', $email);

$age2 = filter_var($_GET['age'] ?? 0, FILTER_VALIDATE_INT, [
    'options' => ['default' => 0, 'min_range' => 0, 'max_range' => 150]
]);

// 13) buffering
ob_start();
echo "Buffered line 1\n";
echo "Buffered line 2\n";
$buffered = ob_get_clean();

// 14) null coalescing assignment
$alias = $_GET['alias'] ?? null;
$alias ??= strtoupper($user);

// 15) helper
function h(string $title): void { echo "\n=== $title ===\n"; }

// -------------------------
// OUTPUT
// -------------------------
h('Basics');
printf("APP_NAME=%s; APP_ENV=%s\n", APP_NAME, APP_ENV);
echo "who: $who\n";
echo "info:\n$info\n";
echo "nowdoc:\n$literal\n";

h('Arrays');
echo 'all: ' . json_encode($all) . "\n";
echo "first=$first, second=$second, name2=$name2\n";
echo "a(after ref inc)=$a\n";

h('Functions');
echo 'sum(all)= ' . $total . "\n";
echo 'box(who)= ' . $boxed . "\n";
echo 'bumpCounter: ' . bumpCounter() . ', ' . bumpCounter() . "\n";

h('Closures & Callables');
echo 'add(2,3)= ' . $add(2, 3) . "\n";
echo 'apply(timesTwo,21)= ' . $applied . "\n";

h('Sorting');
echo 'words: ' . implode(', ', $words) . "\n";

h('Control Flow');
echo "age=$age => stage=$stage\n";
echo "HTTP $code => $statusText\n";

h('Loops & Generators');
echo "sumLoop(0..".($n-1).")= $sumLoop\n";
echo "colors= $colorStr\n";
echo "genSum(5..7)= $genSum\n";

h('JSON & Errors');
echo "json: $json\n";
echo 'decoded: ' . json_encode($decoded, JSON_UNESCAPED_SLASHES) . "\n";
echo 'finally touched? ' . (isset($touchedFinally) ? 'yes' : 'no') . "\n";

h('Regex & Filter');
echo "email='$email' valid? " . ($isEmail ? 'yes' : 'no') . "\n";
echo "age2(validated)=$age2\n";

h('Buffering & Misc');
echo "buffered:\n$buffered";
echo "alias=$alias\n";

h('Done');
echo "Bye!\n";
