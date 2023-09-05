use std::{fmt::Display, str::FromStr};

use argh::FromArgs;
use godot::prelude::{Dictionary, GodotString};

const CUSTOM_PREFIX: &str = "custom:";

#[derive(Debug, Clone)]
pub enum CliError {
    ParseFailure(argh::EarlyExit),
    UnknownTracker { input: String },
    UnknownModelType { input: String },
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseFailure(e) => write!(f, "{e:?}"),
            Self::UnknownTracker { input } => write!(f, "Unknown tracker: {input}"),
            Self::UnknownModelType { input } => write!(f, "Unknown model type: {input}"),
        }
    }
}

/// vpuppr command line interface
#[derive(Debug, FromArgs)]
pub struct Args {
    /// enable verbose logging, overridden by "quiet" if passed
    #[argh(switch, short = 'v', long = "verbose")]
    verbose: bool,
    /// disable all logging, overrides verbose
    #[argh(switch, short = 'q', long = "quiet")]
    quiet: bool,
    #[argh(subcommand)]
    commands: Option<Commands>,
}

impl Args {
    /// Parse some `args`. Args are expected to come from Godot user args.
    pub fn parse(args: &[&str]) -> Result<Self, CliError> {
        Self::from_args(&[env!("CARGO_PKG_NAME")], args).map_err(|e| CliError::ParseFailure(e))
    }

    /// Convert self to a [Dictionary].
    ///
    /// # Note
    /// **All keys must always be provided!**
    pub fn to_dict(&self) -> Dictionary {
        let mut r = Dictionary::new();

        r.insert("verbose", self.verbose);
        r.insert("quiet", self.quiet);

        if let Some(c) = &self.commands {
            r.insert("has_command", true);

            // TODO this seems weird but there doesn't seem to be a way to pass a trait
            // object to argh. The optimal solution would be Option<Box<dyn FromArgs + GodotCommand>>,
            // but FromArgs is not sized and thus cannot be boxed
            match c {
                Commands::Launch(c) => c.populate_dict(&mut r),
                Commands::WithModel(c) => c.populate_dict(&mut r),
            }
        } else {
            r.insert("has_command", false);
        }

        r
    }
}

trait GodotCommand {
    fn populate_dict(&self, dict: &mut Dictionary);
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
pub enum Commands {
    Launch(LaunchCommand),
    WithModel(WithModelCommand),
}

/// Launch vpuppr with some options
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "launch")]
pub struct LaunchCommand {
    /// name of the runner data to launch
    #[argh(positional)]
    runner_data: String,
    /// tracker to start upon launch
    #[argh(option)]
    tracker: Option<Tracker>,
}

impl GodotCommand for LaunchCommand {
    fn populate_dict(&self, dict: &mut Dictionary) {
        dict.insert("command", "launch");

        dict.insert("name", GodotString::from(&self.runner_data));
        dict.insert(
            "tracker",
            if let Some(tracker) = &self.tracker {
                GodotString::from(tracker)
            } else {
                GodotString::new()
            },
        );
    }
}

#[derive(Debug, PartialEq)]
pub enum Tracker {
    MediaPipe,
    IFacialMocap,
    VTubeStudio,
    MeowFace,
    OpenSeeFace,
    Custom(String),
}

impl FromStr for Tracker {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mediapipe" | "mp" => Ok(Self::MediaPipe),
            "ifacialmocap" | "ifm" => Ok(Self::IFacialMocap),
            "vtubestudio" | "vts" => Ok(Self::VTubeStudio),
            "meowface" | "mf" => Ok(Self::MeowFace),
            "openseeface" | "osf" => Ok(Self::OpenSeeFace),
            _ => {
                if let Some(v) = s.strip_prefix(CUSTOM_PREFIX) {
                    if v.len() > 0 {
                        return Ok(Self::Custom(v.to_string()));
                    }
                }

                Err(CliError::UnknownTracker {
                    input: s.to_string(),
                })
            }
        }
    }
}

impl AsRef<str> for Tracker {
    fn as_ref(&self) -> &str {
        match self {
            Tracker::MediaPipe => "mediapipe",
            Tracker::IFacialMocap => "ifacialmocap",
            Tracker::VTubeStudio => "vtubestudio",
            Tracker::MeowFace => "meowface",
            Tracker::OpenSeeFace => "openseeface",
            Tracker::Custom(v) => v.as_str(),
        }
    }
}

/// Launch vpuppr and load a new model
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "with-model")]
pub struct WithModelCommand {
    /// path to the model to load
    #[argh(positional)]
    model_path: String,
    /// force loading as model type
    #[argh(option)]
    model_type: Option<ModelType>,
    /// path to a custom runner
    #[argh(option)]
    runner_path: Option<String>,
    /// path to a custom gui
    #[argh(option)]
    gui_path: Option<String>,
}

