use std::{fs, path::Path, time::SystemTime};
// need a controller for shader file loading and hot reloading from a directory, and input handler to switch between shaders.
use notify_debouncer_mini::{
    DebounceEventResult, Debouncer, new_debouncer,
    notify::*,
    notify::{RecommendedWatcher, RecursiveMode},
};
use std::sync::mpsc;
use std::{path::PathBuf, time::Duration};
use wesl::Wesl;

pub const FALLBACK_SHADER: &str = r#"
struct Uniforms { resolution: vec2<f32>, mouse: vec2<f32>, time: f32, }
@group(0) @binding(0) var<uniform> u: Uniforms;
@vertex fn vs_main(@builtin(vertex_index) v_idx: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(pos[v_idx], 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red screen means fallback loaded
}
"#;

pub enum ShaderLoadResult {
    Success(String),
    CompileError,
    FileError,
}

pub struct ShaderController {
    watcher: ShaderWatcher,
    playlist: ShaderPlaylist,
    pub shader_path: Option<PathBuf>,
    pub last_modified: Option<SystemTime>,
}

impl ShaderController {
    pub fn new(dir: impl AsRef<Path>) -> Result<Self> {
        let shader_playlist = ShaderPlaylist::new(&dir)?;
        let first_shader = shader_playlist.current().cloned();

        Ok(Self {
            watcher: ShaderWatcher::new(&dir)?,
            playlist: shader_playlist,
            shader_path: first_shader,
            last_modified: None,
        })
    }

    pub fn get_shader_source(&self) -> Option<String> {
        match self.current_shader_source() {
            ShaderLoadResult::Success(source) => Some(source),
            ShaderLoadResult::CompileError => None,
            ShaderLoadResult::FileError => Some(FALLBACK_SHADER.to_string()),
        }
    }

    fn current_shader_source(&self) -> ShaderLoadResult {
        log::info!("Loading shader from file: {:?}", &self.shader_path);

        let Some(path) = self.shader_path.as_ref() else {
            return ShaderLoadResult::FileError;
        };

        let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) else {
            return ShaderLoadResult::FileError;
        };

        let module_path = format!("package::{}", file_stem);

        let Ok(parsed_path) = &module_path.parse() else {
            log::error!("Invalid WESL module path: {module_path}. Use underscores, not hyphens.");
            return ShaderLoadResult::FileError;
        };

        let compiler = Wesl::new(&self.playlist.dir);

        match compiler.compile(parsed_path) {
            Ok(compiled_module) => {
                let wgsl_code = compiled_module.to_string();

                match wgpu::naga::front::wgsl::parse_str(&wgsl_code) {
                    Ok(_) => ShaderLoadResult::Success(wgsl_code.to_string()),
                    Err(e) => {
                        log::error!("WGSL Syntax Error in {}: {:?}", path.display(), e);
                        ShaderLoadResult::CompileError
                    }
                }
            }
            Err(e) => {
                log::error!("WESL Compilation Error in {}: {}", path.display(), e);
                ShaderLoadResult::CompileError
            }
        }
    }

    pub fn check_for_updates(&mut self) -> Option<PathBuf> {
        let mut hot_reloaded_path = None;

        while let Ok(path) = self.watcher.receiver.try_recv() {
            if path
                .extension()
                .is_some_and(|ext| ext == "wgsl" || ext == "wesl")
                && let Some(current_path) = self.playlist.current()
                && path.file_name() == current_path.file_name()
            {
                hot_reloaded_path = Some(path);
            }
        }

        if let Some(path) = hot_reloaded_path
            && let Ok(metadata) = std::fs::metadata(&path)
        {
            let new_time = metadata.modified().ok();

            if new_time != self.last_modified {
                self.last_modified = new_time;
                return Some(path);
            }
        }

        None
    }

    pub fn handle_playlist_next(&mut self) {
        if self.playlist.next()
            && let Some(path) = self.playlist.current()
        {
            self.shader_path = Some(path.clone());
            self.last_modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        }
    }

    pub fn handle_playlist_prev(&mut self) {
        if self.playlist.prev()
            && let Some(path) = self.playlist.current()
        {
            self.shader_path = Some(path.clone());
            self.last_modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        }
    }
}

struct ShaderWatcher {
    _debouncer: Debouncer<RecommendedWatcher>,
    receiver: mpsc::Receiver<PathBuf>,
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
                Err(e) => log::error!("File watch error: {:?}", e),
            },
        )?;

        debouncer
            .watcher()
            .watch(dir.as_ref(), RecursiveMode::Recursive)
            .expect("Failed to watch shader directory.");

        Ok(Self {
            _debouncer: debouncer,
            receiver: rx,
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
        match fs::read_dir(&self.dir) {
            Ok(entries) => {
                for entry_result in entries {
                    match entry_result {
                        Ok(entry) => {
                            let path = entry.path();
                            if path.is_file()
                                && path
                                    .extension()
                                    .is_some_and(|ext| ext == "wgsl" || ext == "wesl")
                            {
                                self.files.push(path);
                            }
                        }
                        Err(e) => log::error!("Failed to read directory entry: {}", e),
                    }
                }
            }
            Err(_) => {
                log::error!("Error reading directory: {}", self.dir.display());
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
