#!/bin/bash

# Script triển khai tự động cho ứng dụng Rust Backend
# Tác giả: NQMGaming
# Phiên bản: 1.0.0

# Màu sắc cho thông báo
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Hàm hiển thị thông báo
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Hàm kiểm tra lỗi
check_error() {
    if [ $? -ne 0 ]; then
        log_error "$1"
        exit 1
    fi
}

# Hiển thị banner
echo -e "${GREEN}"
echo "====================================================="
echo "  Rust Backend - Script Triển Khai Tự Động"
echo "====================================================="
echo -e "${NC}"

# Kiểm tra xem git có được cài đặt không
if ! command -v git &> /dev/null; then
    log_error "Git không được cài đặt. Vui lòng cài đặt git và thử lại."
    exit 1
fi

# Kiểm tra xem docker có được cài đặt không
if ! command -v docker &> /dev/null; then
    log_error "Docker không được cài đặt. Vui lòng cài đặt docker và thử lại."
    exit 1
fi

# Kiểm tra xem docker-compose có được cài đặt không
if ! command -v docker compose &> /dev/null; then
    log_warn "Docker Compose không được cài đặt hoặc không phải là plugin của Docker CLI."
    log_warn "Thử sử dụng docker-compose thay thế..."
    
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose không được cài đặt. Vui lòng cài đặt docker-compose và thử lại."
        exit 1
    else
        DOCKER_COMPOSE="docker-compose"
    fi
else
    DOCKER_COMPOSE="docker compose"
fi

# Kiểm tra xem .env file có tồn tại không
if [ ! -f .env ]; then
    log_warn "File .env không tồn tại. Tạo file .env từ .env.example..."
    
    if [ -f .env.example ]; then
        cp .env.example .env
        log_info "Đã tạo file .env từ .env.example. Vui lòng cập nhật các biến môi trường."
    else
        log_error "File .env.example không tồn tại. Vui lòng tạo file .env thủ công."
        exit 1
    fi
fi

# Lưu trữ trạng thái hiện tại
log_info "Lưu trữ các thay đổi cục bộ..."
git stash
check_error "Không thể lưu trữ các thay đổi cục bộ."

# Kéo các thay đổi mới nhất từ repository
log_info "Đang kéo các thay đổi mới nhất từ repository..."
git pull origin main
check_error "Không thể kéo các thay đổi mới nhất từ repository."

# Khôi phục các thay đổi cục bộ nếu có
if git stash list | grep -q "stash@{0}"; then
    log_info "Khôi phục các thay đổi cục bộ..."
    git stash pop
    check_error "Không thể khôi phục các thay đổi cục bộ."
fi

# Dừng các container đang chạy
log_info "Dừng các container đang chạy..."
$DOCKER_COMPOSE down
check_error "Không thể dừng các container đang chạy."

# Xây dựng và khởi động các container
log_info "Đang xây dựng và khởi động các container..."
$DOCKER_COMPOSE up --build -d
check_error "Không thể xây dựng và khởi động các container."

# Kiểm tra trạng thái của các container
log_info "Kiểm tra trạng thái của các container..."
$DOCKER_COMPOSE ps
check_error "Không thể kiểm tra trạng thái của các container."

# Hiển thị logs của các container (tùy chọn)
read -p "Bạn có muốn xem logs của các container không? (y/n): " show_logs
if [[ $show_logs == "y" || $show_logs == "Y" ]]; then
    log_info "Hiển thị logs của các container..."
    $DOCKER_COMPOSE logs -f
fi

log_info "Triển khai hoàn tất thành công!"
echo -e "${GREEN}"
echo "====================================================="
echo "  Rust Backend - Đã triển khai thành công"
echo "====================================================="
echo -e "${NC}"