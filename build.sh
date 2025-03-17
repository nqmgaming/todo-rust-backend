#!/bin/bash

# Biên dịch ứng dụng
cargo build

# Hiển thị kết quả
if [ $? -eq 0 ]; then
    echo "Biên dịch thành công!"
else
    echo "Biên dịch thất bại!"
fi 