use std::process::{Child, Command};
use std::sync::Mutex;

use tauri::AppHandle;

use crate::settings::Settings;

pub struct ServiceManager {
    child_process: Mutex<Option<Child>>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self {
            child_process: Mutex::new(None),
        }
    }

    pub fn start(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let app_dir = std::env::current_dir()?;
        let root_dir = app_dir.parent().unwrap().parent().unwrap().parent().unwrap();
        let main_py = root_dir.join("services").join("main.py");
        let services_dir = root_dir.join("services");

        tracing::info!("Starting Python services orchestrator at: {:?}", main_py);

        let mut cmd = Command::new("uv");
        cmd.arg("run").arg(main_py);
        cmd.current_dir(services_dir);
        apply_service_environment(&mut cmd, settings);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let child = cmd.spawn()?;
        *self.child_process.lock().unwrap() = Some(child);

        Ok(())
    }

    pub fn restart(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();
        self.start(app, settings)
    }

    pub fn stop(&self) {
        if let Some(mut child) = self.child_process.lock().unwrap().take() {
            tracing::info!("Stopping Python services orchestrator...");
            kill_process_tree(&mut child);
            let _ = child.wait();
        }
    }
}

fn apply_service_environment(cmd: &mut Command, settings: &Settings) {
    cmd.env("FLOWLOCAL_WHISPER_MODEL", settings.whisper_model.as_str())
        .env(
            "FLOWLOCAL_WHISPER_LANGUAGE",
            if settings.auto_detect_language {
                ""
            } else {
                settings.language.as_str()
            },
        )
        .env("FLOWLOCAL_WHISPER_VAD", settings.vad_enabled.to_string())
        .env(
            "FLOWLOCAL_WHISPER_VAD_THRESHOLD",
            settings.vad_threshold.to_string(),
        )
        .env(
            "FLOWLOCAL_WHISPER_SILENCE_MS",
            settings.silence_duration_ms.to_string(),
        )
        .env(
            "FLOWLOCAL_WHISPER_DEVICE",
            if settings.use_gpu { "auto" } else { "cpu" },
        )
        .env("FLOWLOCAL_WHISPER_COMPUTE_TYPE", "auto")
        .env("FLOWLOCAL_OLLAMA_HOST", settings.ollama_host.as_str())
        .env("FLOWLOCAL_OLLAMA_MODEL", settings.ollama_model.as_str())
        .env("FLOWLOCAL_EMBEDDING_MODEL", settings.embedding_model.as_str())
        .env("FLOWLOCAL_RAG_MAX_RESULTS", settings.rag_max_results.to_string());
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
fn kill_process_tree(child: &mut Child) {
    use std::os::windows::process::CommandExt;

    let status = Command::new("taskkill")
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();

    if let Err(error) = status {
        tracing::warn!("taskkill failed, falling back to child.kill(): {}", error);
        let _ = child.kill();
    }
}

#[cfg(not(windows))]
fn kill_process_tree(child: &mut Child) {
    let _ = child.kill();
}
