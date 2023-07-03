#!/bin/bash

set -e

rm -rf huertgenwald
cp -r "cache/KDVZ Frechen: Gemeinde Hürtgenwald" huertgenwald
find huertgenwald -type f -exec sed -i -r 's/"https:\/\/sdnetrim.kdvz-frechen.de\/rim4220\/webservice\/oparl\/v1.0\/([^"]*)"/"https:\/\/127.0.0.1:8080\/\1.json"/g' {} \;
find huertgenwald -type f -exec sed -i -r 's/\?page=/%3Fpage%3D/g' {} \;
