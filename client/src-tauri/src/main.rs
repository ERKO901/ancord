// V release buildu na Windows bez konzolového okna.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ancord_client_lib::run();
}
