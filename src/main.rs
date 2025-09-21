use std::env;
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!(concat!(
            "请提供至少一个文件或文件夹路径作为参数\n\n",
            "本软件用于给视频批量转到H265/AV1编码格式，请把视频文件或文件夹拖到本软件图标上即可，支持多个一起拖拽\n\n",
            "本软件依赖 ffmpeg，需确保 ffmpeg.exe 已安装并添加到系统环境变量中\n\n",
            "ffmpeg.exe 下载地址: https://www.gyan.dev/ffmpeg/builds/\n\n"
        ));
        sleep(Duration::from_secs(600)); // 10分钟后自动关闭
        std::process::exit(1);
    }

    println!(
        "选择要转码的目标编码类型:
1. H265 (libx265)   CPU编码, 较慢
2. H265 (hevc_amf)  AMD GPU硬件加速编码, 速度快
3. AV1  (libsvtav1) CPU编码, 非常慢

输入1或2或3则对应以上转码目标，转码完成则正常退出程序。
如果输入11或22或33则对应以上转码目标，但转码完成后将自动关机 (30秒后关机)。\n"
    );

    let mut select_type = 0;
    let mut shutdown_when_done = false;

    while select_type == 0 {
        print!("请输入目标编码类型 (1/2/3): ");
        std::io::stdout().flush().unwrap(); // 确保提示立即显示

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            let input = input.trim().to_lowercase();

            select_type = match input.as_str() {
                "1" | "x265" => 1,
                "2" | "amf" => 2,
                "3" | "av1" => 3,
                "11" => 11,
                "22" => 22,
                "33" => 33,
                _ => 0,
            };

            if select_type == 11 {
                select_type = 1;
                shutdown_when_done = true;
            }else if select_type == 22 {
                select_type = 2;
                shutdown_when_done = true;
            }else if select_type == 33 {
                select_type = 3;
                shutdown_when_done = true;
            }
        }
    }

    if shutdown_when_done {
        println!("提示: 转码完成后，将倒计时30秒关机。\n");
    }

    let video_exts = [
        "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "ts", "mpeg", "mpg", "3gp", "rm",
        "rmvb",
    ];
    let mut video_files = Vec::new();

    for arg in args {
        let path = Path::new(&arg);

        if !path.exists() {
            eprintln!("路径不存在: {}", arg);
            continue;
        }

        if path.is_file() {
            if is_video_file(path, &video_exts) {
                if let Ok(absolute_path) = path.canonicalize() {
                    video_files.push(absolute_path);
                }
            } else {
                eprintln!("跳过非视频文件: {}", arg);
            }
        } else if path.is_dir() {
            find_video_files(path, &video_exts, &mut video_files);
        }
    }
    
    // 过滤掉 _h265 和 _av1 结尾的文件
    video_files.retain(|p| {
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            let lower_stem = stem.to_lowercase();
            !(lower_stem.ends_with("_h265") || lower_stem.ends_with("_av1"))
        } else {
            true
        }
    });

    println!("\n找到 {} 个视频文件需要处理", video_files.len());
    if video_files.is_empty() {
        sleep(Duration::from_secs(10)); // 10秒后自动关闭
        return;
    }

    let mut idx = 1;
    for video_path in video_files.iter() {
        println!("{:<2}: {}", idx, video_path.to_string_lossy()[4..].to_string());
        idx += 1;
    }
    println!();

    let total_files = video_files.len();
    let mut file_count = 1;

    for video_path in video_files {
        let video_path = video_path.to_string_lossy();
        let video_path: &str = &video_path[4..]; // 去掉前面的 "\\?\" 之类的

        println!(
            "[{}/{}] [{}%] 处理中: {}",
            file_count,
            total_files,
            100 * file_count / total_files,
            video_path
        );

        let output_path = {
            let mut p = PathBuf::from(video_path);
            let file_stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
            let new_file_name = match select_type {
                1 | 2 => format!("{}_H265.mp4", file_stem),
                3 => format!("{}_AV1.mp4", file_stem),
                _ => format!("{}_output.mp4", file_stem),
            };
            p.set_file_name(new_file_name.replace("_H264", "").replace("_h264", ""));
            p
        };

        // 执行转码并显示进度
        if !transcode_with_progress(select_type, &video_path, &output_path) {
            eprintln!("\n处理失败: {}", video_path);
        } else {
            println!(); // 换行，为下一个文件的处理做准备
        }

        file_count += 1;
    }

    if shutdown_when_done {
        // shutdown.exe -s -t 30
        Command::new("shutdown.exe")
        .arg("-s")
        .arg("-t")
        .arg("30")
        .spawn()
        .expect("无法计划关机");
    }
}

