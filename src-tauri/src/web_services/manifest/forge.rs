use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, Cursor, Read, Write},
    path::{Path, PathBuf},
    process::{Stdio, Command},
    time::Instant,
};

use log::{debug, error, info};
use serde::Deserialize;
#[cfg(test)]
use tempdir::TempDir;

use crate::{
    consts::{FORGE_FILES_BASE_URL, FORGE_MAVEN_BASE_URL},
    state::resource_manager::{ManifestError, ManifestResult},
    web_services::{
        downloader::{
            download_bytes_from_url, download_json_object_from_url, validate_hash_md5,
            DownloadResult,
        },
        manifest::get_classpath_separator,
    },
};

use super::{
    bytes_from_zip_file, get_directory_separator, maven_to_vec, path_to_utf8_str,
    vanilla::{LaunchArguments, Library},
};

#[derive(Debug, Deserialize)]
pub struct ForgeManifest(pub HashMap<String, Vec<String>>);

#[derive(Debug, Deserialize)]
pub struct ForgeHashes {
    classifiers: ForgeHashClassifiers,
}

impl ForgeHashes {
    pub fn installer_hash(&self) -> &ForgeFileHash {
        &self.classifiers.installer
    }
}

#[derive(Debug, Deserialize)]
pub struct ForgeHashClassifiers {
    sources: ForgeFileHash,
    mdk: ForgeFileHash,       // .zip
    changelog: ForgeFileHash, // .txt
    userdev: ForgeFileHash,
    universal: ForgeFileHash,
    installer: ForgeFileHash,
}

// Forge hashes are md5 NOT sha1
#[derive(Debug, Deserialize)]
pub struct ForgeFileHash {
    #[serde(rename = "jar", alias = "txt", alias = "zip")]
    hash: String,
}

impl From<&str> for ForgeFileHash {
    fn from(s: &str) -> Self {
        Self { hash: s.into() }
    }
}

#[derive(Debug, Deserialize)]
pub struct ForgeVersion {
    id: String,
    time: String,
    #[serde(rename = "releaseTime")]
    released_time: String,
    #[serde(rename = "type")]
    version_type: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "inheritsFrom")]
    inherits_from: String,
    // FIXME: Ignoring for now since this is just a empty json entry in 1.19.3, not sure about other versions
    // logging: Option<ForgeLogging>,
    pub arguments: LaunchArguments,
    pub libraries: Vec<Library>,
}

