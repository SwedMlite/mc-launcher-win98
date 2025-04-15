use serde::Deserialize;
use serde::Serialize;
use std::cmp::min;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct VersionManifest {
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct JavaVersion {
    #[allow(dead_code)]
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

impl JavaVersion {
    pub fn get_major_version(&self) -> u32 {
        self.major_version
    }
}

#[derive(Debug, Deserialize)]
pub struct VersionData {
    pub downloads: Downloads,
    #[serde(default)]
    pub libraries: Vec<Library>,
    #[serde(default, rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    #[serde(default, rename = "javaVersion")]
    pub java_version: Option<JavaVersion>,
}

impl VersionData {
    pub fn get_required_java_version(&self) -> Option<u32> {
        self.java_version.as_ref().map(|jv| jv.get_major_version())
    }
}

#[derive(Debug, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(default, rename = "type")]
    pub _type: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Downloads {
    pub client: DownloadInfo,
}

#[derive(Debug, Deserialize)]
pub struct AssetIndexData {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Deserialize)]
pub struct AssetObject {
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct DownloadInfo {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Library {
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub natives: Option<HashMap<String, String>>,
    #[serde(default)]
    pub extract: Option<Extract>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    #[serde(default)]
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<Os>,
}

#[derive(Debug, Deserialize)]
pub struct Os {
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Extract {
    pub exclude: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub username: String,
    #[serde(default)]
    pub jvm_args: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LaunchStage {
    PreparingLibraries,
    DownloadingLibraries,
    ExtractingNatives,
    PreparingAssets,
    DownloadingAssets,
    AssetLoadComplete,
    ValidatingJava,
    BuildingArguments,
    StartingProcess,
    ProcessStarted,
    LaunchingGame,
    Complete,
}

#[derive(Clone, Debug)]
pub struct LaunchProgress {
    pub stage: LaunchStage,
    pub message: String,
    pub current: usize,
    pub total: usize,
}

impl LaunchProgress {
    pub fn percentage(&self) -> f64 {
        let current = min(self.current, self.total);

        let base_percent = match self.stage {
            LaunchStage::PreparingLibraries => 0.0,
            LaunchStage::DownloadingLibraries => 10.0,
            LaunchStage::ExtractingNatives => 20.0,
            LaunchStage::PreparingAssets => 30.0,
            LaunchStage::DownloadingAssets => 40.0,
            LaunchStage::AssetLoadComplete => 50.0,
            LaunchStage::ValidatingJava => 60.0,
            LaunchStage::BuildingArguments => 70.0,
            LaunchStage::StartingProcess => 80.0,
            LaunchStage::ProcessStarted => 90.0,
            LaunchStage::LaunchingGame => 95.0,
            LaunchStage::Complete => 100.0,
        };

        let next_stage_percent = match self.stage {
            LaunchStage::DownloadingLibraries => 20.0,
            LaunchStage::DownloadingAssets => 50.0,

            _ => match self.stage {
                LaunchStage::PreparingLibraries => 10.0,
                LaunchStage::ExtractingNatives => 30.0,
                LaunchStage::PreparingAssets => 40.0,
                LaunchStage::AssetLoadComplete => 60.0,
                LaunchStage::ValidatingJava => 70.0,
                LaunchStage::BuildingArguments => 80.0,
                LaunchStage::StartingProcess => 90.0,
                LaunchStage::ProcessStarted => 95.0,
                LaunchStage::LaunchingGame => 100.0,
                LaunchStage::Complete => 100.0,
                _ => base_percent + 10.0,
            },
        };

        let stage_range = next_stage_percent - base_percent;
        let stage_progress = (current as f64 / self.total as f64) * stage_range;

        (base_percent + stage_progress).min(100.0)
    }
}