fn transcode_with_progress(select_type: i32, input_path: &str, output_path: &PathBuf) -> bool {
    let mut child = match select_type {
        1 => Command::new("ffmpeg.exe")
            .arg("-hide_banner")
            .arg("-i")
            .arg(input_path)
            .arg("-c:a")
            .arg("aac")
            .arg("-c:v")
            .arg("libx265")
            .arg("-crf")
            .arg("23")
            .arg("-preset")
            .arg("slow")
            .arg("-y") // 覆盖输出文件
            .arg(output_path)
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .expect("无法启动 ffmpeg"),

        2 => Command::new("ffmpeg.exe")
            .arg("-hide_banner")
            .arg("-i")
            .arg(&input_path)
            .arg("-c:a")
            .arg("aac")
            .arg("-c:v")
            .arg("hevc_amf")
            .arg("-quality")
            .arg("quality")
            .arg("-rc")
            .arg("cqp")
            .arg("-qp_i")
            .arg("22")
            .arg("-qp_p")
            .arg("22")
            .arg("-y") // 覆盖输出文件
            .arg(&output_path)
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .expect("无法启动 ffmpeg"),

        3 => Command::new("ffmpeg.exe")
            .arg("-hide_banner")
            .arg("-i")
            .arg(&input_path)
            .arg("-c:a")
            .arg("aac")
            .arg("-c:v")
            .arg("libsvtav1")
            .arg("-crf")
            .arg("28")
            .arg("-preset")
            .arg("5")
            .arg("-y") // 覆盖输出文件
            .arg(&output_path)
            .stderr(Stdio::piped())
            .stdout(Stdio::null())
            .stdin(Stdio::null())
            .spawn()
            .expect("无法启动 ffmpeg"),

        _ => return false,
    };

    let stderr = child.stderr.take().expect("无法获取 stderr");
    let reader = BufReader::new(stderr);

    let mut total_duration: Option<Duration> = None;
    let mut buffer = String::new();

    //当前时间戳
    let start_timestamp = std::time::Instant::now();

    for byte in reader.bytes() {
        if let Ok(b) = byte {
            let ch = b as char;

            if ch == '\r' || ch == '\n' {
                if !buffer.is_empty() {
                    // 解析总时长
                    if total_duration.is_none() {
                        if let Some(duration) = parse_total_duration(&buffer) {
                            total_duration = Some(duration);
                        }
                    }

                    // 解析进度信息
                    if let Some(total) = total_duration {
                        if let Some(progress) = parse_progress(&buffer) {
                            let percentage: f64 = if total.as_secs() > 0 {
                                if progress.current_time.as_secs() == total.as_secs() {
                                    100.0
                                } else {
                                ((progress.current_time.as_secs() as f64)* 100.0) / (total.as_secs() as f64)
                                }
                            } else {
                                0.0
                            };

                            let elapsed_secs = (std::time::Instant::now() - start_timestamp).as_secs();

                            //根据已用时间和百分比计算估计剩余时间
                            let estimated_remaining = if elapsed_secs < 2 {
                                total.as_secs()
                            } else if percentage > 0.0 && percentage < 100.0 {
                                let remain_sec = (100.0 - percentage) * (elapsed_secs as f64) / percentage;
                                remain_sec as u64
                            } else if percentage == 100.0 {
                                0
                            } else {
                                total.as_secs()
                            };

                            let remain_str = if estimated_remaining > 0 {
                                format!("预估剩余时间:{}", format_duration(&Duration::from_secs(estimated_remaining)))
                            } else {
                                "已完成                ".to_string()
                            };

                            // 在同一行更新进度
                            print!(
                                "\r    [{:3.1}%] {} / {} 编码速度:{} 编码耗时:{} {}   ",
                                percentage,
                                format_duration(&progress.current_time),
                                format_duration(&total),
                                progress.speed_str,
                                format_duration(&Duration::from_secs(elapsed_secs)),
                                remain_str
                            );
                            std::io::stdout().flush().unwrap();
                        }
                    }
                }
                buffer.clear();
            } else {
                buffer.push(ch);
            }
        }
    }

    let status = child.wait().expect("子进程执行失败");
    let is_success = status.success();

    if is_success {
        // ffmpeg的进度输出可能达不到100%， 确保显示100%完成
        if let Some(total) = total_duration {
            let elapsed_secs = (std::time::Instant::now() - start_timestamp).as_secs();
            print!(
                "\r    [100%] 视频时长:{} 编码速度:{:1.1}x 编码耗时:{} 已完成                ",
                format_duration(&total),
                total.as_secs_f64() / (elapsed_secs as f64),
                format_duration(&Duration::from_secs(elapsed_secs))
            );
        }else{
            print!("\r    [100%]  ");
        }
        std::io::stdout().flush().unwrap();
    }

    is_success
}

