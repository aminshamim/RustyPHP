wrk.method = "POST"
wrk.body = '{"code": "<?php $a = 10; $b = 5; echo $a + $b * 2; ?>"}'
wrk.headers["Content-Type"] = "application/json"
