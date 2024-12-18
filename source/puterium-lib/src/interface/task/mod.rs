use {
    crate::duration::SimpleDuration,
    schemars::JsonSchema,
    serde::{
        Deserialize,
        Serialize,
    },
    std::{
        collections::HashMap,
        net::SocketAddr,
        path::PathBuf,
    },
};

#[derive(Serialize, Deserialize, Clone, JsonSchema, PartialEq, Eq)]
#[serde(rename = "snake_case", deny_unknown_fields)]
pub struct Environment {
    /// If present, a map of environment variables and a bool, whether inherit from the
    /// context's parent environment variable pool. The bool is required for allowing
    /// overrides when merging configs, normally all entries would be `true`.
    pub clear: Option<HashMap<String, bool>>,
    /// Add or override the following environment variables;
    pub add: HashMap<String, String>,
}

impl Default for Environment {
    fn default() -> Self {
        return Self {
            clear: Some([].into_iter().collect()),
            add: Default::default(),
        };
    }
}

/// All dependencies will prevent the dependent from starting until they've reached
/// started state, and cause the dependent to stop when they leave started state.
/// Additional behaviors are indicated in this struct.
#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
#[serde(rename = "snake_case", deny_unknown_fields)]
pub enum DependencyType {
    /// Sets `transitive_on` in the dependency when the dependent is `on` (i.e. turns
    /// on deps that are off).
    Strong,
    Weak,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename = "snake_case", deny_unknown_fields)]
pub enum FiniteTaskEndAction {
    /// Nothing happens, task continues to be considered on and started.
    None,
    /// Set the user-on state to `false` once the task ends.
    TurnOff,
    /// Delete the task once the task ends. It will no longer show up in output and
    /// will be considered off.
    Delete,
}

impl Default for FiniteTaskEndAction {
    fn default() -> Self {
        return Self::None;
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename = "snake_case", deny_unknown_fields)]
pub struct TaskSpecEmpty {
    #[serde(default)]
    pub upstream: HashMap<String, DependencyType>,
    #[serde(default)]
    pub default_off: bool,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename = "snake_case", deny_unknown_fields)]
pub struct Command {
    #[serde(default)]
    pub working_directory: Option<PathBuf>,
    #[serde(default)]
    pub environment: Environment,
    pub command: Vec<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename = "snake_case", deny_unknown_fields)]
pub enum StartedCheck {
    /// Consider started when this tcp socket has a listener
    TcpSocket(SocketAddr),
    /// Consider started when a file exists at the following path
    Path(PathBuf),
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename = "snake_case", deny_unknown_fields)]
pub struct TaskSpecPerpetual {
    #[serde(default)]
    pub upstream: HashMap<String, DependencyType>,
    #[serde(default)]
    pub default_off: bool,
    /// Command to run
    pub command: Command,
    /// How to determine if command has started - otherwise immediately transition to
    /// started from starting.
    #[serde(default)]
    pub started_check: Option<StartedCheck>,
    /// How long to wait between restarts when the command fails. Defaults to 60s.
    #[serde(default)]
    pub restart_delay: Option<SimpleDuration>,
    /// How long to wait before force killing the process if it fails to stop. Defaults
    /// to 30s.
    pub stop_timeout: Option<SimpleDuration>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Eq)]
#[serde(rename = "snake_case", deny_unknown_fields)]
pub struct TaskSpecFinite {
    #[serde(default)]
    pub upstream: HashMap<String, DependencyType>,
    #[serde(default)]
    pub default_off: bool,
    /// Command to run
    pub command: Command,
    /// Which exit codes are considered success.  By default, `0`.
    #[serde(default)]
    pub success_codes: Vec<i32>,
    /// What to do when the command succeeds
    #[serde(default)]
    pub started_action: FiniteTaskEndAction,
    /// How long to wait between restarts when the command exits. Defaults to 60s.
    #[serde(default)]
    pub restart_delay: Option<SimpleDuration>,
    /// How long to wait before force killing the process if it fails to stop. Defaults
    /// to 30s.
    pub stop_timeout: Option<SimpleDuration>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone)]
#[serde(rename = "snake_case", deny_unknown_fields)]
pub enum Task {
    /// This is a task that has no internal state or process, etc, but can be used as a
    /// node in the graph for grouping other tasks (a.k.a. a target or loosely, a
    /// run-level).
    ///
    /// An empty task starts immediately and never fails.
    Empty(TaskSpecEmpty),
    /// A task that continues to run until stopped.
    ///
    /// Perpetual tasks are considered started immediately, unless a `start_check`
    /// command is provided.
    Perpetual(TaskSpecPerpetual),
    /// A task that stops on its own (a.k.a one shot).
    ///
    /// Finite tasks are considered started once they successfully exit.
    Finite(TaskSpecFinite),
    /// An external task is a task where the state is determined by an external process
    /// that communicates with puterium via API to communicate state changes.  Since it
    /// is externally managed, it can have no dependencies.
    ///
    /// When the task is set `user_on`, it is immediately also considered `started`
    /// (and vice-versa for `user_off`).
    External,
}
