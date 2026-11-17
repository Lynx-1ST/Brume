# Bản Windows của Lynx-1ST/Serein

## Tải và chọn tiếng Việt

Tải bản Windows x64 từ [GitHub Releases của fork](https://github.com/Lynx-1ST/Serein/releases).
Dùng `-Setup.exe` để cài đặt, hoặc giải nén `.zip` để chạy bản portable.
Bản `.exe` độc lập cũng được đính kèm; gói ZIP có thêm giấy phép và công cụ đăng ký thông báo.

Vào **Settings → Language & Time → Display language → Tiếng Việt**. Lựa chọn được lưu trên thiết bị;
chế độ **Hệ thống** cũng nhận diện `vi-VN`. Nội dung tin nhắn và ngôn ngữ giao thức Discord
không thay đổi. Bản dịch ban đầu có hỗ trợ dịch máy và đã rà các nhãn điều hướng chính;
các chuỗi ít dùng vẫn cần người dùng kiểm tra trong ngữ cảnh thực tế.

## Chat trong kênh thoại và người xem stream

Nhấn nút bong bóng **Mở trò chuyện** ở bên phải hàng kênh thoại để mở tin nhắn của kênh.
Thao tác này không tham gia cuộc gọi hoặc chuyển cuộc gọi đang kết nối.
Nhấp đúp vào tên kênh vẫn dùng luồng tham gia thoại hiện có.

Tính năng hiển thị avatar, số lượng và tên người xem stream, cùng bản sửa **Open Stream**,
được port từ [PR upstream #1017](https://github.com/ViceVerse-cz/Serein/pull/1017),
head `9e38c330a66043e34e3052db95cb41c1539c87a4`, của dylanjkl.
Giữ giới hạn 64 ID người xem và chỉ hiển thị dữ liệu của phiên stream đang hoạt động.
Việc Discord gửi danh sách người xem cho stream được xem vẫn là hành vi giao thức chưa được
xác minh trực tiếp; nếu không nhận được dữ liệu, ứng dụng không hiện danh sách.

## CI/CD

- CI native chỉ dùng runner Windows x64.
- **Windows Release** build và phát hành bản nightly khi push vào `main`.
- Để phát hành bản ổn định, chạy workflow từ `main` qua **Actions → Windows Release → Run workflow**,
  nhập phiên bản SemVer chưa dùng (ví dụ `0.1.1`), hoặc push tag `v0.1.1`.
- Workflow kiểm tra demo, chạy `cargo xtask check`, đóng gói production có voice và không có demo,
  rồi tải bộ cài, EXE, ZIP và `SHA256SUMS.txt` lên Release.
- Job build chỉ có quyền đọc; job publish dùng `GITHUB_TOKEN` với `contents: write`.
  Không cần PAT hoặc khóa ký macOS. Workflow release đa nền tảng của upstream không chạy trong fork.
- Updater của bản fork chỉ nhận gói từ `Lynx-1ST/Serein`. Nightly thuộc kênh cập nhật nightly;
  bản ổn định thuộc kênh production. Tên ZIP và định dạng checksum giữ tương thích updater.

Các EXE chưa được ký mã. Chưa xác minh đăng nhập, cuộc gọi, stream thật hoặc giao diện native
trên máy phát triển này; kiểm thử tổng hợp không chứng minh tương thích Discord thực tế.