fn parse_total_duration(line: &str) -> Option<Duration> {
    if let Some(start) = line.find("Duration: ") {
        let duration_str = &line[start + 10..];
        if let Some(comma_pos) = duration_str.find(',') {
            let time_str = &duration_str[..comma_pos];
            return parse_time_to_duration(time_str);
        }
    }
    None
}

struct ProgressInfo {
    current_time: Duration,
    speed_str: String,
}

fn parse_progress(line: &str) -> Option<ProgressInfo> {
    // 查找 time= 字段
    let time = if let Some(start) = line.find("time=") {
        let time_str = &line[start + 5..];
        if let Some(space_pos) = time_str.find(' ') {
            parse_time_to_duration(&time_str[..space_pos])
        } else {
            None
        }
    } else {
        None
    };

    // 提取 speed= 后到 x 字符（包含 x）
    let speed_str = if let Some(start) = line.find("speed=") {
        let speed_part = &line[start + 6..];
        if let Some(x_pos) = speed_part.find('x') {
            speed_part[..=x_pos].trim().to_string()
        } else {
            "0.0x  ".to_string()
        }
    } else {
        "0.0x  ".to_string()
    };

    // 如果speed_str过短，补齐空格
    let speed_str = if speed_str.len() < 6 {
        format!("{:<6}", speed_str)
    } else {
        speed_str
    };

    if let Some(time) = time {
        Some(ProgressInfo {
            current_time: time,
            speed_str: speed_str,
        })
    } else {
        None
    }
}

fn parse_time_to_duration(time_str: &str) -> Option<Duration> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 3 {
        let hours = parts[0].parse::<u64>().ok()?;
        let minutes = parts[1].parse::<u64>().ok()?;
        let seconds = parts[2].parse::<f64>().ok()?;

        let total_seconds = hours * 3600 + minutes * 60 + seconds as u64;
        let nanos = (seconds.fract() * 1_000_000_000.0) as u32;

        Some(Duration::new(total_seconds, nanos))
    } else {
        None
    }
}

fn format_duration(duration: &Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn is_video_file(path: &Path, exts: &[&str]) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| exts.iter().any(|&e| ext.eq_ignore_ascii_case(e)))
        .unwrap_or(false)
}

fn find_video_files(dir: &Path, exts: &[&str], results: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_video_files(&path, exts, results);
            } else if is_video_file(&path, exts) {
                if let Ok(absolute_path) = path.canonicalize() {
                    results.push(absolute_path);
                }
            }
        }
    }
}
