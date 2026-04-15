// Windows 发布版下不要额外弹出控制台窗口，这行不能删。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mirage_lib::run()
}