impl GodotCommand for WithModelCommand {
    fn populate_dict(&self, dict: &mut Dictionary) {
        dict.insert("command", "with_model");

        dict.insert("model_path", GodotString::from(&self.model_path));
        dict.insert(
            "model_type",
            if let Some(v) = &self.model_type {
                GodotString::from(v)
            } else {
                GodotString::new()
            },
        );
        dict.insert(
            "runner_path",
            if let Some(v) = &self.runner_path {
                GodotString::from(v)
            } else {
                GodotString::new()
            },
        );
        dict.insert(
            "gui_path",
            if let Some(v) = &self.gui_path {
                GodotString::from(v)
            } else {
                GodotString::new()
            },
        );
    }
}

#[derive(Debug, PartialEq)]
pub enum ModelType {
    Glb,
    Vrm,
    PngTuber,
    Custom(String),
}

impl FromStr for ModelType {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "glb" => Ok(Self::Glb),
            "vrm" => Ok(Self::Vrm),
            "pngtuber" | "png tuber" => Ok(Self::PngTuber),
            _ => {
                if let Some(v) = s.strip_prefix(CUSTOM_PREFIX) {
                    if v.len() > 0 {
                        return Ok(Self::Custom(v.to_string()));
                    }
                }

                Err(CliError::UnknownModelType {
                    input: s.to_string(),
                })
            }
        }
    }
}

