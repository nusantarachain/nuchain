#!/usr/bin/env bash

source .env

if [ "$1" == "" ]; then
    echo "Usage: ulpoad_bin.sh [FILE-TO-UPLOAD]"
    exit 1
fi

REMOTE_TARGET=$DOWNLOAD_SERVER:/home/www/nuchain_download_repo/

echo "Uploading to $REMOTE_TARGET"$(basename $1)

scp -P 22 -i $PRIVATE_KEY $1 root@$REMOTE_TARGET
