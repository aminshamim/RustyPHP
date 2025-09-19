-- Random complex PHP code for each request
math.randomseed(os.time())

request = function()
    local operations = {
        '{"code": "<?php for($i=0; $i<10; $i++) { echo $i; } ?>"}',
        '{"code": "<?php $arr = [1,2,3]; foreach($arr as $v) { echo $v; } ?>"}',
        '{"code": "<?php function test($x) { return $x * 2; } echo test(21); ?>"}',
        '{"code": "<?php $data = [\"a\" => 1, \"b\" => 2]; echo $data[\"a\"] + $data[\"b\"]; ?>"}'
    }
    
    local body = operations[math.random(1, #operations)]
    
    return wrk.format("POST", nil, {["Content-Type"] = "application/json"}, body)
end
