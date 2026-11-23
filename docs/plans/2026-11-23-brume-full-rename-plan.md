# Kế hoạch: đổi toàn bộ Serein → Brume

Tài liệu này dành cho người/agent thực thi. Viết để đọc một lần là làm được, không cần hỏi lại.

Repo: `Lynx-1ST/Serein` (đã rời mạng lưới fork, là repo độc lập).
Sản phẩm: client Discord viết bằng Rust + egui, Windows x64, giao diện tiếng Việt.

---

## 0. Bối cảnh bắt buộc đọc

Một phần việc **đã làm rồi** trong commit `f5e68692` (PR #29, đã merge vào `main`):

- `apps/desktop/src/main.rs` — tên ứng dụng `"Serein"` → `"Brume"`
- `packaging/windows/installer.nsi` — `PRODUCT_NAME`, `PRODUCT_PUBLISHER`, `InstallDir`, các thông báo
- `.github/workflows/windows-release.yml` — tiêu đề release → `"Brume $env:VERSION (Windows)"`
- `README.md`, `docs/windows-fork.md`
- `assets/brand/` — đã có `brume.svg`, `brume-flat.svg`, `brume-mark.svg`

**Commit đó chứa một lỗi phải sửa trước tiên** — xem mục 1.

Còn lại khoảng **410 file** có chuỗi "serein" (không phân biệt hoa thường).

---

## 1. LỖI PHẢI SỬA TRƯỚC: bộ cài không nhận ra bản cũ

`packaging/windows/installer.nsi:45`

```nsis
InstallDirRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "InstallLocation"
```

Bộ cài tìm bản đã cài bằng khóa registry **đặt tên theo `PRODUCT_NAME`**. PR #29 đổi
`PRODUCT_NAME` thành `Brume`, nên bộ cài mới sẽ tìm `...\Uninstall\Brume` — không tồn tại ở
bất kỳ máy nào. Hậu quả: cài **song song** vào `%LOCALAPPDATA%\Programs\Brume`, để lại bản cũ
trên đĩa, hai mục trong Apps & features, và bản cũ vẫn giữ entry tự khởi động.

Đã xác minh trên máy chủ dự án:

```
HKCU\...\Uninstall\Serein  →  InstallLocation = D:\Program Files\Serein, Version = 0.1.0-nightly.20261123.83
HKCU\...\Uninstall\Brume   →  không tồn tại
```

**Cách sửa:** tách định danh khỏi tên hiển thị.

```nsis
!define PRODUCT_ID   "Serein"   ; dùng cho khóa registry và dò bản cũ — GIỮ NGUYÊN ở bước này
!define PRODUCT_NAME "Brume"    ; mọi chữ người dùng nhìn thấy
```

Dùng `PRODUCT_ID` cho `InstallDirRegKey` và toàn bộ đường dẫn
`Software\Microsoft\Windows\CurrentVersion\Uninstall\...` (các dòng 45, 198–205).
Dùng `PRODUCT_NAME` cho `Name`, Start Menu, `MUI_FINISHPAGE_RUN_TEXT`, mọi `MessageBox`.

Mục 5 sẽ đổi `PRODUCT_ID` sang `Brume` kèm bước di trú. **Không** làm gộp hai việc.

---

## 2. Bốn thứ sẽ gãy nếu đổi tên mà không di trú

Đây là phần một agent bắt đầu từ đầu sẽ bỏ sót. Mỗi mục đều đã kiểm trong mã nguồn.

### 2.1 Updater tìm file theo tên cố định — nghiêm trọng nhất

`apps/desktop/src/updater.rs:536-553`, hàm `asset_name()`:

```rust
format!("serein-{tag}-Linux-X64.AppImage")
// và dạng tương tự cho Windows/macOS
```

Mọi bản **đã cài** đều chứa updater tìm file tên `serein-…`. Nếu release mới chỉ có
`brume-…` thì chúng **không bao giờ cập nhật được nữa** và người dùng phải tự tải tay.
Đây là lỗi không sửa ngược được sau khi đã phát hành.

**Giải pháp:** trong giai đoạn chuyển tiếp, bước publish **tải lên cả hai tên** cho cùng một
tệp. Bản cũ tìm thấy `serein-…`, bản mới tìm thấy `brume-…`. Giữ ít nhất 10 bản phát hành
rồi mới bỏ tên cũ.

### 2.2 Thư mục dữ liệu

`crates/local-store/src/lib.rs`

```
269:  dirs::data_local_dir().map(|root| root.join("serein-development"))   // bản build từ source
273:  dirs::data_local_dir().map(|root| root.join("serein"))               // bản cài
296:  resolve_data_dir(std::env::var_os("SEREIN_DATA_DIR"), default_data_dir())
1768: let configured = default.join("serein-development");
```

Đổi tên thư mục mà không di trú = mất toàn bộ cache, cấu hình và trạng thái đã đọc.

**Giải pháp:** lúc khởi động, nếu thư mục mới chưa có và thư mục cũ có thì **đổi tên** thư mục
cũ thành mới. Một lần, không chép, không xóa. Nếu đổi tên thất bại thì dùng tiếp thư mục cũ
và ghi log, tuyệt đối không tạo thư mục trắng.

Giữ `SEREIN_DATA_DIR` được chấp nhận song song với `BRUME_DATA_DIR` (biến mới ưu tiên hơn).

### 2.3 Entry tự khởi động Windows

`crates/platform/src/startup.rs:49`

```rust
const VALUE: PCWSTR = w!("Serein");
```

Và `load()` so khớp **chuỗi lệnh đầy đủ** với đường dẫn exe hiện tại
(`parse_command`, dòng 92–102). Đổi tên giá trị mà không di trú thì tự khởi động im lặng
ngừng hoạt động và giá trị cũ nằm lại vĩnh viễn.

**Giải pháp:** lúc khởi động, nếu giá trị `Serein` tồn tại và chuỗi lệnh của nó trỏ đúng vào
exe của mình thì ghi giá trị `Brume` tương đương rồi xóa giá trị `Serein`.

### 2.4 Thư mục cài và khóa gỡ cài đặt

Xem mục 1. Khi đổi `PRODUCT_ID`, bộ cài phải: đọc `...\Uninstall\Serein`, nếu có thì cài đè
vào đúng `InstallLocation` đó, viết khóa mới `...\Uninstall\Brume`, rồi **xóa khóa cũ** để
Apps & features không hiện hai mục.

---

## 3. Thứ tự thực hiện

Mỗi bước là một commit riêng, một PR riêng, chờ CI xanh rồi mới sang bước sau.

| # | Việc | Rủi ro |
|---|---|---|
| 1 | Sửa `installer.nsi` theo mục 1 (`PRODUCT_ID` = `Serein`) | thấp |
| 2 | Publish asset dưới **cả hai tên** trong `windows-release.yml` | thấp |
| 3 | Di trú thư mục dữ liệu (2.2) | trung bình |
| 4 | Di trú entry tự khởi động (2.3) | trung bình |
| 5 | Đổi `PRODUCT_ID` → `Brume` kèm xóa khóa cũ (2.4) | cao |
| 6 | Đổi `asset_name()` sang `brume-…` | cao |
| 7 | Quét chuỗi còn lại (mục 4) | thấp |
| 8 | Icon và tài nguyên nhị phân (mục 6) | thấp |

Bước 5 và 6 chỉ làm **sau khi** bước 2, 3, 4 đã phát hành và chạy ổn ít nhất vài bản nightly.
Lý do: chúng dựa vào việc người dùng đã chạy qua một bản có mã di trú.

---

## 4. Quét chuỗi còn lại

Sau khi xong di trú, đổi nốt phần còn lại. Những chỗ đã biết:

| Vị trí | Nội dung |
|---|---|
| `apps/desktop/src/audio.rs:175-176` | `"Audio worker stopped; restart Serein"` |
| `apps/desktop/src/font_import.rs:173,176` | khóa font `"Serein Custom"` |
| `apps/desktop/src/extension_bridge.rs:2229,2340` | `"Serein contributors"` |
| `apps/desktop/examples/profile_preview.rs:935` | `"Serein · offline profile preview"` |
| `apps/desktop/src/camera_gateway_demo.rs:123` | `"Serein synthetic camera Gateway replay"` |
| `apps/desktop/examples/extension_settings.rs:81` | `"Make Serein yours"` |
| `crates/platform/src/app_menu.rs:110` | `#[name = "SereinMenuTarget"]` (macOS) |
| `crates/platform/src/tray/macos.rs:27` | `#[name = "SereinTrayTarget"]` (macOS) |
| `.github/ISSUE_TEMPLATE/*.yml` | mẫu báo lỗi |
| `docs/**` | tài liệu |

Hai mục macOS là tên lớp Objective-C đăng ký lúc chạy; đổi được nhưng phải đổi đồng bộ cả
chuỗi lẫn nơi tra cứu, và không kiểm được trên Windows.

Lệnh tìm phần còn sót:

```powershell
git grep -rin "serein" -- . | Where-Object { $_ -notmatch "LICENSE|THIRD_PARTY|docs/plans" }
```

**Không đổi:** `LICENSE-MIT`, `LICENSE-APACHE`, `THIRD_PARTY_NOTICES.md`, và mọi dòng ghi công
tác giả gốc. Giấy phép MIT/Apache cho phép đổi tên sản phẩm nhưng **bắt buộc giữ thông báo bản
quyền**, và không được ngụ ý dự án gốc chứng thực cho bản này.

---

## 5. Một quyết định còn bỏ ngỏ

`apps/desktop/src/extensions.rs:25`

```rust
"https://raw.githubusercontent.com/ViceVerse-cz/Serein/main/extensions/catalog.json";
```

Repo này đã tách khỏi upstream nhưng vẫn **tải danh mục extension từ repo của họ**. Đây là dư
âm fork có ý nghĩa chức năng, không phải chuyện thương hiệu. Hai lựa chọn: trỏ về repo này
(phải tự duy trì danh mục) hoặc giữ nguyên (tiếp tục phụ thuộc). Chủ dự án chưa quyết. PR #29
đã thêm chú thích đánh dấu chỗ này.

---

## 6. Icon và tài nguyên nhị phân

Mark mới đã có ở `assets/brand/brume-mark.svg`: bốn dải sương **khoét rỗng** khỏi khối bo tròn
`rx=60`, độ dài lệch nhau. Phẳng, không viền, không bóng đổ.

Ràng buộc bắt buộc: mark phải đọc được khi **chỉ còn một màu**. `brume-mark.svg` là bóng đặc,
được tô màu lúc vẽ, và bản tray chỉ dùng kênh alpha. Mọi ý tưởng phụ thuộc gradient đều hỏng.

Phải dựng lại:

- `packaging/windows/Serein.ico` → `Brume.ico` (nhiều kích thước, phải có 16 và 32 px)
- `apps/desktop/build.rs:21` trỏ tới đường dẫn `.ico` — cập nhật theo
- `packaging/windows/serein.rc` → đổi tên và nội dung
- `packaging/macos/Serein.icns`, `Serein.icon`; `packaging/linux/hicolor/*`
- atlas icon: tên `serein-mark` → `brume-mark`, sinh bằng `tools/generate-icons.py`
- `assets/brand/README.md` — mô tả lại các tệp

Giữ nguyên bảng màu `#8BBEFF → #506DDC → #2D247C` và màu nhấn giao diện `#1a72e8`, nên
không phải kiểm tra lại độ tương phản.

---

## 7. Cách kiểm tra

```powershell
cargo fmt --all --check
cargo xtask check          # biên dịch + clippy -D warnings + test toàn workspace
cargo xtask package        # bắt buộc với thay đổi runtime/đóng gói
```

CI chạy `cargo clippy --workspace --all-targets --locked -- -D warnings`, tức biên dịch cả
test với mọi cảnh báo coi là lỗi. CI **không** chạy bước publish release, nên sai sót trong
khối ghi chú phát hành chỉ lộ ra lúc phát hành thật.

Đẩy lên `main` sẽ kích hoạt CI; CI xanh sẽ kích hoạt Windows Release và sinh bản nightly.

**Lưu ý về `workflow_run`:** bước publish chạy **định nghĩa workflow lấy từ nhánh mặc định tại
thời điểm kích hoạt**, không phải từ commit đang build. Nên mọi thay đổi trong
`windows-release.yml` chỉ có hiệu lực từ bản phát hành **kế tiếp**. Đã quan sát thật: bản `.83`
build mã đã đổi tên nhưng vẫn mang tiêu đề `Serein` vì workflow cũ chạy.

Không kiểm được bằng tự động: hình dạng icon ở 16 px, và việc di trú có thật sự chạy đúng trên
máy đã cài bản cũ. Hai thứ này phải cài thử và nhìn.

---

## 8. Việc ngoài lề đáng dọn

Trên máy chủ dự án có hai vị trí cài khác nhau:

```
entry tự khởi động  →  D:\App Center\Serein\serein.exe
khóa gỡ cài đặt     →  D:\Program Files\Serein
```

Nhiều khả năng là hai bản cài chồng từ trước, không liên quan việc đổi tên. Nên dọn trước khi
chạy bước 5, nếu không sẽ rất khó phân biệt lỗi di trú với lỗi có sẵn.
