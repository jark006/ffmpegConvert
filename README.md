# ffmpegConvert

### 使用 ffmpeg 给视频批量转码

需下载 **ffmpeg** ( [https://www.gyan.dev/ffmpeg/builds/](https://www.gyan.dev/ffmpeg/builds/) )，然后配置系统变量Path，或者将 `ffmpeg.exe` 直接放到本程序同一目录中。

内置可选的的转码目标

1. H265 (libx265)   CPU编码, 编码速度较慢
2. H265 (hevc_amf)  AMD GPU硬件加速编码, 编码速度速度快，但画质一般
3. AV1  (libsvtav1) CPU编码, 编码速度很慢，压缩率高
4. AV1  (libaom-av1) CPU编码, 编码速度最慢，压缩率最高

### 实际命令行参数

```sh
ffmpeg -hide_banner -i "input.mp4" -c:a aac -c:v libx265 -crf 23 -preset slow -y "output_H265.mp4"
ffmpeg -hide_banner -i "input.mp4" -c:a aac -c:v hevc_amf -quality quality -rc cqp -qp_i 22 -qp_p 22 -y "output_H265.mp4"
ffmpeg -hide_banner -i "input.mp4" -c:a aac -c:v libsvtav1 -crf 28 -preset 5 -y "output_AV1.mp4"
ffmpeg -hide_banner -i "input.mp4" -c:a aac -c:v libaom-av1 -crf 28 -preset 8 -y "output_AV1.mp4"
```

### 自定义转码参数

可在程序文件旁，新建和程序同名的 `ffmpegConvert.txt`，填入如下格式文本新增配置。
每一行由两个“#”字符分割，第一部分是编码参数，第二部分是输出文件名称的附加后缀，第三部分是该条参数的说明。

```sh
-c:a aac -c:v libx265 -crf 23 -preset slow # _H265 # H265 (libx265)   CPU编码, 编码速度较慢
-c:a aac -c:v hevc_amf -quality quality -rc cqp -qp_i 22 -qp_p 22 # _H265 # H265 (hevc_amf)  AMD GPU硬件加速编码, 编码速度速度快，但画质一般
// 可继续添加 ...
```

### 软件使用方法

下载本软件：[https://github.com/jark006/ffmpegConvert/releases](https://github.com/jark006/ffmpegConvert/releases)

然后选中一个或多个 **视频文件或文件夹** `拖到本软件图标上` 即可，即用本软件打开拖过来的那些文件或文件夹。转码输出位置和原视频文件相同。

输入序号 `1 或 2 或 ...` 则对应以上转码目标，转码完成则正常退出程序。

若输入负数的序号 `-1 或 -2 或 ...` 则转码完成后，将自动关机 (30秒后关机)。