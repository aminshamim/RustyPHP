wrk.method = "POST"
wrk.body = '{"code": "<?php $name = \"RustyPHP\"; $version = 1.0; echo \"$name v$version\"; ?>"}'
wrk.headers["Content-Type"] = "application/json"
