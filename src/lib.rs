use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum PackMlMode {
    Invalid,
    Production,
    Manual,
    Maintenance,
    Calibration,
    External,
    Locked,
}

#[derive(Deserialize, Serialize, Debug)]
pub enum PackMlState {
    Invalid,
    Clearing,
    Stopped,
    Starting,
    Idle,
    Suspended,
    Execute,
    Stopping,
    Aborting,
    Aborted,
    Holding,
    Held,
    Unholding,
    Suspending,
    Unsuspending,
    Resetting,
    Completing,
    Completed,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
#[derive(Debug)]
pub struct StatusUpdate {
    pub mode: PackMlMode,
    pub state: PackMlState,
    pub step: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "Command")]
pub enum CommandType {
    Start,
    Abort,
    Clear,
    Hold,
    Stop,
    Suspend,
    Unhold,
    Unsuspend,
    Complete,
}
