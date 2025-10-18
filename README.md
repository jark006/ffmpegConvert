# ffmpegConvert

### 使用 ffmpeg 给视频批量转到 H265/AV1 编码格式

需下载 **ffmpeg** 并配置到环境变量: [https://www.gyan.dev/ffmpeg/builds/](https://www.gyan.dev/ffmpeg/builds/)

目前可选的的转码目标

1. H265 (libx265)   CPU编码, 编码速度较慢，但画质较好
2. H265 (hevc_amf)  AMD GPU硬件加速编码, 编码速度速度快，但画质一般
3. AV1  (libsvtav1) CPU编码, 编码速度非常慢，但画质较好，压缩率高

### 实际命令行参数

```sh
ffmpeg -hide_banner -i "input.mp4" -c:a aac -c:v libx265 -crf 23 -preset slow -y "output_H265.mp4"
ffmpeg -hide_banner -i "input.mp4" -c:a aac -c:v hevc_amf -quality quality -rc cqp -qp_i 22 -qp_p 22 -y "output_H265.mp4"
ffmpeg -hide_banner -i "input.mp4" -c:a aac -c:v libsvtav1 -crf 28 -preset 5 -y "output_AV1.mp4"
```
**为了软件快捷使用，以上参数不可更改。若需要其他参数，则需编辑源码，重新编译**

### 使用方法

下载本软件：[https://github.com/jark006/ffmpegConvert/releases](https://github.com/jark006/ffmpegConvert/releases)

然后把 **视频文件或文件夹** `拖到本软件图标上` 即可，即用本软件打开拖过来的那些文件或文件夹，支持选中多个拖过来。转码输出位置和原视频文件相同。

输入 `1 或 2 或 3` 则对应以上转码目标，转码完成则正常退出程序。

输入 `11 或 22 或 33` 则对应以上转码目标，但转码完成后将自动关机 (30秒后关机)。