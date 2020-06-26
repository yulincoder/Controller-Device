#!/bin/bash
for((i=0;i<2;i++));
do
    sn="abc"$i
    echo -e "\n----------\n"${sn}
    curl -i -X POST -H "Content-Type: application/json" -d "{\"data\":\"fuck\", \"sn\":\"${sn}\"}" http://39.105.63.97:8080/push/push_msg
    #curl -i -X POST -H "Content-Type: application/json" -d "{\"data\":\"fuck\", \"sn\":\"${sn}\"}" http://127.0.0.1:8080/push/push_msg
done
