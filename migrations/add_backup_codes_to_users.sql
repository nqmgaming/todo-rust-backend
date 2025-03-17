-- Thêm cột backup_codes vào bảng users
ALTER TABLE users ADD COLUMN IF NOT EXISTS backup_codes TEXT[] DEFAULT NULL; 