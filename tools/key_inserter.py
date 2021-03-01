#!/usr/bin/env python
#
# Tool digunakan untuk mempermudah insert key ke local Nuchain node melalui CLI.
#
#

import requests as req
import os
import json

url = os.environ.get("NUCHAIN_RPC", "http://localhost:9933")
headers = {'Content-Type': "application/json"}


def payload(method, *params):
    rv = {
        "id": 1,
        "jsonrpc": "2.0",
        "method": method,
        "params": params
    }
    return json.dumps(rv)

def insert_key(ktype, secseed, pubkey):
    data = payload("author_insertKey", ktype, secseed, pubkey)
    print(data)
    resp = req.post(url, headers=headers, data = data)
    print("%s:=> %s" % (ktype, resp))
    if resp.status_code == 200:
        print(resp.text)
    else:
        print("error %s" % resp)
    return resp

if __name__ == "__main__":
    print("Nuchain Node Key inserter")
    print("rpc:", url)
    print("Input key type `gran`:")
    secseed = input("secseed: ").strip()
    pubkey = input("pubkey: ").strip()
    insert_key("gran", secseed, pubkey)
    print("----------------------------------")
    print("Input key type `babe, imon, audi`:")
    secseed = input("secseed: ").strip()
    pubkey = input("pubkey: ").strip()
    insert_key("babe", secseed, pubkey)
    insert_key("imon", secseed, pubkey)
    insert_key("audi", secseed, pubkey)
