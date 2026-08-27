#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    match wubilex_app::run() {
        Ok(exit_code) => std::process::exit(exit_code),
        Err(error) => {
            eprintln!("WubiLex 启动失败：{error}");
            std::process::exit(1);
        }
    }
}