#[derive(Debug, Deserialize)]
pub struct ForgeData {
    client: String,
    server: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgeProcessor {
    sides: Option<Vec<String>>,
    jar: String,
    classpath: Vec<String>,
    args: Vec<String>,
    outputs: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct ForgeInstall {
    spec: u32,
    profile: String,
    version: String,
    // path: Option<String>,
    minecraft: String,
    #[serde(rename = "serverJarPath")]
    server_jar_path: Option<String>,
    pub data: HashMap<String, ForgeData>,
    pub processors: Vec<ForgeProcessor>,
    pub libraries: Vec<Library>,
}

#[derive(Debug, Deserialize)]
pub struct ForgeInstallerProfile {
    pub version: ForgeVersion,
    pub profile: ForgeInstall,
}

pub struct InstallerArgumentPaths {
    pub libraries_path: PathBuf,
    pub versions_dir_path: PathBuf,
    pub minecraft_version: String,
    pub forge_loader_version: String,
    pub tmp_dir: PathBuf,
}

pub async fn download_forge_hashes(forge_version: &str) -> DownloadResult<ForgeHashes> {
    let url = format!("{}/{}/meta.json", FORGE_FILES_BASE_URL, forge_version);
    Ok(download_json_object_from_url::<ForgeHashes>(&url).await?)
}

pub async fn download_forge_version(
    forge_version: &str,
    minecraft_version: &str,
    valid_hash: &ForgeFileHash,
    version_path: &Path,
    tmp_dir: &Path,
) -> ManifestResult<ForgeInstallerProfile> {
    // FIXME: This changes depending on the game version
    // https://github.com/gorilla-devs/GDLauncher/blob/391dd9cc7ef5ac6ef050327abb516eb6799f0539/src/common/reducers/actions.js#L1284
    let terminal = "installer.jar";
    let url = format!(
        "{0}/{1}/forge-{1}-{2}",
        FORGE_MAVEN_BASE_URL, forge_version, terminal
    );
    let bytes = download_bytes_from_url(&url).await?;

    if !validate_hash_md5(&bytes, &valid_hash.hash) {
        let error = "Could not validate installer hash, download aborted.".into();
        error!("{}", &error);
        return Err(ManifestError::MismatchedFileHash(error));
    }

    // Write bytes to the forge installers path.
    let dir_path = &version_path.join(minecraft_version).join("forgeInstallers");
    fs::create_dir_all(dir_path)?;

    // Save the forge installer file
    let path = dir_path.join(format!("forge-{}-{}", forge_version, terminal));
    if !path.exists() {
        let mut file = File::create(path)?;
        file.write_all(&bytes)?;
    }

    // Unzip the json files in memory
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;

    let version_file = archive.by_name("version.json")?;
    let version_bytes = bytes_from_zip_file(version_file);

    let install_profile_file = archive.by_name("install_profile.json")?;
    let install_profile_bytes = bytes_from_zip_file(install_profile_file);

    // Extract the rest of the archive into the tmp_dir
    archive.extract(tmp_dir)?;

    Ok(ForgeInstallerProfile {
        profile: serde_json::from_slice(&install_profile_bytes)?,
        version: serde_json::from_slice(&version_bytes)?,
    })
}

pub fn patch_forge(
    java_path: &Path,
    processors: Vec<ForgeProcessor>,
    data: HashMap<String, ForgeData>,
    forge_universal_path: Option<String>,
    argument_paths: InstallerArgumentPaths,
) -> Result<(), io::Error> {
    info!("Patching Forge");
    // Copy the data map so it can be mutable.
    let mut forge_data_map = HashMap::new();
    forge_data_map.extend(data.into_iter());

    // Format the client_lzma_path from the forge_universal_path
    if let Some(library_name) = forge_universal_path {
        // FIXME: Currently ignoring the "path" part of the install_profile.json
        let client_lzma_str = maven_to_vec(&library_name, Some("-clientdata"), Some(".lzma"))
            .join(&get_directory_separator());
        let client_lzma_path = argument_paths.libraries_path.join(client_lzma_str);
        let client_lzma_parent = client_lzma_path.parent().unwrap();
        if !client_lzma_parent.exists() {
            fs::create_dir_all(client_lzma_parent)?;
        }

        debug!(
            "Client lzma path: {}",
            path_to_utf8_str(&argument_paths.libraries_path.join(&client_lzma_path))
        );

        fs::copy(
            argument_paths.tmp_dir.join("data").join("client.lzma"),
            &client_lzma_path,
        )?;
        // Patches issue wit BINPATCH where it uses a relative path but should use the client_lzma_path created above
        forge_data_map.insert(
            "BINPATCH".into(),
            ForgeData {
                client: path_to_utf8_str(&argument_paths.libraries_path.join(client_lzma_path))
                    .into(),
                // TODO: Implement server
                server: "__UNIMPLEMENTED__".into(),
            },
        );
    } else {
        // FIXME: Populate errors to caller
        error!("Error getting forge universal path, does it exist?");
        return Ok(());
    }

    // Iterate over each processor and run them with the correctly substituted arguments.
    info!("Spawning forge patching processors...");
    let timer = Instant::now();
    for processor in processors {
        // Ignoring server side processors
        if let Some(sides) = &processor.sides {
            if !sides.contains(&"client".into()) {
                continue;
            }
        }

        let jar_path = argument_paths
            .libraries_path
            .join(maven_to_vec(&processor.jar, None, None).join(&get_directory_separator()));

        let classpaths: Vec<_> = processor
            .classpath
            .iter()
            .map(|classpath| {
                let processor_classpath =
                    maven_to_vec(classpath, None, None).join(&get_directory_separator());
                path_to_utf8_str(&argument_paths.libraries_path.join(processor_classpath))
                    .to_owned()
            })
            .collect();

        // Get the path to the version dir for a specific minecraft version.
        let game_version_path = argument_paths
            .versions_dir_path
            .join(&argument_paths.minecraft_version);

        // Create forge installer path inside the game version dir
        let forge_installers_path = game_version_path.join("forgeInstallers");
        if !forge_installers_path.exists() {
            fs::create_dir_all(&forge_installers_path)?;
        }

        let formatted_args: Vec<String> = processor
            .args
            .iter()
            .map(|argument| {
                replace_arg_if_possible(
                    argument,
                    &forge_data_map,
                    &forge_installers_path,
                    &game_version_path,
                    &argument_paths,
                )
            })
            .map(|argument| compute_path_if_possible(&argument, &argument_paths.libraries_path))
            .collect();

        if let Some(main_class) = obtain_main_class_from_jar(&jar_path) {
            // Format classpaths to include the processor jar
            let formatted_classpaths = format!(
                "{}{}{}",
                classpaths.join(&get_classpath_separator()),
                get_classpath_separator(),
                path_to_utf8_str(&jar_path),
            );
            let mut args: Vec<String> = vec!["-cp".into()];
            args.push(formatted_classpaths);
            args.push(main_class);
            args.extend(formatted_args);

            // Spawn a process for the forge processor with no output.
            let mut command = Command::new(java_path);
            command
                .current_dir(&argument_paths.tmp_dir)
                .args(args)
                .stdout(Stdio::null());
            debug!("Forge Processor: {:#?}", command);
            let mut child = command.spawn().expect("Could not spawn instance.");
            let id = child.id();
            info!("Spawned forge processor with PID {}", id);
            let status = child.wait()?;
            info!("Forge processor({}) exited with exit code: {}", id, status);
        } else {
            error!("Error obtaining main class from jar: {:#?}", &jar_path);
        }
    }
    info!(
        "Finished patching forge in {}ms",
        timer.elapsed().as_millis()
    );
    Ok(())
}

/// Extracts the jar manifest into memory and pulls out the 'Main-Class' entry if it exists.
fn obtain_main_class_from_jar(jar_path: &Path) -> Option<String> {
    let file = File::open(jar_path).unwrap();
    let reader = BufReader::new(file);

    // Read jar and pull out MANIFEST.MF and convert to a string
    let mut archive = zip::ZipArchive::new(reader).unwrap();
    let version_file = archive.by_name("META-INF/MANIFEST.MF").unwrap();
    let bytes = version_file.bytes().collect::<Result<Vec<u8>, _>>().ok()?;
    let manifest = String::from_utf8(bytes).ok()?;

    let id = "Main-Class: ";
    // Split the manifest at id and again at the newline to get only the main class, trimming excess whitespace.
    if let Some(entry) = manifest.find(id) {
        let (_, main_class_line) = manifest.split_at(entry + id.len());
        let main_class = &main_class_line[0..main_class_line.find('\n')?].trim();
        Some(main_class.to_string())
    } else {
        None
    }
}

// TODO: Allow using a side instead of always assuming 'client'
fn replace_arg_if_possible(
    arg: &str,
    data: &HashMap<String, ForgeData>,
    forge_installers_path: &Path,
    game_version_path: &Path,
    argument_paths: &InstallerArgumentPaths,
) -> String {
    // Early return the argument if there is no formatting to be done
    if !arg.contains('{') {
        return arg.into();
    }

    let mut formatted_arg = arg
        .replace("{SIDE}", "client")
        .replace("{ROOT}", path_to_utf8_str(forge_installers_path)) // Dirname of ${app_dir}/versions/<version>/forgeInstallers/<loaderVersion>.jar
        .replace(
            "{MINECRAFT_JAR}",
            path_to_utf8_str(
                &game_version_path
                    .join("client")
                    .join(format!("{}.jar", argument_paths.minecraft_version)),
            ),
        ) // Minecraft jar path
        .replace(
            "{MINECRAFT_VERSION}",
            path_to_utf8_str(
                &game_version_path.join(format!("{}.json", argument_paths.minecraft_version)),
            ),
        ) // Minecraft version json path
        .replace(
            "{INSTALLER}",
            path_to_utf8_str(
                &forge_installers_path
                    .join(&format!("{}.jar", argument_paths.forge_loader_version)),
            ),
        ) // ${app_dir}/versions/<version>/forgeInstallers/<loaderVersion>.jar
        .replace(
            "{LIBRARY_DIR}",
            path_to_utf8_str(&argument_paths.libraries_path),
        ); // Libraries path

    // Replace arguments from the installer_profile's 'data' entry
    for (key, value) in data {
        let substr = format!("{{{}}}", key);
        formatted_arg = formatted_arg.replace(&substr, &value.client);
    }

    formatted_arg
}

fn compute_path_if_possible(arg: &str, libraries_path: &Path) -> String {
    if arg.starts_with('[') {
        // Create path from maven with '[]' around them.
        let mut path = libraries_path.to_path_buf();
        for piece in maven_to_vec(&arg.replace('[', "").replace(']', ""), None, None) {
            path = path.join(piece);
        }
        // If the parent dir doesnt exist yet, make it
        let parent_dir = path.parent().unwrap();
        if !parent_dir.exists() {
            match fs::create_dir_all(parent_dir) {
                Ok(_) => {}
                Err(error) => {
                    error!("Error creating parent directories: {}", error);
                    return arg.to_owned();
                }
            }
        }

        path_to_utf8_str(&path).to_owned()
    } else {
        arg.to_owned()
    }
}

#[test]
pub fn test_download_forge_hashes() {
    let forge_version = "1.19.3-44.1.16";

    tauri::async_runtime::block_on(async move {
        let x = download_forge_hashes(forge_version).await;
        assert!(x.is_ok());
        println!("Hashes: {:#?}", x.unwrap());
    });
}

#[test]
pub fn test_download_forge_version() {
    let forge_version = "1.19.3-44.1.16";
    let tmp_dir = TempDir::new("temp").unwrap();

    tauri::async_runtime::block_on(async move {
        let x = download_forge_version(
            forge_version,
            "1.19.3",
            &"268bde630c51b1e94257d76377ec2424".into(),
            Path::new("/home/loucas/.config/com.autm.launcher/versions"),
            tmp_dir.path(),
        )
        .await;
        // println!("test_download_forge_version: {:#?}", &x);
        assert!(x.is_ok());
        let fp = x.unwrap();
        // println!("Result: {:#?}", x.unwrap());
        let paths = InstallerArgumentPaths {
            libraries_path: Path::new("/home/loucas/.config/com.autm.launcher/libraries")
                .to_path_buf(),
            versions_dir_path: Path::new("/home/loucas/.config/com.autm.launcher/versions")
                .to_path_buf(),
            minecraft_version: "1.19.3".into(),
            forge_loader_version: forge_version.into(),
            tmp_dir: tmp_dir.path().to_path_buf(),
        };

        // Find the path to the forge universal jar from the libraries list
        let forge_universal_path = fp
            .profile
            .libraries
            .iter()
            .map(|library| library.name.clone())
            .find(|name| name.starts_with("net.minecraftforge:forge:"));

        patch_forge(
            Path::new("/home/loucas/.config/com.autm.launcher/java/17.0.3/bin/java"),
            fp.profile.processors,
            fp.profile.data,
            forge_universal_path,
            paths,
        ).unwrap()
    });
}
