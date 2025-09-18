use winres;

fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("icon.ico")
    .set("InternalName", "ffmpegConvert.exe")
    .set("OriginalFilename", "ffmpegConvert.exe")
    .set("FileDescription", "视频批量转到H265/AV1编码格式")
    .set("LegalCopyright", "Copyright © 2025 JARK006")
    .set("ProductName", "ffmpegConvert")
    .set("CompanyName", "JARK006")
    .set_language(0x804); // 中文简体 - China
    res.compile().unwrap();
}