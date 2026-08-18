use std::{
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Mutex,
};

use serde::Deserialize;
use serde_json::json;

use super::{TranslationEngine, TranslationError};

pub struct MarianEngine {
    model_root: PathBuf,
    worker: Mutex<Option<Worker>>,
}

struct Worker {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

#[derive(Debug, Deserialize)]
struct MarianResp {
    ok: bool,
    translated: Option<String>,
    error: Option<String>,
}

impl MarianEngine {
    pub fn new(model_root: PathBuf) -> Self {
        Self {
            model_root,
            worker: Mutex::new(None),
        }
    }

    fn script_path() -> PathBuf {
        // V1: resolve relativo ao CWD (cargo run na raiz do repo)
        // Fallback: relativo ao diretório do exe
        let p1 = PathBuf::from("scripts").join("marian_translate.py");
        if p1.exists() {
            return p1;
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let p2 = dir.join("scripts").join("marian_translate.py");
                if p2.exists() {
                    return p2;
                }
            }
        }

        p1
    }

    fn spawn_worker(&self) -> Result<Worker, TranslationError> {
        let script = Self::script_path();
        if !script.exists() {
            return Err(TranslationError::EngineUnavailable(format!(
                "Missing script: {}",
                script.display()
            )));
        }

        // Preferir pythonw (não abre console). Fallback python.
        let mut cmd = Command::new("pythonw");
        cmd.arg("-u").arg(&script);

        let child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .or_else(|_| {
                let mut cmd = Command::new("python");
                cmd.arg("-u").arg(&script);
                cmd.stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
            })
            .map_err(|e| TranslationError::EngineUnavailable(format!("Failed to spawn python: {e}")))?;

        let mut child = child;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| TranslationError::EngineUnavailable("Failed to open stdin".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| TranslationError::EngineUnavailable("Failed to open stdout".to_string()))?;

        Ok(Worker {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    fn ensure_worker(&self) -> Result<std::sync::MutexGuard<'_, Option<Worker>>, TranslationError> {
        let mut guard = self
            .worker
            .lock()
            .map_err(|_| TranslationError::EngineUnavailable("Worker mutex poisoned".to_string()))?;

        let need_spawn = match guard.as_mut() {
            None => true,
            Some(w) => match w.child.try_wait() {
                Ok(Some(_status)) => true, // morreu
                Ok(None) => false,
                Err(_) => true,
            },
        };

        if need_spawn {
            *guard = Some(self.spawn_worker()?);

            // ping opcional (sanity)
            if let Some(w) = guard.as_mut() {
                let req = json!({ "cmd": "ping" }).to_string();
                w.stdin
                    .write_all(req.as_bytes())
                    .and_then(|_| w.stdin.write_all(b"\n"))
                    .and_then(|_| w.stdin.flush())
                    .map_err(|e| TranslationError::EngineUnavailable(format!("Worker ping write failed: {e}")))?;

                let mut line = String::new();
                w.stdout
                    .read_line(&mut line)
                    .map_err(|e| TranslationError::EngineUnavailable(format!("Worker ping read failed: {e}")))?;
            }
        }

        Ok(guard)
    }

    fn map_lang(src: &str, tgt: &str) -> Result<(String, String), TranslationError> {
        let src = src.trim().to_lowercase();
        let tgt = tgt.trim().to_lowercase();

        if tgt != "zh-cn" {
            return Err(TranslationError::LanguageNotSupported(format!(
                "Marian only wired for target zh-cn right now (got {tgt})"
            )));
        }

        match src.as_str() {
            "pt" | "en" | "es" => Ok((src, tgt)),
            _ => Err(TranslationError::LanguageNotSupported(src)),
        }
    }
}

impl TranslationEngine for MarianEngine {
    fn name(&self) -> &'static str {
        "marian"
    }

    fn is_available(&self) -> bool {
        // mínimo: script existe
        Self::script_path().exists()
    }

    fn requires_api_key(&self) -> bool {
        false
    }

    fn translate(&self, text: &str, source: &str, target: &str) -> Result<String, TranslationError> {
        let (source, target) = Self::map_lang(source, target)?;

        // model_root pode ser relativo; normaliza em runtime
        let model_root = if self.model_root.as_os_str().is_empty() {
            PathBuf::from("./models")
        } else {
            self.model_root.clone()
        };

        // garante worker
        let mut guard = self.ensure_worker()?;
        let w = guard
            .as_mut()
            .ok_or_else(|| TranslationError::EngineUnavailable("Worker not available".to_string()))?;

        let req = json!({
            "text": text,
            "source": source,
            "target": target,
            "model_root": model_root.to_string_lossy(),
        })
        .to_string();

        w.stdin
            .write_all(req.as_bytes())
            .and_then(|_| w.stdin.write_all(b"\n"))
            .and_then(|_| w.stdin.flush())
            .map_err(|e| TranslationError::EngineUnavailable(format!("Worker write failed: {e}")))?;

        let mut line = String::new();
        w.stdout
            .read_line(&mut line)
            .map_err(|e| TranslationError::EngineUnavailable(format!("Worker read failed: {e}")))?;

        let resp: MarianResp = serde_json::from_str(line.trim())
            .map_err(|e| TranslationError::TranslationFailed(format!("Invalid worker JSON: {e} | {line}")))?;

        if !resp.ok {
            return Err(TranslationError::TranslationFailed(
                resp.error.unwrap_or_else(|| "unknown error".to_string()),
            ));
        }

        let out = resp.translated.unwrap_or_default().trim().to_string();
        if out.is_empty() {
            return Err(TranslationError::TranslationFailed(
                "Marian returned empty translation".to_string(),
            ));
        }

        Ok(out)
    }
}
