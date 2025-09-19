wrk.method = "POST"
wrk.body = '{"code": "<?php echo \"Hello, World!\"; ?>"}'
wrk.headers["Content-Type"] = "application/json"
