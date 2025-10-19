use std::env;
use std::ffi::OsStr;
use std::io::{BufReader, Read, Write};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;
use winapi::um::wincon::SetConsoleTitleW;
use std::fmt;
use winapi::shared::ntdef::HANDLE;

pub fn set_console_title(title: &str) -> bool {
    let wide: Vec<u16> = OsStr::new(title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe { SetConsoleTitleW(wide.as_ptr()) != 0 }
}

struct ConvertParameter {
    params: &'static str,
    subfix: &'static str,
    description: &'static str,
}

fn log_file_path() -> PathBuf {
    let mut p = env::current_exe().expect("无法获取可执行文件路径");
    p.set_extension("log");
    p
}

// 读取额外参数（从与可执行文件同名但扩展名为 .txt 的旁侧文件）
fn load_params_from_sidecar(convert_params: &mut Vec<ConvertParameter>) {
    if let Ok(mut sidecar) = env::current_exe() {
        sidecar.set_extension("txt");
        if !sidecar.exists() {
            return;
        }

        let content = match std::fs::read_to_string(&sidecar) {
            Ok(s) => s,
            Err(_) => return,
        };

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
                continue; // 跳过空行和注释行
            }

            // 按第一个 '#' 分割，前面是参数，中间是输出文件名称的附加后缀，后面是描述（可选）
            let parts: Vec<&str> = line.split('#').map(|s| s.trim()).collect();
            if parts.len() < 2 {
                continue; // 至少需要参数和输出后缀
            }

            let params_part = parts[0];
            let subfix_part = parts[1];
            let desc_part = if parts.len() > 2 { parts[2] } else { parts[0] };

            // 过滤无意义行：参数部分不能为空且应包含 '-'（简单判断）
            if params_part.is_empty() || !params_part.contains('-') {
                continue;
            }

            // 将动态字符串泄漏为 'static，方便与现有 ConvertParameter<'static> 兼容
            let params: &'static str = Box::leak(params_part.to_string().into_boxed_str());
            let subfix: &'static str = Box::leak(subfix_part.to_string().into_boxed_str());
            let description: &'static str = Box::leak(desc_part.to_string().into_boxed_str());

            convert_params.push(ConvertParameter { params, subfix, description });
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!(concat!(
            "请提供至少一个文件或文件夹路径作为参数\n\n",
            "本软件用于给视频批量转码，请把视频文件或文件夹拖到本软件图标上即可，支持多个一起拖拽\n\n",
            "本软件依赖 ffmpeg，需确保 ffmpeg.exe 位于本程序同一目录下，或者将其所在文件夹添加到系统环境变量中\n\n",
            "ffmpeg.exe 下载地址: https://www.gyan.dev/ffmpeg/builds/\n\n",
            "本软件开源免费，源码地址: https://github.com/JARK006/ffmpegConvert"
        ));
        sleep(Duration::from_secs(600)); // 10分钟后自动关闭
        std::process::exit(1);
    }

    let mut convert_params: Vec<ConvertParameter> = vec![
        ConvertParameter {
            params: "-c:a aac -c:v libx265 -crf 23 -preset slow",
            subfix: "_H265",
            description: "H265 (libx265)   CPU编码, 较慢",
        },
        ConvertParameter {
            params: "-c:a aac -c:v hevc_amf -quality quality -rc cqp -qp_i 22 -qp_p 22",
            subfix: "_H265",
            description: "H265 (hevc_amf)  AMD GPU硬件加速编码, 速度快",
        },
        ConvertParameter {
            params: "-c:a aac -c:v libsvtav1 -crf 28 -preset 4",
            subfix: "_AV1",
            description: "AV1  (libsvtav1) CPU编码, 非常慢",
        },
        ConvertParameter {
            params: "-c:a aac -c:v libaom-av1 -crf 28 -cpu-used 8 -b:v 0 -row-mt 1",
            subfix: "_AV1",
            description: "AV1  (libaom-av1) CPU编码, 最慢",
        },
    ];

    load_params_from_sidecar(&mut convert_params);

    println!("选择要转码的目标编码类型的序号，转码完成则正常退出程序。如果输入负数序号则转码完成后将自动关机 (30秒后关机)。\n");
    for (i, param) in convert_params.iter().enumerate() {
        println!("  {:<2}: {}", i + 1, param.description);
    }
    println!();

    let mut select_index = 0;
    let mut shutdown_when_done = false;

    while select_index <= 0 || select_index > (convert_params.len() as i32) {
        print!("请输入序号: ");
        std::io::stdout().flush().unwrap(); // 确保提示立即显示

        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_ok() {
            select_index = input.trim().parse::<i32>().unwrap_or(0);

            if select_index < 0 {
                select_index = -select_index;
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
        sleep(Duration::from_secs(2)); // 2秒后自动关闭
        return;
    }

    video_files.sort_by(|a, b| {
        let a_str = a.to_string_lossy();
        let b_str = b.to_string_lossy();
        natural_sort_rs::natural_cmp(&a_str, &b_str)
    });

    let mut idx = 1;
    for video_path in video_files.iter() {
        println!(
            "{:<2}: {}",
            idx,
            video_path.to_string_lossy()[4..].to_string()
        );
        idx += 1;
    }
    println!();

    let total_files = video_files.len();
    let mut file_count = 1;

    for video_path in video_files {
        let video_path = video_path.to_string_lossy();
        let video_path: &str = &video_path[4..]; // 去掉前面的 "\\?\" 之类的

        println!("[{}/{}] 处理中: {}", file_count, total_files, video_path);

        let output_path = {
            let mut p = PathBuf::from(video_path);
            let default_output_name = format!("output_{}", chrono::Local::now().format("%Y%m%d%H%M%S"));
            let file_stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(default_output_name.as_str());
            let new_file_name = format!("{}{}.mp4", file_stem, convert_params[(select_index - 1) as usize].subfix);            
            p.set_file_name(new_file_name.replace("_H264", "").replace("_h264", ""));
            p
        };

        // 执行转码并显示进度
        if !transcode_with_progress(
            &convert_params[(select_index - 1) as usize],
            &video_path,
            &output_path,
            &format!("[{}/{}]", file_count, total_files),
        ) {
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

fn transcode_with_progress(
    convert_params: &ConvertParameter,
    input_path: &str,
    output_path: &PathBuf,
    title_prefix: &str,
    ) -> bool {

    // 输出日志
    match std::fs::OpenOptions::new().create(true).append(true).open(log_file_path()) {
        Ok(mut f) => {
            let ts = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
            let _ = writeln!(f, "[{}] 输入: {}", ts, input_path);
        }
        Err(e) => {
            eprintln!("无法打开日志文件 {}: {}", log_file_path().display(), e);
        }
    }

    let mut child = Command::new("ffmpeg.exe")
        .arg("-hide_banner")
        .arg("-i")
        .arg(&input_path)
        .args(convert_params.params.split_whitespace())
        .arg("-y") // 覆盖输出文件
        .arg(&output_path)
        .stderr(Stdio::piped())
        .stdout(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .expect("无法启动 ffmpeg");

    let stderr = child.stderr.take().expect("无法获取 stderr");
    let reader = BufReader::new(stderr);

    let mut total_duration: Option<Duration> = None;
    let mut buffer = String::new();

    //当前时间戳
    let start_timestamp = std::time::Instant::now();

    let mut percent_int_last = -1;

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
                            let percentage: f64 = if total.as_millis() > 0 {
                                if progress.current_time == total {
                                    100.0
                                } else {
                                    ((progress.current_time.as_millis() as f64) * 100.0)
                                        / (total.as_millis() as f64)
                                }
                            } else {
                                0.0
                            };

                            let elapsed_millis =
                                (std::time::Instant::now() - start_timestamp).as_millis() as u64;

                            //根据已用时间和百分比计算估计剩余时间
                            let estimated_remaining_millis = if elapsed_millis < 1_000_000 {
                                total.as_millis() as u64
                            } else if percentage > 0.0 && percentage < 100.0 {
                                let remain_millis =
                                    (100.0 - percentage) * (elapsed_millis as f64) / percentage;
                                remain_millis as u64
                            } else if percentage == 100.0 {
                                0
                            } else {
                                total.as_millis() as u64
                            };

                            let remain_str = if estimated_remaining_millis > 0 {
                                format!(
                                    "剩余:{}",
                                    format_duration(&Duration::from_millis(estimated_remaining_millis))
                                )
                            } else {
                                "已完成                ".to_string()
                            };

                            // 在同一行更新进度
                            print!(
                                "\r    [{:3.1}%] {} / {} 速度:{} 用时:{} {}   ",
                                percentage,
                                format_duration(&progress.current_time),
                                format_duration(&total),
                                progress.speed_str,
                                format_duration(&Duration::from_millis(elapsed_millis)),
                                remain_str
                            );
                            std::io::stdout().flush().unwrap();

                            let percen_int = percentage as i32;
                            if percen_int != percent_int_last {
                                percent_int_last = percen_int;

                                set_console_title(&format!(
                                    "{} {}% {}",
                                    title_prefix,
                                    percen_int,
                                    Path::new(input_path)
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or(input_path)
                                ));
                            }
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
        let mut log_content = format!(
            "[{}] 输出: {}\n                      ",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            output_path.display()
        );

        let mut elapsed_secs = (std::time::Instant::now() - start_timestamp).as_secs();
        if elapsed_secs == 0 {
            elapsed_secs = 1;
        }

        // ffmpeg的进度输出可能达不到100%， 确保显示100%完成
        if let Some(total) = total_duration {
            print!(
                "\r    [100%] 视频时长:{} 速度:{:1.1}x 用时:{} 已完成                ",
                format_duration(&total),
                total.as_secs_f64() / (elapsed_secs as f64),
                format_duration(&Duration::from_secs(elapsed_secs))
            );

            log_content.push_str(&format!(
                "视频时长:{} 速度:{:1.1}x 用时:{}    ",
                format_duration(&total),
                total.as_secs_f64() / (elapsed_secs as f64),
                format_duration(&Duration::from_secs(elapsed_secs))
            ));
        } else {
            print!("\r    [100%]  ");

            log_content.push_str(&format!(
                "用时:{}    ", format_duration(&Duration::from_secs(elapsed_secs))
            ));
        }

        // 再输出文件体积对比，例如: 795.46 MB -> 389.43 MB (-51%)
        if let Ok(input_metadata) = std::fs::metadata(input_path) {
            if let Ok(output_metadata) = std::fs::metadata(output_path) {
                let input_size = input_metadata.len() as f64;
                let output_size = output_metadata.len() as f64;
                let reduction = if input_size > 0.0 {
                    100.0 * (output_size - input_size) / input_size
                } else {
                    0.0
                };

                fn format_size(size: f64) -> String {
                    if size >= 1_073_741_824.0 {
                        format!("{:.2} GB", size / 1_073_741_824.0)
                    } else if size >= 1_048_576.0 {
                        format!("{:.2} MB", size / 1_048_576.0)
                    } else if size >= 1024.0 {
                        format!("{:.2} KB", size / 1024.0)
                    } else {
                        format!("{:.2} B", size)
                    }
                }
                println!();

                log_content.push_str(&format!(
                    "{} -> {} ({:.1}%)",
                    format_size(input_size),
                    format_size(output_size),
                    reduction
                ));

                println!(
                    "    {} -> {} ({:.1}%)",
                    format_size(input_size),
                    format_size(output_size),
                    {
                        // 包装一个带有 Drop 的临时值，保证在 println 完成后恢复控制台颜色
                        struct ColorF64 {
                            val: f64,
                            handle: HANDLE,
                        }
                        impl fmt::Display for ColorF64 {
                            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                                // 保证以一位小数输出（与原来 {:.1} 一致）
                                write!(f, "{:.1}", self.val)
                            }
                        }
                        impl Drop for ColorF64 {
                            fn drop(&mut self) {
                                unsafe {
                                    // 恢复默认颜色（白色）
                                    let _ = winapi::um::wincon::SetConsoleTextAttribute(self.handle, 0x07);
                                }
                            }
                        }

                        let attr: u16 = if reduction > 0.0 {
                            0x0C // 明亮红色 (FOREGROUND_RED | FOREGROUND_INTENSITY)
                        } else if reduction < -20.0 {
                            0x0A // 明亮绿色 (FOREGROUND_GREEN | FOREGROUND_INTENSITY)
                        } else if reduction < 0.0 {
                            0x09 // 蓝色 (FOREGROUND_BLUE | FOREGROUND_INTENSITY)
                        } else {
                            0x07 // 默认
                        };

                        let h: HANDLE = unsafe { winapi::um::processenv::GetStdHandle(winapi::um::winbase::STD_OUTPUT_HANDLE) };
                        unsafe {
                            let _ = winapi::um::wincon::SetConsoleTextAttribute(h, attr);
                        }

                        ColorF64 { val: reduction, handle: h }
                    }
                );
            }
        }

        std::io::stdout().flush().unwrap();
        
        // 记录日志
        match std::fs::OpenOptions::new().create(true).append(true).open(log_file_path()) {
            Ok(mut f) => {
                let _ = writeln!(f, "{}", log_content);
            }
            Err(e) => {
                eprintln!("无法打开日志文件 {}: {}", log_file_path().display(), e);
            }
        }
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
