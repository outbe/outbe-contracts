#!/usr/bin/env sh
set -e

IFS=$'\r\n'
GLOBIGNORE='*'

target_file=$1

contracts="["

for x in `cat $target_file | grep CONTRACT_ADDRESS`; do
  name=$(echo $x | cut -d = -f1)
  address=$(echo $x | cut -d = -f2)
  contracts="${contracts}{\"name\": \"${name}\", \"address\": \"$address\"},"
done

contracts="${contracts%?}]"

echo $contracts
