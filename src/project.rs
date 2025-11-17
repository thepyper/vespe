use crate::ast2::{JsonPlusEntity, JsonPlusObject};
use crate::constants::{CTX_DIR_NAME, CTX_ROOT_FILE_NAME, METADATA_DIR_NAME};
use crate::execute2::{ContextAnalysis, ModelContent};
use crate::file::{FileAccessor, ProjectFileAccessor};
use crate::path::{PathResolver, ProjectPathResolver};

use std::sync::Arc;

use anyhow::Context as AnyhowContext;
use anyhow::Result;

use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::config::{EditorInterface, ProjectConfig};
use crate::editor::{lockfile::FileBasedEditorCommunicator, EditorCommunicator};

pub struct Project {
    editor_interface: Option<Arc<dyn EditorCommunicator>>,
    file_access: Arc<ProjectFileAccessor>,
    path_res: Arc<ProjectPathResolver>,
    project_config: ProjectConfig,
}

pub struct ExecuteContextInput {
    pub context_name: String,
    pub input_file: Option<String>,
    pub args: Option<Vec<String>>,
    pub defines: Option<Vec<String>>,
    pub additional_aux_paths: Option<Vec<PathBuf>>,
    pub output_path: Option<PathBuf>,
}

impl Default for ExecuteContextInput {
    fn default() -> Self {
        Self {
            context_name: String::new(),
            input_file: None,
            args: None,
            defines: None,
            additional_aux_paths: None,
            output_path: None,
        }
    }
}

impl Project {
    pub fn init(root_path: &Path) -> Result<()> {
        let ctx_dir = root_path.join(CTX_DIR_NAME);
        if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
            anyhow::bail!("ctx project already initialized in this directory.");
        }

        std::fs::create_dir_all(&ctx_dir).context("Failed to create .ctx directory")?;

        let ctx_root_file = ctx_dir.join(CTX_ROOT_FILE_NAME);
        std::fs::write(&ctx_root_file, "Feel The BuZZ!!")
            .context("Failed to write .ctx_root file")?;

        let mut project_config = ProjectConfig::default();

        project_config.git_integration_enabled = super::git::is_in_git_repository(&ctx_dir)?;

        let file_access = Arc::new(ProjectFileAccessor::new(root_path, None));
        let path_res = Arc::new(ProjectPathResolver::new(
            root_path.to_path_buf(),
            project_config.aux_paths.clone(),
            None, // output_path
        ));

        let project = Project {
            editor_interface: None,
            file_access,
            path_res,
            project_config,
        };

        project.save_project_config()?;
        project.commit(Some("Initialized vespe project.".into()))?;

