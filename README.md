# Serein · Windows fork

Ứng dụng Discord desktop viết bằng **Rust**, với giao diện native **egui/wgpu**. Đây là bản fork của [ViceVerse-cz/Serein](https://github.com/ViceVerse-cz/Serein), do **Lynx-1ST** duy trì, tập trung vào Windows và trải nghiệm tiếng Việt.

Giao diện chính chạy native; đăng nhập sử dụng trang Discord chính thức trong WebView2 tạm thời.

[**Tải bản Windows**](https://github.com/Lynx-1ST/Serein/releases) · [Hướng dẫn chi tiết](docs/windows-fork.md) · [Báo lỗi](https://github.com/Lynx-1ST/Serein/issues) · [Dự án gốc](https://github.com/ViceVerse-cz/Serein)

<p align="center">
  <img src="docs/preview.png" alt="Giao diện Serein — ảnh minh họa từ dự án gốc" width="900" />
</p>

<p align="center">
  <a href="https://github.com/Lynx-1ST/Serein/releases"><img src="https://img.shields.io/github/v/release/Lynx-1ST/Serein?include_prereleases&label=Windows%20release" alt="Bản phát hành của fork" /></a>
  <a href=".github/workflows/ci.yml"><img src="https://img.shields.io/badge/platform-Windows%20x64-blue" alt="Windows x64" /></a>
  <a href="rust-toolchain.toml"><img src="https://img.shields.io/badge/Rust-1.99.1-orange?logo=rust" alt="Rust 1.99.1" /></a>
  <a href="LICENSE-MIT"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-green" alt="MIT hoặc Apache-2.0" /></a>
</p>

> [!WARNING]
> Serein là ứng dụng không chính thức, không được Discord chứng thực. Việc dùng tài khoản cá nhân qua client bên thứ ba có rủi ro vi phạm điều khoản Discord và bị khóa tài khoản. Đọc [khả năng tương thích](docs/discord-compatibility.md) và [hướng dẫn đăng nhập](docs/authentication.md) trước khi sử dụng.

## Tải và cài đặt

Fork này chỉ phát hành **Windows x64**. Mở [GitHub Releases của Lynx-1ST/Serein](https://github.com/Lynx-1ST/Serein/releases) và chọn gói phù hợp:

| Gói | Cách dùng |
| --- | --- |
| Bộ cài `-Setup.exe` | Cài cho tài khoản Windows hiện tại; phù hợp với phần lớn người dùng. |
| Bản portable `.zip` | Giải nén vào thư mục có quyền ghi, rồi chạy `serein.exe`. Gói có kèm giấy phép và công cụ đăng ký thông báo. |
| EXE độc lập | Chạy trực tiếp; dùng ZIP nếu cần đầy đủ tài liệu và công cụ đi kèm. |
| `SHA256SUMS.txt` | Đối chiếu mã SHA-256 của các tệp tải về. |

Đăng nhập cần **Microsoft Edge WebView2 Runtime**. Các gói phát hành của fork hiện chưa được ký mã.

Để bật tiếng Việt, vào **Settings → Language & Time → Display language → Tiếng Việt**. Lựa chọn được lưu trên thiết bị; chế độ ngôn ngữ hệ thống cũng nhận diện `vi-VN`.

### Cập nhật trong ứng dụng

Updater của fork lấy bản phát hành từ **Lynx-1ST/Serein**. Vào phần **Cập nhật** trong cài đặt để kiểm tra; chọn kênh **nightly** nếu đang dùng các bản `-nightly`, hoặc kênh **production** cho bản ổn định.

Bản cài từ upstream hoặc bản fork cũ chưa đổi địa chỉ updater cần được thay bằng bản EXE của fork trước. Bản portable cần nằm trong thư mục có quyền ghi để cập nhật được.

## Những thay đổi của fork

| Phần | Thay đổi |
| --- | --- |
| Tiếng Việt | Bổ sung catalog tiếng Việt, rà thuật ngữ theo ngữ cảnh Discord và định dạng ngày giờ. Nếu còn chuỗi chưa dịch hoặc dịch chưa tự nhiên, hãy báo lỗi kèm ảnh và vị trí. |
| Chat trong kênh thoại | Hover hàng kênh để hiện nút trò chuyện và mời khi có quyền; menu chuột phải có **Mở trò chuyện**, không cần tham gia cuộc gọi. |
| Thành viên trong voice | Hiện tag máy chủ; ưu tiên người đang live, sau đó xếp theo tên hiển thị không phân biệt hoa/thường, dùng ID để phân xử tên trùng. |
| Thanh thành viên bên phải | Kéo mép trái để chỉnh độ rộng; lưu lựa chọn trên thiết bị. |
| Xem stream | Chọn tối đa bốn stream trong cùng cuộc gọi, cải thiện lịch trình chiếu khung hình và hiển thị người xem. |
| PiP trong Serein | Xem stream của người khác hoặc bản xem trước của mình khi chuyển sang màn hình khác; hỗ trợ kéo, đổi kích thước, dãy avatar, biểu tượng mắt và hover tên người xem. |
| Nút X trên PiP | Dừng xem stream được chọn; với stream của mình thì dừng chia sẻ. Nút trở lại đưa về kênh thoại. |
| Điều khiển stream | Nút ba chấm khi đang chia sẻ mở chỉnh độ phân giải/FPS. Áp dụng sẽ khởi động lại stream với cấu hình mới và giữ kết nối thoại. |
| Giao diện xem stream | Bỏ bảng ghi chú hover ở góc dưới trái của hình stream; giữ các nút điều khiển ở giữa. |
| Liên kết | Menu sao chép liên kết; sửa thao tác mở URL bằng trình duyệt mặc định trên Windows. |
| Mic và cuộc gọi | Thêm điều chỉnh độ nhạy tự động, nhận diện giọng nói nâng cao, cảnh báo không phát hiện âm thanh và xác nhận chuyển kênh thoại. |
| Cảnh báo mic câm | Đo mức tín hiệu **thô từ thiết bị**, trước lọc tạp âm và trước cổng nhận diện. Trước đây đo sau lọc tạp âm nên ngồi im trong phòng yên tĩnh với Voice Isolation cũng bị báo oan. |
| Âm thanh stream | Vòng lặp thoại chạy trễ không còn xóa sạch bộ đệm phát của stream mà chỉ cắt phần tồn dư, nên hết hiện tượng hụt tiếng dồn 3–4 lần khi máy đang tải nặng. |
| Stereo cho stream | Stream được giải mã và phát **stereo** suốt chặng, từ bộ giải mã tới thiết bị. Thoại vẫn mono vì Discord gửi mono. Hai kênh dùng chung pha khi nội suy để không bị lệch. |
| Bộ nhớ hình của GPU | **Settings → Storage** có thêm ngân sách bộ nhớ hình cho card đồ họa (32–192 MB, mặc định 96 MB giữ nguyên hành vi cũ). Hạ xuống thì giải phóng ngay, đổi lại ảnh phải tải lại khi cuộn ngược. |
| Ảnh động | Avatar và sticker động vẽ nhỏ hơn kích thước gốc không còn nạp thừa một texture mỗi khung hình. |
| Gợi ý nhắc tên | Dùng biệt danh đúng máy chủ khi dữ liệu có sẵn, kể cả thành viên ngoài vùng danh sách đang hiển thị. |
| Emoji | Cải thiện độ nét theo DPI với Twemoji và cache có giới hạn. |
| DAVE | Tích hợp bản port đã sửa từ PR #862, gồm xử lý chuyển khóa và khôi phục stream. |

Lọc tạp âm mặc định ở chế độ **Tự động**: dùng **DeepFilterNet** và tự chuyển về **RNNoise** khi máy không xử lý kịp. Khi đang chuyển về RNNoise, bảng cài đặt sẽ nói rõ điều đó thay vì im lặng.

Nhận diện giọng nói nâng cao dùng xác suất giọng nói từ **RNNoise**, không phải thuật toán độc quyền của Discord. Khi RNNoise đang lọc tạp âm thì xác suất này có sẵn; khi DeepFilterNet lọc, một bộ dò RNNoise riêng chạy thêm chỉ để lấy xác suất.

Danh sách người xem phụ thuộc dữ liệu Discord gửi, ứng dụng không tự suy đoán khi thiếu dữ liệu.

### Giới hạn đã biết

- Phần làm lại khung trang trí ngoài của popout tài khoản đang được hoãn.
- Animation khởi động mới chưa được tích hợp.

Chi tiết cấu hình và giới hạn: [Windows fork](docs/windows-fork.md).

## Các chức năng nền tảng

Kế thừa từ Serein gốc: nhắn tin và DM, Markdown, biểu cảm/GIF, tệp đính kèm, luồng và diễn đàn, hồ sơ người dùng, quản lý máy chủ, theme/extension và Rich Presence. Voice có Opus, DAVE, khử tiếng vọng, lọc ồn, nhấn để nói, camera và chia sẻ màn hình native.

Mã nguồn và kiểm thử tổng hợp không chứng minh mọi chức năng hoạt động với Discord thực tế. Xem [tài liệu tương thích](docs/discord-compatibility.md) cho các giới hạn giao thức; ảnh minh họa phía trên không phải bằng chứng kiểm thử bản Windows hiện tại.

## Đồng bộ upstream và CI/CD

- Fork này **không còn tự đồng bộ** với dự án gốc. Khi muốn lấy thay đổi từ upstream, merge bằng tay: `git fetch upstream && git merge upstream/main`, xử lý xung đột rồi đẩy lên `main`.
- Có commit mới thì merge vào fork, giữ các thay đổi riêng; gặp xung đột thì dừng để xử lý, không force push.
- Sau khi sync, **Serein Fork CI** được yêu cầu chạy. Khi CI của `main` thành công, **Windows Release** build và phát hành nightly của đúng commit đã kiểm tra.
- Khi merge PR, CI có thể dùng lại kết quả thành công trong 24 giờ nếu cây Git trùng hoàn toàn và đủ mọi job bắt buộc. Nếu không xác minh được, CI chạy đầy đủ.
- Build native và đóng gói chỉ dành cho Windows x64. Một số kiểm tra phụ trợ không cần Windows vẫn dùng runner Ubuntu.
- Không có commit mới thì không tạo thêm release chỉ vì đến lịch sync. Lịch GitHub Actions có thể chạy trễ.

## Build từ mã nguồn trên Windows

Cần Git, Rust theo [toolchain đã ghim](rust-toolchain.toml), Visual Studio C++ Build Tools với Windows SDK, CMake và libclang. Đăng nhập cần WebView2. Xem thêm [yêu cầu nền tảng](docs/platform-support.md).

```powershell
git clone https://github.com/Lynx-1ST/Serein.git
cd Serein

# Xem giao diện bằng dữ liệu tổng hợp
cargo run --locked --features demo -- --demo

# Chạy client thông thường khi bạn sẵn sàng đăng nhập
cargo run --locked

# Kiểm tra workspace
cargo xtask check

# Đóng gói production có voice, không kèm demo
cargo xtask package
```

Build từ source dùng thư mục dữ liệu `serein-development`, tách khỏi bản cài đặt. Có thể đặt `SEREIN_DATA_DIR` thành đường dẫn tuyệt đối để tách các môi trường phát triển.

## Tài liệu và báo lỗi

- [Thiết lập tiếng Việt, voice và quy trình phát hành của fork](docs/windows-fork.md)
- [Đăng nhập](docs/authentication.md) · [Tương thích Discord](docs/discord-compatibility.md)
- [Kiến trúc](docs/architecture.md) · [Voice](docs/voice.md) · [Extension](docs/extensions.md)
- [Lưu trữ dữ liệu](docs/storage-policy.md) · [Mô hình bảo mật](docs/threat-model.md)
- [Báo lỗi cho fork](https://github.com/Lynx-1ST/Serein/issues): ghi phiên bản, phiên bản Windows, các bước tái hiện và ảnh nếu cần. Không gửi token hoặc thông tin đăng nhập.

## Ghi công và giấy phép

Serein gốc được phát triển bởi [ViceVerse-cz và các cộng tác viên](https://github.com/ViceVerse-cz/Serein). Fork giữ giấy phép **MIT hoặc Apache-2.0**, tùy lựa chọn: [LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE).

Các bản port tham khảo upstream: [#1017](https://github.com/ViceVerse-cz/Serein/pull/1017), [#1040](https://github.com/ViceVerse-cz/Serein/pull/1040), [#916](https://github.com/ViceVerse-cz/Serein/pull/916), [#886](https://github.com/ViceVerse-cz/Serein/pull/886) và [#862](https://github.com/ViceVerse-cz/Serein/pull/862). Thông tin ghi công chi tiết được giữ trong lịch sử commit và tài liệu liên quan.

Giấy phép thư viện, font và Twemoji: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
