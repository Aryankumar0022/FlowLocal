// FlowLocal — Application entry point
// The `windows_subsystem` attribute hides the console window in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    flow_local_lib::run();
}
