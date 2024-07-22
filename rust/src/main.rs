mod slack;

use std::fs::File;
use std::io::Read;

use slack::send_single_image_to_slack;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    slack::send_slack_message("Hello, world!").unwrap();

    let mut file = File::open("cargo.toml")?;
    let mut image_data = Vec::new();
    file.read_to_end(&mut image_data)?;

    // ファイル名
    let filename = "cargo.toml";

    // 画像のタイトル（Slackで表示される）
    let title = "My Amazing Image";

    slack::send_single_image_to_slack(&image_data, filename, title)?;

    Ok(())
}
