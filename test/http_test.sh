#!/bin/bash
for((i=0;i<100;i++));
do
    sn="abc"$i
    echo ${sn}
    curl -i -X POST -H "Content-Type: application/json" -d "{\"data\":\"fuck\", \"sn\":\"${sn}\"}" http://127.0.0.1:8080/push/push_msg
done
