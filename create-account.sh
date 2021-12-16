#!/bin/bash

set -x
curl -i http://127.0.0.1:3000/account -X POST -H 'Content-Type: application/json' --data-binary '{"accountId":"sid","balance":0}'
