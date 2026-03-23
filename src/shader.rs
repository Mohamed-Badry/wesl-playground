use std::{fs, path::Path};
// need a controller for shader file loading and hot reloading from a directory, and input handler to switch between shaders.
use notify_debouncer_mini::{
    DebounceEventResult, Debouncer, new_debouncer,
    notify::*,
    notify::{RecommendedWatcher, RecursiveMode},
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

#[derive(Debug)]
struct ShaderPlaylist {
    dir: PathBuf,
    files: Vec<PathBuf>,
    current_index: usize,
}

impl ShaderPlaylist {
    fn new(dir: impl AsRef<Path>) -> Result<Self> {
        let mut playlist = Self {
            dir: dir.as_ref().to_path_buf(),
            files: Vec::new(),
            current_index: 0,
        };
        playlist.refresh();
        Ok(playlist)
    }

    fn refresh(&mut self) {
        self.files.clear();
        if let Ok(entries) = fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "wgsl") {
                    self.files.push(path);
                }
            }
        }
        self.files.sort();

        if self.current_index >= self.files.len() {
            self.current_index = 0;
        }
    }

    fn current(&self) -> Option<&PathBuf> {
        self.files.get(self.current_index)
    }

    fn next(&mut self) -> bool {
        if self.files.is_empty() {
            return false;
        }
        self.current_index = (self.current_index + 1) % self.files.len();
        true
    }

    fn prev(&mut self) -> bool {
        if self.files.is_empty() {
            return false;
        }
        if self.current_index == 0 {
            self.current_index = self.files.len() - 1;
        } else {
            self.current_index -= 1;
        }
        true
    }
}

pub struct ShaderController {
    watcher: ShaderWatcher,
    pub playlist: ShaderPlaylist,
    pub shader_path: Option<PathBuf>,
}

impl ShaderController {
    pub fn new(dir: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            watcher: ShaderWatcher::new(&dir)?,
            playlist: ShaderPlaylist::new(&dir)?,
            shader_path: None,
        })
    }

    pub fn check_for_updates(&mut self) -> Option<PathBuf> {
        let current_path = self.playlist.current().unwrap();
        let mut should_reload = false;

        while let Ok(path) = self.watcher.reciever.try_recv() {
            if path.extension().is_some_and(|ext| ext == "wgsl") {
                if path.file_name() == current_path.file_name() {
                    should_reload = true;
                }
            }
        }
        if should_reload {
            Some(current_path.clone())
        } else {
            None
        }
    }

    pub fn handle_playlist_next(&mut self) {
        let playlist = &mut self.playlist;
        if playlist.next() {
            self.shader_path = playlist.current().cloned();
        }
        dbg!(&self.shader_path);
        dbg!(&self.playlist);
    }

    pub fn handle_playlist_prev(&mut self) {
        let playlist = &mut self.playlist;
        if playlist.prev() {
            self.shader_path = playlist.current().cloned();
        }
        dbg!(&self.shader_path);
        dbg!(&self.playlist);
    }
}
