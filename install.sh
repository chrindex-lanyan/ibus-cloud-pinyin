#!/bin/bash

cargo build --release
sudo cp target/release/ibus-cloud-pinyin /usr/local/bin/
sudo cp fcpinyin.xml /usr/share/ibus/component/
