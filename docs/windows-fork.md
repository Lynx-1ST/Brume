# Bản Windows của Lynx-1ST/Serein

## Tải và chọn tiếng Việt

Tải bản Windows x64 từ [GitHub Releases của fork](https://github.com/Lynx-1ST/Serein/releases).
Dùng `-Setup.exe` để cài đặt, hoặc giải nén `.zip` để chạy bản portable.
Bản `.exe` độc lập cũng được đính kèm; gói ZIP có thêm giấy phép và công cụ đăng ký thông báo.

Vào **Settings → Language & Time → Display language → Tiếng Việt**. Lựa chọn được lưu trên thiết bị;
chế độ **Hệ thống** cũng nhận diện `vi-VN`. Nội dung tin nhắn và ngôn ngữ giao thức Discord
không thay đổi. Toàn bộ catalog đã được rà theo ngữ cảnh Discord, gồm thuật ngữ máy chủ chung,
luồng, bản nháp, biểu cảm và quyền thành viên. Ngày/tháng, thứ và thời gian tương đối được hiển thị
bằng tiếng Việt ở tin nhắn, hồ sơ, huy hiệu, lời mời và thông tin máy chủ.

## Người xem stream

Khi di chuột lên hàng kênh thoại, nút **Mở trò chuyện** và nút mời (nếu có quyền) xuất hiện.
Rời chuột khỏi hàng sẽ ẩn các nút. **Mở trò chuyện** trong menu chuột phải cũng mở text chat
của kênh mà không tham gia cuộc gọi. Nhấp đúp vào tên kênh vẫn dùng luồng tham gia thoại.
Người tham gia hiển thị tag máy chủ cạnh tên khi có dữ liệu, theo cùng tùy chọn ẩn tag ở
danh sách thành viên. Tag nhường chỗ cho tên và trạng thái mic khi thanh kênh quá hẹp.

Tính năng hiển thị avatar, số lượng và tên người xem stream, cùng bản sửa **Open Stream**,
được port từ [PR upstream #1017](https://github.com/ViceVerse-cz/Serein/pull/1017),
head `9e38c330a66043e34e3052db95cb41c1539c87a4`, của dylanjkl.
Giữ giới hạn 64 ID người xem và chỉ hiển thị dữ liệu của phiên stream đang hoạt động.
Việc Discord gửi danh sách người xem cho stream được xem vẫn là hành vi giao thức chưa được
xác minh trực tiếp; nếu không nhận được dữ liệu, ứng dụng không hiện danh sách.

## Xem nhiều stream và cửa sổ nổi

Port [PR #916](https://github.com/ViceVerse-cz/Serein/pull/916) để chọn tối đa bốn stream
trong cùng cuộc gọi. Mỗi stream có phiên kết nối, trạng thái, hình ảnh và danh sách người xem
riêng. Dừng một stream không dừng các stream còn lại; âm lượng stream dùng bộ điều khiển chung.
[PR #886](https://github.com/ViceVerse-cz/Serein/pull/886) đưa lịch vẽ theo thời điểm trình chiếu
của khung hình. Các công cụ lab Linux đi kèm chỉ dành cho thử nghiệm ngoại tuyến.

PiP nổi bên trong Serein khi rời màn hình cuộc gọi; có thể mở từ menu của ô stream, kéo,
đổi kích thước và chọn stream qua menu chuột phải, gồm bản xem trước stream của mình.
PiP tái sử dụng texture đang xem, hiện dãy avatar và số người xem; di chuột lên để xem tên.
Nút trở lại và X nằm ở góc trên phải khi hover PiP. Tên màn hình chia sẻ và người xem
nằm trong thanh dưới riêng; tên dài được rút gọn để không chồng lên avatar người xem.
Nút X dừng xem stream được chọn, hoặc dừng chia sẻ nếu là stream của mình; nút trở lại
đưa về đúng kênh thoại. Mục đóng PiP trong menu chuột phải chỉ ẩn cửa sổ.
Lựa chọn được xóa khi đổi tài khoản hoặc phiên gọi, tránh hiển thị ảnh của phiên cũ.

## Biệt danh khi gợi ý nhắc tên

Trong máy chủ, tên ưu tiên biệt danh của đúng máy chủ. Ngoài máy chủ, ứng dụng dùng
biệt danh bạn bè nếu đã đặt, rồi tên hiển thị toàn cục (hoặc tên tài khoản do Discord cung cấp).
DM không dùng tên kênh cũ để thay cho tên người dùng. Hồ sơ mở trong máy chủ, tên stream
và danh sách voice cùng dùng thông tin thành viên đang lưu của máy chủ đó.

Sự kiện đổi/xóa biệt danh cập nhật các bản ghi thành viên đang giữ và danh sách voice,
không cần rời hoặc tham gia lại kênh. Thông tin voice vẫn được giữ khi đổi máy chủ đang xem.
Việc cập nhật tên không thay đổi trạng thái mic, stream hay kết nối cuộc gọi.

Gợi ý `@mention` lấy biệt danh máy chủ từ các cửa sổ thành viên, kết quả tìm kiếm và chunk đang
được giữ trong bộ nhớ có giới hạn, kể cả người nằm ngoài phần danh sách thành viên đang hiển thị.
Tên được chọn đúng theo máy chủ của kênh; khi không có biệt danh thì dùng tên hiển thị toàn cục
hoặc tên người dùng đã được giao thức cung cấp. Việc xóa biệt danh không giữ lại tên cũ từ cache.
Đây là bản sửa cho [issue #1044](https://github.com/ViceVerse-cz/Serein/issues/1044).

## Cài đặt mic và chuyển kênh thoại

Trong cài đặt **Giọng nói & video**, có thể bật **Tự động điều chỉnh độ nhạy đầu vào** và
**Nhận diện giọng nói nâng cao**. Ngưỡng thủ công được giữ lại khi chuyển sang chế độ tự động.
Nhận diện nâng cao dùng xác suất giọng nói từ RNNoise trên worker âm thanh; đây không phải
cam kết dùng cùng thuật toán độc quyền của Discord. Hai tính năng mới mặc định tắt để giữ
cấu hình thu mic cũ; preset Studio bỏ qua xử lý và ngưỡng đầu vào.

**Cảnh báo không phát hiện âm thanh** hiện thông báo sau 15 giây gần như im lặng khi đang
trong cuộc gọi với mic bật. Cảnh báo không tự mở mic, ghi âm hoặc gửi dữ liệu bổ sung; tắt mic,
không giữ phím nhấn để nói hoặc thiết bị chưa sẵn sàng sẽ ngừng theo dõi. Thông báo này là gợi ý
kiểm tra đầu vào, không kết luận mic hỏng. Có thể tắt trong cài đặt.

**Xác nhận chuyển kênh thoại** mặc định bật. Khi tắt, ứng dụng vẫn chờ kết thúc kết nối cũ
trước khi bắt đầu cuộc gọi mới. Các lựa chọn được lưu trên thiết bị. Đã kiểm tra logic ngưỡng
bằng dữ liệu tổng hợp; chất lượng nhận giọng nói, tiếng ồn và chi phí CPU với mic thật chưa đo.

## Emoji trên màn hình DPI cao

Port [PR upstream #1040](https://github.com/ViceVerse-cz/Serein/pull/1040) của Neur0nz,
head `617e449446750db0a9ff4cb56dc84c1150c42b0d`. Emoji Unicode dùng SVG Twemoji đã có trong
app, được raster hóa theo kích thước pixel thực trên worker nền. Không tải emoji Unicode
từ dịch vụ khác. Cache texture dùng tối đa 16 MiB, hàng đợi có giới hạn và giải phóng ảnh
không dùng; emoji máy chủ chọn kích thước CDN phù hợp với DPI. Nếu ảnh lỗi, app giữ phần
dự phòng thay vì chặn giao diện. Chưa xác minh trực tiếp chất lượng hiển thị native ở DPI cao
hoặc các kích thước phản hồi của CDN Discord trên máy phát triển này.

## Hồ sơ cá nhân ở góc dưới trái

Nhấp vào tài khoản để xem banner, avatar, bio, huy hiệu, tag và hoạt động có trong dữ liệu
hồ sơ hiện tại. Popout dùng cùng cơ chế tải ảnh, chủ đề và phần trang trí được app hỗ trợ
như hồ sơ người dùng; vẫn tuân theo tùy chọn ẩn mỹ phẩm và xác nhận liên kết ngoài.
Phần thông tin dài cuộn trong khu vực riêng để giữ thao tác trạng thái và đổi tài khoản.
Nếu Discord chưa cung cấp thông tin hoặc tải ảnh lỗi, app giữ nội dung dự phòng.

## Độ rộng danh sách thành viên

Kéo đường phân cách ở mép trái của danh sách thành viên bên phải để đổi độ rộng từ 190 đến
360 px. Lựa chọn được lưu trên thiết bị; thu nhỏ cửa sổ chỉ giới hạn độ rộng tạm thời và vẫn
chừa chỗ cho phần chat. Đặt lại bố cục khôi phục độ rộng mặc định 240 px.

Việc lưu lựa chọn nâng schema cache cục bộ từ 29 lên 30 và giữ các cài đặt hiện tại.
Các bản cũ chỉ đọc schema 29 sẽ từ chối mở cache đã nâng cấp; lưu ý điều này nếu quay lại
EXE cũ. Dữ liệu trên Discord không bị thay đổi.

## Kiểm tra issue âm thanh #1041

[Issue #1041](https://github.com/ViceVerse-cz/Serein/issues/1041) có cơ sở trong mã Linux hiện tại:
hàm `OwnApplication::allows()` vẫn chấp nhận PID `1995`, binary `pipewire`, tên
`cpal-pulseaudio-27190` khi PID của Serein là `27190`. Đã chạy chính hàm này với dữ liệu tổng hợp
trong một harness Rust/Wasm không dùng thiết bị âm thanh. Trường hợp native được chấp nhận,
trường hợp binary `serein` bị loại và ứng dụng game vẫn được chấp nhận.

Windows dùng capture riêng: loại cây tiến trình Serein hoặc lọc session theo PID/cây tiến trình.
Chưa có bằng chứng đường lỗi Linux này xảy ra trên Windows, cũng chưa thử cuộc gọi thật.
Theo yêu cầu chỉ hỗ trợ Windows, đợt này không thay đổi phần thu âm Linux.

## Tự đồng bộ upstream

Workflow **Sync upstream** kiểm tra `ViceVerse-cz/Serein:main` mỗi 6 giờ. Có thể chạy ngay
bằng **Actions → Sync upstream → Run workflow** trên `main`.

Nếu có commit mới, GitHub merge vào `main` của fork và giữ các commit riêng. Không dùng
force push, reset hoặc ghi đè fork bằng upstream. Khi có xung đột, workflow dừng và báo lỗi
để chủ fork giải quyết; không tự chọn bỏ một phía của xung đột.

Sau khi sync, workflow chủ động yêu cầu **Serein Fork CI**, vì các thay đổi do `GITHUB_TOKEN`
tạo không tự kích hoạt workflow `push`. Khi toàn bộ CI qua, **Windows Release** tự build EXE
và publish bản nightly của đúng commit đã kiểm tra. Nếu không có commit mới và CI đã được
yêu cầu cho commit hiện tại, không chạy lại CI hoặc tạo release khác.

Nếu merge đã xong nhưng gửi yêu cầu CI bị lỗi, lượt sync tiếp theo sẽ thử gửi lại yêu cầu.
Nếu CI đã chạy và thất bại, sửa lỗi hoặc chạy lại CI trong Actions; workflow không lặp build
cùng commit lỗi mỗi 6 giờ. Theo dõi lỗi/xung đột tại tab Actions của fork.

Lịch GitHub Actions có thể chạy trễ và có thể bị tắt khi repo công khai lâu không hoạt động.
Chỉ đồng bộ code nhánh `main`; không sao chép binary/Release của upstream.

## CI/CD

- CI native chỉ dùng runner Windows x64.
- **Serein Fork CI** chạy kiểm tra một lần trên mỗi thay đổi vào `main`; không build gói release.
- **Windows Release** tự chạy sau khi toàn bộ CI của `main` qua, dùng đúng SHA đã kiểm tra.
  CI đã cũ hơn `main` hiện tại sẽ không tạo release. Build thủ công/tag vẫn chạy kiểm tra.
  CI và Release giữ cache Rust riêng để lưu được cả dependency debug lẫn bản tối ưu;
  Release dùng lại khóa cache của các bản Windows trước đó.
- Để phát hành bản ổn định, chạy workflow từ `main` qua **Actions → Windows Release → Run workflow**,
  nhập phiên bản SemVer chưa dùng (ví dụ `0.1.1`), hoặc push tag `v0.1.1`.
- Workflow kiểm tra demo, chạy `cargo xtask check`, đóng gói production có voice và không có demo,
  rồi tải bộ cài, EXE, ZIP và `SHA256SUMS.txt` lên Release.
- Job build chỉ có quyền đọc; job publish dùng `GITHUB_TOKEN` với `contents: write`.
  Sync dùng token tích hợp với `contents: write` và `actions: write`; không cần PAT hoặc khóa ký
  macOS trong cấu hình thông thường. Nếu chính sách GitHub chặn merge, workflow báo lỗi thay vì
  tìm cách vượt quyền. Workflow release đa nền tảng của upstream không chạy trong fork.
- Updater của bản fork chỉ nhận gói từ `Lynx-1ST/Serein`. Nightly thuộc kênh cập nhật nightly;
  bản ổn định thuộc kênh production. Tên ZIP và định dạng checksum giữ tương thích updater.
  Các nút GitHub/báo lỗi trong trang Updates cũng trỏ về fork này.

Các EXE chưa được ký mã. Chưa xác minh đăng nhập, cuộc gọi, stream thật hoặc giao diện native
trên máy phát triển này; kiểm thử tổng hợp không chứng minh tương thích Discord thực tế.

Voice participants are ordered with active streamers first, then the displayed server nickname/name case-insensitively, with user ID breaking ties. This is a deterministic Discord-like order, not a claim of identical locale collation.

While sharing, the stage's three-dot button opens quality settings. Apply retires the old capture and waits for stream-stop acknowledgement before restarting; the voice call stays connected. Leaving the call or losing stream permission cancels the queued restart. Right-clicking a message link offers Copy link (including masked and RTL links). Windows external links use the OS protocol association directly rather than parsing a browser command template.

CI avoids a second full check after a PR merge only when main and the explicitly checked PR head have identical Git trees. The same fork's CI must have completed every required job successfully within 24 hours. Direct/upstream pushes, different merge trees, missing/skipped checks or unavailable API evidence run the full suite. The format preflight gates expensive jobs; Windows Release still builds the exact verified main commit with its separate optimized cache.
