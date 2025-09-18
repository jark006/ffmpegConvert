# ffmpegConvert

### 使用 ffmpeg 给视频批量转到 H265/AV1 编码格式

需安装 **ffmpeg** 并配置到环境变量: [https://www.gyan.dev/ffmpeg/builds/](https://www.gyan.dev/ffmpeg/builds/)

目前可选的的转码目标

1. H265 (libx265)
2. H265 (hevc_amf)
3. AV1  (libsvtav1)

实际命令行
```sh
ffmpeg -hide_banner -i "input.mp4" -c:a copy -c:v libx265 -crf 23 -preset slow -y "output_H265.mp4"
ffmpeg -hide_banner -i "input.mp4" -c:a copy -c:v hevc_amf -quality quality -rc cqp -qp_i 22 -qp_p 22 "output_H265.mp4"
ffmpeg -hide_banner -i "input.mp4" -c:a copy -c:v libsvtav1 -crf 28 -preset 5 "output_AV1.mp4"
```

### 使用方法

下载本软件：[https://github.com/jark006/ffmpegConvert/releases](https://github.com/jark006/ffmpegConvert/releases)

然后把视频文件或文件夹拖到本软件图标上即可，支持选中多个拖过来。