        Ok(())
    }

    pub fn find(path: &Path) -> Result<Project> {
        let mut current_path = path.to_path_buf();

        loop {
            let ctx_dir = current_path.join(CTX_DIR_NAME);
            if ctx_dir.is_dir() && ctx_dir.join(CTX_ROOT_FILE_NAME).is_file() {
                let root_path = current_path.canonicalize()?;
                let project_config_path = root_path
                    .join(CTX_DIR_NAME)
                    .join(METADATA_DIR_NAME)
                    .join("project_config.json");
                let project_config = Self::load_project_config(&project_config_path)?;

                let editor_path = ctx_dir.join(METADATA_DIR_NAME).join(".editor");
                let editor_interface: Option<Arc<dyn EditorCommunicator>> =
                    match project_config.editor_interface {
                        EditorInterface::VSCode => {
                            Some(Arc::new(FileBasedEditorCommunicator::new(&editor_path)?)
                                as Arc<dyn EditorCommunicator>)
                        }
                        _ => None,
                    };

                let file_access = Arc::new(ProjectFileAccessor::new(
                    &root_path,
                    editor_interface.clone(),
                ));
                let path_res = Arc::new(ProjectPathResolver::new(
                    root_path.clone(),
                    project_config.aux_paths.clone(),
                    None, // output_path
                ));

                return Ok(Project {
                    editor_interface,
                    file_access,
                    path_res,
                    project_config,
                });
            }

            if !current_path.pop() {
                break;
            }
        }

        anyhow::bail!("No .ctx project found in the current directory or any parent directory.")
    }

    pub fn execute_context(&self, input: ExecuteContextInput) -> Result<ModelContent> {
        let mut data = match input.args {
            Some(args) => {
                let mut data = args
                    .iter()
                    .enumerate()
                    .map(|(i, x)| (format!("${}", i + 1), JsonPlusEntity::NudeString(x.clone())))
                    .collect::<BTreeMap<String, JsonPlusEntity>>();
                data.insert(
                    "$args".to_string(),
                    JsonPlusEntity::DoubleQuotedString(args.join(" ")),
                );
                let data = JsonPlusObject::from_map(data);
                data
            }
            None => JsonPlusObject::new(),
        };
        if let Some(defines) = input.defines {
            for define in defines {
                if let Some((key, value)) = define.split_once('=') {
                    let key = format!("${}", key);
                    data.insert(key, JsonPlusEntity::NudeString(value.to_string()));
                }
            }
        }
        data.insert(
            "$input".to_string(),
            JsonPlusEntity::DoubleQuotedString(input.input_file.unwrap_or(String::new())),
        );
        let mut path_res_builder = self.path_res.clone();

        if let Some(aux_paths) = input.additional_aux_paths {
            path_res_builder = Arc::new(path_res_builder.with_additional_aux_paths(aux_paths));
        }

        if let Some(output_path) = input.output_path {
            path_res_builder = Arc::new(path_res_builder.with_alternative_output_path(output_path));
        }

        let path_res = path_res_builder;

        let content = crate::execute2::execute_context(
            self.file_access.clone(),
            path_res,
            &input.context_name,
            Some(&data),
        )?;
        self.commit(Some(format!("Executed context {}.", input.context_name)))?;
        Ok(content)
    }

    pub fn analyze_context(&self, context_name: &str) -> Result<ContextAnalysis> {
        let analysis = crate::execute2::analyze_context(
            self.file_access.clone(),
            self.path_res.clone(),
            context_name,
        )?;
        Ok(analysis)
    }

    pub fn project_home(&self) -> PathBuf {
        self.path_res.project_home()
    }

    pub fn contexts_root(&self) -> PathBuf {
        self.path_res.contexts_root()
    }

    pub fn project_config_path(&self) -> PathBuf {
        self.path_res.metadata_home().join("project_config.json")
    }

    pub fn create_context_file(
        &self,
        name: &str,
        initial_content: Option<String>,
    ) -> Result<PathBuf> {
        let file_path = self.path_res.resolve_output_file(&format!("{}", name))?;
        if file_path.exists() {
            anyhow::bail!("Context file already exists: {}", file_path.display());
        }
        let content = initial_content.unwrap_or_else(|| "".to_string());
        self.file_access.write_file(&file_path, &content, None)?;

        self.commit(Some(format!("Created new context {}", name)))?;

        Ok(file_path)
    }

    pub fn save_project_config(&self) -> Result<()> {
        let config_path = self.project_config_path();
        std::fs::create_dir_all(
            config_path.parent().unwrap(), // TODO brutto!!
        )
        .context(format!("Failed to create metadata directory",))?;
        let serialized = serde_json::to_string_pretty(&self.project_config)?;
        self.file_access.write_file(
            &config_path,
            &serialized,
            Some("Saved project config file."),
        )?;
        Ok(())
    }

    pub fn load_project_config(project_config_path: &PathBuf) -> Result<ProjectConfig> {
        match std::fs::read_to_string(project_config_path) {
            Ok(content) => Ok(serde_json::from_str(&content)?),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(ProjectConfig::default()),
            Err(e) => Err(anyhow::Error::new(e).context("Failed to read project config file")),
        }
    }

    pub fn add_aux_path(&mut self, path: PathBuf) -> Result<()> {
        self.project_config.aux_paths.push(path);
        self.save_project_config()?;
        self.commit(Some("Added auxiliary path to project config.".into()))?;
        Ok(())
    }

    pub fn commit(&self, title_message: Option<String>) -> Result<()> {
        if self.project_config.git_integration_enabled {
            self.file_access.commit(title_message)
        } else {
            Ok(())
        }
    }
}
