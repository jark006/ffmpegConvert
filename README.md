# ffmpegConvert

### 使用 ffmpeg 给视频批量转码

需下载 **ffmpeg** ( [https://www.gyan.dev/ffmpeg/builds/](https://www.gyan.dev/ffmpeg/builds/) )，然后配置系统变量Path，或者将 `ffmpeg.exe` 直接放到本程序同一目录中。若使用 NVIDIA NVENC 预置，需确保 ffmpeg 支持 `hevc_nvenc` / `av1_nvenc`，并已安装可用的 NVIDIA 驱动。

内置可选的的转码目标

1. H265 (libx265)    CPU编码, 编码速度较慢
2. H265 (hevc_amf)   AMD GPU硬件加速编码, 编码速度快，但画质一般
3. H265 (hevc_nvenc) NVIDIA GPU硬件加速编码, 存储优化, 适合 RTX 50系
4. AV1  (av1_nvenc)  NVIDIA GPU硬件加速编码, 存储优化, 适合 RTX 50系
5. AV1  (av1_nvenc)  NVIDIA GPU硬件加速编码, 快速存储优化
6. AV1  (libsvtav1)  CPU编码, 速度较慢, 适合 动画/游戏/屏幕录制
7. AV1  (libsvtav1)  CPU编码, 速度较慢, 适合 实拍电影/剧集 (film-grain 合成颗粒)

### 自定义转码参数

可在程序文件旁，新建和程序同名的 `ffmpegConvert.txt`，填入如下格式文本新增配置。
每一行由两个“#”字符分割，第一部分是编码参数，第二部分是输出文件名称的附加后缀，第三部分是该条参数的说明。

```sh
-c:a libopus -c:v libx265 -crf 23 -preset slow # _H265 # H265 (libx265)   CPU编码, 编码速度较慢
-c:a libopus -c:v hevc_amf -quality quality -rc cqp -qp_i 22 -qp_p 22 # _H265 # H265 (hevc_amf)  AMD GPU硬件加速编码, 编码速度速度快，但画质一般
// 可继续添加 ...
```

### 软件使用方法

下载本软件：[https://github.com/jark006/ffmpegConvert/releases](https://github.com/jark006/ffmpegConvert/releases)

然后选中一个或多个 **视频文件或文件夹** `拖到本软件图标上` 即可，即用本软件打开拖过来的那些文件或文件夹。转码输出位置和原视频文件相同。

输入序号 `1 或 2 或 ...` 则对应以上转码目标，转码完成则正常退出程序。

若输入负数的序号 `-1 或 -2 或 ...` 则转码完成后，将自动关机 (30秒后关机)。