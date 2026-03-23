use std::path::Path;
// need a controller for shader file loading and hot reloading from a directory, and input handler to switch between shaders.
use notify_debouncer_mini::{
    DebounceEventResult, Debouncer, new_debouncer,
    notify::*,
    notify::{RecommendedWatcher, RecursiveMode, Watcher},
};
use std::sync::mpsc;
use std::{path::PathBuf, time::Duration};

struct ShaderWatcher {
    _debouncer: Debouncer<RecommendedWatcher>,
    reciever: mpsc::Receiver<PathBuf>,
}

impl ShaderWatcher {
    fn new(dir: impl AsRef<Path>) -> Result<Self> {
        let (tx, rx) = mpsc::channel();

        let mut debouncer = new_debouncer(
            Duration::from_millis(200),
            move |res: DebounceEventResult| match res {
                Ok(events) => {
                    for event in events {
                        let _ = tx.send(event.path);
                    }
                }
                Err(e) => eprintln!("File watch error: {:?}", e),
            },
        )?;

        debouncer
            .watcher()
            .watch(dir.as_ref(), RecursiveMode::Recursive)
            .expect("Failed to watch shader directory.");

        Ok(Self {
            _debouncer: debouncer,
            reciever: rx,
        })
    }
}

pub struct ShaderController {
    watcher: ShaderWatcher,
    pub shader_path: Option<PathBuf>,
}

impl ShaderController {
    pub fn new(dir: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            watcher: ShaderWatcher::new(dir)?,
            shader_path: None,
        })
    }

    pub fn check_for_updates(&mut self) -> Option<PathBuf> {
        while let Ok(path) = self.watcher.reciever.try_recv() {
            if path.extension().and_then(|ext| ext.to_str()) == Some("wgsl") {
                self.shader_path = Some(path);
            }
        }
        self.shader_path.clone()
    }
}
