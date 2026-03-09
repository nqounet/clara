use std::io::{Read, Write};
use std::process::{Command, Stdio};
use tauri::Emitter;

use crate::config::ClaraConfig;

pub fn execute_cli(
    window: &tauri::Window,
    prompt: &str,
    config: &ClaraConfig,
    yolo: bool,
) -> Result<String, String> {
    let mut cmd = Command::new(&config.cli_command);
    cmd.args(&config.cli_args);

    if yolo {
        cmd.arg("--yolo");
    }

    if let Some(ref m) = config.model {
        cmd.arg("--model");
        cmd.arg(m);
    }

    if let Some(ref wd) = config.working_dir {
        if wd.exists() {
            cmd.current_dir(wd);
        } else {
            return Err(format!(
                "Workspaceディレクトリが存在しません: {}\n設定を確認してください。",
                wd.display()
            ));
        }
    }

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("CLIコマンドの起動に失敗しました: {}", e))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(prompt.as_bytes())
            .map_err(|e| format!("標準入力への書き込みに失敗しました: {}", e))?;
    }

    let mut stdout = child.stdout.take().ok_or("標準出力の取得に失敗しました")?;
    let mut full_output = String::new();
    let mut buffer = [0u8; 1024];
    let mut leftover = Vec::new();

    loop {
        let n = stdout
            .read(&mut buffer)
            .map_err(|e| format!("標準出力の読み込みに失敗しました: {}", e))?;
        if n == 0 {
            break;
        }

        leftover.extend_from_slice(&buffer[..n]);

        let (valid_str, next_leftover) = match std::str::from_utf8(&leftover) {
            Ok(s) => (s, Vec::new()),
            Err(e) => {
                let valid_up_to = e.valid_up_to();
                let s = std::str::from_utf8(&leftover[..valid_up_to]).unwrap();
                (s, leftover[valid_up_to..].to_vec())
            }
        };

        if !valid_str.is_empty() {
            window
                .emit("streaming-response", valid_str)
                .map_err(|e| format!("イベントの発行に失敗しました: {}", e))?;
            full_output.push_str(valid_str);
        }
        leftover = next_leftover;
    }

    if !leftover.is_empty() {
        let s = String::from_utf8_lossy(&leftover);
        window
            .emit("streaming-response", &s)
            .map_err(|e| format!("イベントの発行に失敗しました: {}", e))?;
        full_output.push_str(&s);
    }

    let status = child
        .wait()
        .map_err(|e| format!("コマンドの待機に失敗しました: {}", e))?;

    if status.success() {
        Ok(full_output)
    } else {
        let mut err_msg = String::new();
        if let Some(mut stderr) = child.stderr.take() {
            let _ = stderr.read_to_string(&mut err_msg);
        }
        Err(format!("CLIコマンドがエラーを返しました: {}", err_msg))
    }
}