impl AsRef<str> for ModelType {
    fn as_ref(&self) -> &str {
        match self {
            ModelType::Glb => "glb",
            ModelType::Vrm => "vrm",
            ModelType::PngTuber => "pngtuber",
            ModelType::Custom(v) => v.as_ref(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let args = Args::parse(&["--verbose"]).unwrap();

        assert_eq!(args.verbose, true);
        assert_eq!(args.quiet, false);
    }

    #[test]
    fn empty() {
        let args = Args::from_args(&["vpuppr"], &[]).unwrap();

        assert_eq!(args.verbose, false);
        assert_eq!(args.quiet, false);
        assert!(args.commands.is_none());
    }

    #[test]
    fn no_command_verbose() {
        let args = Args::from_args(&["vpuppr"], &["--verbose"]).unwrap();

        assert_eq!(args.verbose, true);
        assert_eq!(args.quiet, false);
    }

    #[test]
    fn no_command_quiet() {
        let args = Args::from_args(&["vpuppr"], &["--quiet"]).unwrap();

        assert_eq!(args.verbose, false);
        assert_eq!(args.quiet, true);
    }

    #[test]
    fn no_command_quiet_verbose() {
        let args = Args::from_args(&["vpuppr"], &["--quiet", "--verbose"]).unwrap();

        assert_eq!(args.verbose, true);
        assert_eq!(args.quiet, true);
    }

    mod launch {
        use super::*;

        #[test]
        fn name_only() {
            let args = Args::from_args(&["vpuppr"], &["launch", "blah"]).unwrap();

            match args.commands.unwrap() {
                Commands::Launch(v) => {
                    assert_eq!(v.runner_data, "blah");
                    assert!(v.tracker.is_none());
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn tracker_full_name() {
            let args = Args::from_args(&["vpuppr"], &["launch", "blah", "--tracker", "mediapipe"])
                .unwrap();

            match args.commands.unwrap() {
                Commands::Launch(v) => {
                    assert_eq!(v.runner_data, "blah");
                    assert_eq!(v.tracker.unwrap(), Tracker::MediaPipe);
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn tracker_abbreviated_name() {
            let args =
                Args::from_args(&["vpuppr"], &["launch", "blah", "--tracker", "mp"]).unwrap();

            match args.commands.unwrap() {
                Commands::Launch(v) => {
                    assert_eq!(v.runner_data, "blah");
                    assert_eq!(v.tracker.unwrap(), Tracker::MediaPipe);
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn tracker_ignore_case() {
            let args = Args::from_args(&["vpuppr"], &["launch", "blah", "--tracker", "meDiAPIPE"])
                .unwrap();

            match args.commands.unwrap() {
                Commands::Launch(v) => {
                    assert_eq!(v.runner_data, "blah");
                    assert_eq!(v.tracker.unwrap(), Tracker::MediaPipe);
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn custom_tracker() {
            let args = Args::from_args(&["vpuppr"], &["launch", "blah", "--tracker", "custom:woo"])
                .unwrap();

            match args.commands.unwrap() {
                Commands::Launch(v) => {
                    assert_eq!(v.runner_data, "blah");
                    assert_eq!(v.tracker.unwrap(), Tracker::Custom("woo".to_string()));
                }
                _ => assert!(false),
            }

            let args = Args::from_args(
                &["vpuppr"],
                &["launch", "blah", "--tracker", "custom:custom:"],
            )
            .unwrap();

            match args.commands.unwrap() {
                Commands::Launch(v) => {
                    assert_eq!(v.runner_data, "blah");
                    assert_eq!(v.tracker.unwrap(), Tracker::Custom("custom:".to_string()));
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn missing_positional_name() {
            let args = Args::from_args(&["vpuppr"], &["launch", "--tracker", "mp"]);

            assert!(args.is_err());
        }

        #[test]
        fn unhandled_tracker() {
            let args =
                Args::from_args(&["vpuppr"], &["launch", "blah", "--tracker", "__invalid__"]);

            assert!(args.is_err());
        }

        #[test]
        fn missing_tracker_arg() {
            let args = Args::from_args(&["vpuppr"], &["launch", "blah", "--tracker"]);

            assert!(args.is_err());
        }

        #[test]
        fn empty_custom_tracker_name() {
            let args = Args::from_args(&["vpuppr"], &["launch", "blah", "--tracker", "custom:"]);

            assert!(args.is_err());
        }

        #[test]
        fn out_of_order_args() {
            let args = Args::from_args(&["vpuppr"], &["launch", "mediapipe", "blah", "--tracker"]);

            assert!(args.is_err());
        }
    }

    mod with_model {
        use super::*;

        #[test]
        fn model_path_only() {
            let args = Args::from_args(&["vpuppr"], &["with-model", "./blah.vrm"]).unwrap();

            match args.commands.unwrap() {
                Commands::WithModel(v) => {
                    assert_eq!(v.model_path, "./blah.vrm");
                    assert!(v.model_type.is_none());
                    assert!(v.runner_path.is_none());
                    assert!(v.gui_path.is_none());
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn force_model() {
            let args = Args::from_args(
                &["vpuppr"],
                &["with-model", "./blah.vrm", "--model-type", "glb"],
            )
            .unwrap();

            match args.commands.unwrap() {
                Commands::WithModel(v) => {
                    assert_eq!(v.model_path, "./blah.vrm");
                    assert_eq!(v.model_type.unwrap(), ModelType::Glb);
                    assert!(v.runner_path.is_none());
                    assert!(v.gui_path.is_none());
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn custom_runner_path() {
            let args = Args::from_args(
                &["vpuppr"],
                &["with-model", "./blah.vrm", "--runner-path", "./test.tscn"],
            )
            .unwrap();

            match args.commands.unwrap() {
                Commands::WithModel(v) => {
                    assert_eq!(v.model_path, "./blah.vrm");
                    assert!(v.model_type.is_none());
                    assert_eq!(v.runner_path.unwrap(), "./test.tscn");
                    assert!(v.gui_path.is_none());
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn custom_gui_path() {
            let args = Args::from_args(
                &["vpuppr"],
                &["with-model", "./blah.vrm", "--gui-path", "./gui.tscn"],
            )
            .unwrap();

            match args.commands.unwrap() {
                Commands::WithModel(v) => {
                    assert_eq!(v.model_path, "./blah.vrm");
                    assert!(v.model_type.is_none());
                    assert!(v.runner_path.is_none());
                    assert_eq!(v.gui_path.unwrap(), "./gui.tscn");
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn custom_runner_and_gui_path() {
            let args = Args::from_args(
                &["vpuppr"],
                &[
                    "with-model",
                    "./blah.vrm",
                    "--model-type",
                    "pngtuber",
                    "--runner-path",
                    "./test.tscn",
                    "--gui-path",
                    "./gui.tscn",
                ],
            )
            .unwrap();

            match args.commands.unwrap() {
                Commands::WithModel(v) => {
                    assert_eq!(v.model_path, "./blah.vrm");
                    assert_eq!(v.model_type.unwrap(), ModelType::PngTuber);
                    assert_eq!(v.runner_path.unwrap(), "./test.tscn");
                    assert_eq!(v.gui_path.unwrap(), "./gui.tscn");
                }
                _ => assert!(false),
            }
        }

        #[test]
        fn model_type_ignore_case() {
            let args = Args::from_args(
                &["vpuppr"],
                &["with-model", "./blah.vrm", "--model-type", "PnGTUBer"],
            )
            .unwrap();

            match args.commands.unwrap() {
                Commands::WithModel(v) => {
                    assert_eq!(v.model_path, "./blah.vrm");
                    assert_eq!(v.model_type.unwrap(), ModelType::PngTuber);
                    assert!(v.runner_path.is_none());
                    assert!(v.gui_path.is_none());
                }
                _ => assert!(false),
            }
        }
    }
}
