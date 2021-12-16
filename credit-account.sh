#!/bin/bash

set -x
curl -i http://127.0.0.1:3000/account/sid/balance -X POST -H 'Content-Type: application/json' --data-binary '{"amount":0.10}'

