use lazy_static::lazy_static;
use reqwest::Client;
use std::fs::File;
use std::io::Write;
use tokio;
use std::path::Path;
use std::collections::HashMap;

use tokio::task::JoinSet;
// Small bonus
//You can check what this url points to ~
const URL: &str = "https://d2nvs31859zcd8.cloudfront.net/f4cb2726cc0ac920360d_vedal987_45106169883_1733165997/chunked/index-muted-92RT7F9JEL.m3u8";
const TSURL: &str = "https://d2nvs31859zcd8.cloudfront.net/f4cb2726cc0ac920360d_vedal987_45106169883_1733165997/chunked";
lazy_static! {
    // need add use_rustls_tls() method to enable http2
    static ref CLIENT: Client = Client::builder().use_rustls_tls().build().unwrap();
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ts_files = get_ts_files(get_m3u8_file(URL).await?).await?;
    
    let mut join_set = JoinSet::new();
    for ts_file in ts_files.iter().cloned() {
        join_set.spawn(async move {
            if let Err(e) = download_ts_files(&ts_file).await {
                eprintln!("Error downloading {}: {:?}", ts_file, e);
            }
        });
    }
    ffmpeg_next::init().unwrap();
    // Await all tasks to complete
    use std::time::Instant;
    
    let start_time = Instant::now();
    while let Some(result) = join_set.join_next().await {
        if let Err(e) = result {
            eprintln!("Task failed: {:?}", e);
        } else {
            println!("Task completed");
        }
    }
    let end_time = Instant::now();
    println!("Time taken: {:?}", end_time.duration_since(start_time));
    println!("Merging ts files");
    merge_ts_files(ts_files).unwrap();
    Ok(())
}
async fn download_ts_files(ts_file: &str) -> anyhow::Result<()> {
    let url = format!("{}/{}", TSURL, ts_file);
    let response = CLIENT.get(url).send().await?;
    let body = response.bytes().await?;
    let file_name = ts_file;
    let mut file = File::create(&file_name)?;
    file.write_all(&body)?;
    println!("Downloaded {}", file_name);
    Ok(())
}
async fn get_m3u8_file(url: &str) -> anyhow::Result<String> {
    let response = CLIENT.get(url).send().await?;
    let body = response.text().await?;
    Ok(body)
}
async fn get_ts_files(str: String) -> anyhow::Result<Vec<String>> {
    let lines = str.lines();
    let mut ts_files = lines
        .filter(|line| line.contains(".ts"))
        .map(|l| l.to_string())
        .rev().take(20)
        .collect::<Vec<String>>();
    ts_files.reverse();
    // println!("{:?}", ts_files);
    Ok(ts_files)
}

fn merge_ts_files(ts_files: Vec<String>) -> anyhow::Result<()> {
    ffmpeg_next::init()?;

    let output_path = "output.ts";
    let mut output_format_context = ffmpeg_next::format::output(&output_path)?;

    for (index, input_path) in ts_files.iter().enumerate() {
        let input_path = Path::new(input_path);
        let mut input_format_context = match ffmpeg_next::format::input(input_path) {
            Ok(context) => context,
            Err(e) => {
                eprintln!("Error opening input file {}: {:?}", input_path.display(), e);
                continue;
            }
        };

        if index == 0 {
            for stream in input_format_context.streams() {
                let mut output_stream = output_format_context.add_stream(stream.parameters().id()).unwrap();
                output_stream.set_time_base(stream.time_base());
                output_stream.set_parameters(stream.parameters());
            }
            output_format_context.write_header()?;
        }

        let mut last_dts_map: HashMap<usize, i64> = HashMap::new();

        for (stream, mut packet) in input_format_context.packets() {
            let output_stream_index = stream.index();
            packet.set_stream(output_stream_index);

            packet.rescale_ts(
                stream.time_base(),
                output_format_context
                    .stream(output_stream_index)
                    .expect("Stream not found")
                    .time_base(),
            );

            // Stream-specific DTS check
            if let Some(current_dts) = packet.dts() {
                if let Some(&last_dts_value) = last_dts_map.get(&output_stream_index) {
                    if current_dts <= last_dts_value {
                        eprintln!("Non-monotonic DTS detected for stream {}: {} <= {}", output_stream_index, current_dts, last_dts_value);
                        continue; // Skip this packet
                    }
                }
                last_dts_map.insert(output_stream_index, current_dts);
            }

            if let Err(e) = packet.write_interleaved(&mut output_format_context) {
                eprintln!("写入数据包时出错: {:?}", e);
            }
        }
    }

    if let Err(e) = output_format_context.write_trailer() {
        eprintln!("写入尾部时出错: {:?}", e);
    }
    Ok(())
}

