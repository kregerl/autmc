use crate::state::{resource_manager::ResourceManager, ManagerFromAppHandle};
use autmc_authentication::MinecraftAccount;
use bytes::Bytes;
use futures::future::BoxFuture;
use log::{debug, error, info, warn};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use tauri::{AppHandle, Emitter, Manager, State, Wry};
use tempdir::TempDir;
use xmltree::{Element, XMLNode};
use zip::ZipArchive;

use crate::{
    consts::{JAVA_VERSION_MANIFEST_URL, LAUNCHER_NAME, LAUNCHER_VERSION},
    state::{
        instance_manager::{self, InstanceConfiguration, InstanceManager, InstanceState},
        resource_manager::{ManifestError, ManifestResult, ResourceState},
    },
    web_services::{
        downloader::{
            boxed_buffered_download_stream, buffered_download_stream, download_bytes_from_url,
            download_json_object_from_url, validate_hash_sha1, DownloadError, Downloadable,
        },
        manifest::{
            fabric::{download_fabric_profile, obtain_fabric_library_hashes},
            forge::{
                download_forge_hashes, download_forge_version, patch_forge, ForgeInstallerProfile,
                InstallerArgumentPaths,
            },
            get_classpath_separator, path_to_utf8_str,
            vanilla::{
                Argument, AssetObject, DownloadableClassifier, JavaRuntimeFile,
                JavaRuntimeManifest, JavaRuntimeType, VanillaVersion,
            },
        },
    },
};

use super::{
    downloader::{hash_bytes_sha1, validate_file_hash},
    manifest::vanilla::{
        AssetIndex, DownloadMetadata, JarType, JavaManifest, JavaRuntime, JavaVersion,
        LaunchArguments, LaunchArguments113, Library, Logging, Rule, RuleType,
        VanillaManifestVersion,
    },
};

/// Checks if a single rule matches every case.
/// Returns true when an allow rule matches or a disallow rule does not match.
fn rule_matches(rule: &Rule) -> bool {
    let rule_type = &rule.rule_type;
    if rule_type.is_none() {
        return match rule.action.as_str() {
            "allow" => true,
            "disallow" => false,
            _ => unimplemented!("Unknwon rule action: {}", rule.action),
        };
    }
    match rule_type.as_ref().unwrap() {
        RuleType::Features(_feature_rules) => {
            error!(
                "Implement feature rules for arguments: {:#?}",
                _feature_rules
            );
            // FIXME: Currently just skipping these
            false
        }
        RuleType::OperatingSystem(os_rules) => {
            // Check if all the rules match the current system.
            let mut rule_matches = false;
            for (key, value) in os_rules {
                match key.as_str() {
                    "name" => {
                        let os_type = env::consts::OS;
                        if value == os_type || (os_type == "macos" && value == "osx") {
                            rule_matches = true;
                        }
                    }
                    "arch" => {
                        let os_arch = env::consts::ARCH;
                        if value == os_arch || (value == "x86" && os_arch == "x86_64") {
                            rule_matches = true;
                        }
                    }
                    "version" => { /*TODO: Check version of os to make sure it matches*/ }
                    _ => unimplemented!("Unknown rule map key: {}", key),
                }
            }
            // Check if we allow or disallow this downloadable
            match rule.action.as_str() {
                "allow" => rule_matches,
                "disallow" => !rule_matches,
                _ => unimplemented!("Unknwon rule action: {}", rule.action),
            }
        }
    }
}

fn rules_match(rules: &[Rule]) -> bool {
    let mut result = false;
    for rule in rules {
        if rule_matches(rule) {
            result = true;
        } else {
            return false;
        }
    }
    result
}

fn determine_key_for_java_manifest<'a>(
    java_version_manifest_map: &HashMap<String, JavaManifest>,
) -> &'a str {
    let os = env::consts::OS;
    let key = if os == "macos" { "mac-os" } else { os };

    if java_version_manifest_map.contains_key(key) {
        return key;
    }
    let architecture = env::consts::ARCH;
    match key {
        "linux" => {
            if architecture == "x86" {
                "linux-i386"
            } else {
                key
            }
        }
        "mac-os" => {
            if architecture == "arm" {
                "mac-os-arm64"
            } else {
                key
            }
        }
        "windows" => {
            if architecture == "x86" {
                "windows-x86"
            } else if architecture == "x86_64" {
                "windows-x64"
            } else {
                unreachable!("Unexpected windows architecture: {}", architecture)
            }
        }
        _ => {
            unreachable!(
                "Unknown java version this OS: {}. Expected `linux`, `mac-os` or `windows`",
                key
            )
        }
    }
}
struct LaunchArgumentPaths {
    // logging configurations are optional since they dont exist in versions 1.6.4 and older
    logging: Option<(String, PathBuf)>,
    library_paths: Vec<PathBuf>,
    instance_path: PathBuf,
    jar_path: PathBuf,
    asset_dir_path: PathBuf,
    library_directory: PathBuf,
}

// TODO: Add -Xmx and -Xms arguments for memory
fn construct_jvm_arguments113(
    arguments: &LaunchArguments113,
    argument_paths: &LaunchArgumentPaths,
    mc_version: &str,
) -> Vec<String> {
    let mut formatted_arguments = Vec::new();

    if arguments.jvm.is_none() {
        return formatted_arguments;
    }

    for jvm_arg in arguments.jvm.as_ref().unwrap().iter() {
        match jvm_arg {
            // For normal arguments, check if it has something that should be replaced and replace it
            Argument::Arg(value) => {
                let sub_arg = substitute_jvm_arguments(value, mc_version, argument_paths);
                formatted_arguments.push(match sub_arg {
                    Some(argument) => argument,
                    None => value.into(),
                });
            }
            // For conditional args, check their rules before adding to formatted_arguments vec
            Argument::ConditionalArg { rules, values } => {
                if !rules_match(rules) {
                    continue;
                }
                for value in values {
                    let sub_arg = substitute_jvm_arguments(value, mc_version, argument_paths);
                    formatted_arguments.push(match sub_arg {
                        Some(argument) => argument,
                        None => value.into(),
                    });
                }
            }
        }
    }
    formatted_arguments
}

// TODO: Add -Xmx and -Xms arguments for memory
fn construct_jvm_arguments112(
    mc_version: &str,
    argument_paths: &LaunchArgumentPaths,
) -> Vec<String> {
    vec![
        substitute_jvm_arguments(
            "-Djava.library.path=${natives_directory}",
            mc_version,
            argument_paths,
        )
        .unwrap(),
        "-cp".to_string(),
        substitute_jvm_arguments("${classpath}", mc_version, argument_paths).unwrap(),
    ]
}

fn construct_arguments(
    main_class: String,
    additional_arguments: String,
    // (Width, Height)
    resolution: (String, String),
    arguments: &LaunchArguments,
    modloader_arguments: Option<LaunchArguments>,
    modloader_type: &ModloaderType,
    mc_version: &VanillaManifestVersion,
    asset_index: &str,
    argument_paths: LaunchArgumentPaths,
) -> Vec<String> {
    // IDEA: Vec could be 'with_capacity' if we calculate capacity first.
    let mut formatted_arguments: Vec<String> = Vec::new();
    let mut game_args: Vec<Argument> = Vec::new();

    // Empty strings will screw up the jvm arguments
    if !additional_arguments.is_empty() {
        formatted_arguments.push(additional_arguments);
    }

    // Create game arguments from the launch arguments.
    game_args.append(&mut match arguments {
        // Substitute values in for placeholders in the jvm arguments.

        // Versions <= 1.12  use a string of game arguments and do not provide any jvm arguments.
        LaunchArguments::LaunchArguments112(game_args) => {
            formatted_arguments.append(&mut construct_jvm_arguments112(
                &mc_version.id,
                &argument_paths,
            ));

            // If the modloader is forge and 1.12.2 or older, then ignore vanilla arguments since they
            // are already provided by the forge arguments.
            match modloader_arguments {
                // If we have some arguments and the modloader type is forge
                Some(_) if *modloader_type == ModloaderType::Forge => Vec::new(),
                _ => {
                    // Split game arg string on whitespace to get individual args
                    game_args
                        .split_ascii_whitespace()
                        .map(|split| Argument::Arg(split.into()))
                        .collect::<Vec<Argument>>()
                }
            }
        }
        // Versions >= 1.13 provide the game and jvm arguments.
        LaunchArguments::LaunchArguments113(arguments) => {
            formatted_arguments.append(&mut construct_jvm_arguments113(
                arguments,
                &argument_paths,
                &mc_version.id,
            ));
            arguments.game.to_vec()
        }
    });

    // Append modloader arguments if a modloader is selected.
    if let Some(args) = modloader_arguments {
        game_args.append(&mut match args {
            LaunchArguments::LaunchArguments112(game_args) => {
                formatted_arguments.append(&mut construct_jvm_arguments112(
                    &mc_version.id,
                    &argument_paths,
                ));
                // Split game arg string on whitespace to get individual args
                game_args
                    .split_ascii_whitespace()
                    .map(|split| Argument::Arg(split.into()))
                    .collect::<Vec<Argument>>()
            }
            // Versions >= 1.13 provide the game and jvm arguments.
            LaunchArguments::LaunchArguments113(arguments) => {
                formatted_arguments.append(&mut construct_jvm_arguments113(
                    &arguments,
                    &argument_paths,
                    &mc_version.id,
                ));
                arguments.game.to_vec()
            }
        });
    }

    if let Some((arg, path)) = &argument_paths.logging {
        // Construct the logging configuration argument
        if let Some(substr) = get_arg_substring(arg) {
            formatted_arguments.push(arg.replace(substr, path_to_utf8_str(path)));
        }
    }

    // Add main class
    formatted_arguments.push(main_class);

    // Substitute values in for placeholders in the game arguments, skipping account-specific arguments.
    for game_arg in game_args.iter() {
        match game_arg {
            // For normal arguments, check if it has something that should be replaced and replace it
            Argument::Arg(value) => {
                let sub_arg = substitute_game_arguments(
                    value,
                    &resolution,
                    mc_version,
                    asset_index,
                    &argument_paths,
                );
                formatted_arguments.push(match sub_arg {
                    Some(argument) => argument,
                    None => value.into(),
                });
            }
            // For conditional args, check their rules before adding to formatted_arguments vec
            Argument::ConditionalArg { rules, values } => {
                if !rules_match(rules) {
                    continue;
                }
                for value in values {
                    let sub_arg = substitute_game_arguments(
                        value,
                        &resolution,
                        mc_version,
                        asset_index,
                        &argument_paths,
                    );
                    formatted_arguments.push(match sub_arg {
                        Some(argument) => argument,
                        None => value.into(),
                    });
                }
            }
        }
    }
    println!("HERE: {:#?}", formatted_arguments);
    formatted_arguments
}

// Returns the substring inside the argument if it exists, otherwise None
fn get_arg_substring(arg: &str) -> Option<&str> {
    let substr_start = arg.chars().position(|c| c == '$');
    let substr_end = arg.chars().position(|c| c == '}');

    if let (Some(start), Some(end)) = (substr_start, substr_end) {
        debug!("get_arg_substring: {}", &arg[start..=end]);
        Some(&arg[start..=end])
    } else {
        None
    }
}

// Returns a string with the substituted value in the jvm argument or None if it doesn't apply.
// mc_version is only needed here for one forge specific launch option
fn substitute_jvm_arguments(
    arg: &str,
    mc_version: &str,
    argument_paths: &LaunchArgumentPaths,
) -> Option<String> {
    debug!("substitute_jvm_arguments: {}", arg);
    let classpath_strs: Vec<&str> = argument_paths
        .library_paths
        .iter()
        .map(|path| path_to_utf8_str(path))
        .collect();

    let mut formatted_argument: Option<String> = None;
    // Iterate here since some arguments(forge) can have multiple substitutions in them
    loop {
        let arg_to_replace = if let Some(argument) = &formatted_argument {
            argument.as_str()
        } else {
            arg
        };

        let substring = get_arg_substring(arg_to_replace);

        if let Some(substr) = substring {
            info!("Substituting {} for jvm arguments", &substr);
            formatted_argument = match substr {
                "${natives_directory}" => Some(arg_to_replace.replace(
                    substr,
                    path_to_utf8_str(&argument_paths.instance_path.join("natives")),
                )),
                "${launcher_name}" => Some(arg_to_replace.replace(substr, LAUNCHER_NAME)),
                "${launcher_version}" => Some(arg_to_replace.replace(substr, LAUNCHER_VERSION)),
                "${classpath}" => {
                    debug!("Vec: {:#?}", classpath_strs);
                    debug!(
                        "Classpath: {} ",
                        classpath_strs.join(&get_classpath_separator())
                    );
                    Some(arg_to_replace.replace(
                        substr,
                        &format!(
                            "{}{}{}",
                            classpath_strs.join(&get_classpath_separator()),
                            get_classpath_separator(),
                            path_to_utf8_str(&argument_paths.jar_path)
                        ),
                    ))
                }
                // Forge specific jvm arguments
                "${library_directory}" => Some(
                    arg_to_replace
                        .replace(substr, path_to_utf8_str(&argument_paths.library_directory)),
                ),
                "${classpath_separator}" => {
                    Some(arg_to_replace.replace(substr, &get_classpath_separator()))
                }
                "${version_name}" => Some(arg_to_replace.replace(substr, mc_version)),
                _ => formatted_argument,
            };
            debug!("Got: {:#?}", formatted_argument);
        } else {
            break;
        }
    }
    formatted_argument
}

fn substitute_game_arguments(
    arg: &str,
    resolution: &(String, String),
    mc_version: &VanillaManifestVersion,
    asset_index: &str,
    argument_paths: &LaunchArgumentPaths,
) -> Option<String> {
    let substring = get_arg_substring(arg);

    if let Some(substr) = substring {
        info!("Substituting {} for game arguments", &substr);
        match substr {
            "${version_name}" => Some(arg.replace(substr, &mc_version.id)),
            "${game_directory}" => {
                Some(arg.replace(substr, path_to_utf8_str(&argument_paths.instance_path)))
            }
            "${assets_root}" => {
                Some(arg.replace(substr, path_to_utf8_str(&argument_paths.asset_dir_path)))
            }
            "${game_assets}" => Some(arg.replace(
                substr,
                path_to_utf8_str(&argument_paths.asset_dir_path.join("virtual").join("legacy")),
            )),
            "${assets_index_name}" => Some(arg.replace(substr, asset_index)),
            "${user_type}" => Some(arg.replace(substr, "mojang")),
            "${version_type}" => Some(arg.replace(substr, &mc_version.version_type)),
            "${resolution_width}" => Some(arg.replace(substr, &resolution.0)),
            "${resolution_height}" => Some(arg.replace(substr, &resolution.1)),
            "${user_properties}" => {
                debug!("Substituting user_properties at substr: {}", substr);
                Some(arg.replace(substr, "{}"))
            }
            _ => None,
        }
    } else {
        None
    }
}

pub fn substitute_account_specific_arguments(
    arg: &str,
    active_account: &MinecraftAccount,
) -> Option<String> {
    if let Some(substr) = get_arg_substring(arg) {
        match substr {
            "${auth_player_name}" => Some(arg.replace(substr, &active_account.name)),
            "${auth_uuid}" => Some(arg.replace(substr, &active_account.uuid)),
            "${auth_access_token}" => {
                Some(arg.replace(substr, &active_account.minecraft_access_token))
            }
            "${clientid}" => None,  // FIXME: Unknown
            "${auth_xuid}" => None, // FIXME: Unknown
            _ => None,
        }
    } else {
        None
    }
}

struct LibraryData {
    downloadables: Vec<Box<dyn Downloadable + Send + Sync>>,
    classifiers: Vec<DownloadableClassifier>,
}

fn separate_classifiers_from_libraries(libraries: Vec<Library>) -> LibraryData {
    let mut downloadables: Vec<Box<dyn Downloadable + Send + Sync>> = Vec::new();
    let mut classifiers: Vec<DownloadableClassifier> = Vec::new();

    // Since libraries can have one artifact but no classifiers, or no artifact but (one or many) classifiers, or an artifact AND classifiers
    // we extract all the downloadable artifacts into a vec and perform a single buffered download with them.
    for library in libraries {
        let downloads = &library.downloads;
        // Push the artifact if library has one.
        if let Some(artifact) = &downloads.artifact {
            downloadables.push(Box::new(artifact.to_owned()));
        }
        // If there is a natives json entry with an applicable (os dependent) classifier, get and append it
        let key = library.determine_key_for_classifiers();
        if let Some(classifier_key) = key {
            // Classifiers could be "missing" if the wrong key is used. An error is logged and downloads continue
            let classifier = match library.get_classifier(&classifier_key) {
                Some(classifier) => classifier,
                None => continue,
            };
            classifiers.push(classifier.clone());
            downloadables.push(Box::new(classifier.classifier));
        }
    }
    LibraryData {
        classifiers,
        downloadables,
    }
}

async fn download_libraries(
    libraries_dir: &Path,
    libraries: &[Box<dyn Downloadable + Send + Sync>],
) -> ManifestResult<Vec<PathBuf>> {
    info!("Downloading {} libraries...", libraries.len());
    if !libraries_dir.exists() {
        fs::create_dir(libraries_dir)?;
    }
    let start = Instant::now();
    // Perform one buffered download for all libraries, including classifiers
    boxed_buffered_download_stream(libraries, libraries_dir, |bytes, artifact| {
        // Skip empty hashes for forge 1.11 and older.
        if !artifact.hash().is_empty() && !validate_hash_sha1(bytes, artifact.hash()) {
            let err = format!("Error downloading {}, invalid hash.", &artifact.url());
            error!("{}", err);
            return Err(DownloadError::InvalidFileHash(err));
        }
        debug!("Downloading library: {}", artifact.name());
        // Windows only?
        // let artifact_path = str::replace(artifact.name(), "/", "\\");
        let path = artifact.path(libraries_dir);
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        Ok(())
    })
    .await?;
    info!(
        "Successfully downloaded libraries in {}ms",
        start.elapsed().as_millis()
    );
    let mut file_paths: Vec<PathBuf> = Vec::with_capacity(libraries.len());
    for library in libraries {
        file_paths.push(library.path(libraries_dir));
    }

    Ok(file_paths)
}

async fn download_game_jar(
    versions_dir: &Path,
    jar_type: JarType,
    download: &DownloadMetadata,
    version_id: &str,
) -> ManifestResult<PathBuf> {
    let jar_str = match jar_type {
        JarType::Client => "client",
        JarType::Server => "server",
    };
    // Create all dirs in path to file location.
    let dir_path = &versions_dir.join(version_id).join(jar_str);
    fs::create_dir_all(dir_path)?;

    let path = dir_path.join(format!("{}.jar", &version_id));
    let valid_hash = download.hash();
    // Check if the file exists and the hash matches the download's sha1.
    if !validate_file_hash(&path, valid_hash) {
        info!("Downloading {} {} jar", version_id, jar_str);
        let bytes = download_bytes_from_url(download.url()).await?;
        if !validate_hash_sha1(&bytes, valid_hash) {
            let err = format!(
                "Error downloading {} {} jar, invalid hash.",
                version_id, jar_str
            );
            error!("{}", err);
            return Err(ManifestError::MismatchedFileHash(err));
        }
        let mut file = File::create(&path)?;
        file.write_all(&bytes)?;
    }
    Ok(path)
}

async fn download_java_from_runtime_manifest(
    java_dir: &Path,
    manifest: &JavaRuntime,
) -> ManifestResult<PathBuf> {
    info!("Downloading java runtime manifset");
    let version_manifest: JavaRuntimeManifest =
        download_json_object_from_url(manifest.manifest.url()).await?;
    let base_path = &java_dir.join(&manifest.version.name);

    let mut files: Vec<JavaRuntimeFile> = Vec::new();
    // Links is a Vec<(Path, Target)>
    let mut links: Vec<(String, String)> = Vec::new();
    // Create directories first and save the remaining.
    for jrt in version_manifest.files {
        match jrt {
            JavaRuntimeType::File(jrt_file) => files.push(jrt_file),
            JavaRuntimeType::Directory(dir) => {
                let path = &base_path.join(dir);
                fs::create_dir_all(path)?;
            }
            JavaRuntimeType::Link { path, target } => links.push((path, target)),
        }
    }

    // Next download files.
    // FIXME: Currently downloading `raw` files, switch to lzma and decompress locally.
    info!("Downloading all java files.");
    let start = Instant::now();
    buffered_download_stream(&files, base_path, |bytes, jrt| {
        if !validate_hash_sha1(bytes, jrt.hash()) {
            let err = format!("Error downloading {}, invalid hash.", &jrt.url());
            error!("{}", err);
            return Err(DownloadError::InvalidFileHash(err));
        }
        let path = jrt.path(base_path);
        let mut file = File::create(path)?;
        #[cfg(target_family = "unix")]
        {
            use std::os::unix::prelude::PermissionsExt;

            // Mark the file as executable on unix os's
            if jrt.executable {
                let mut permissions = file.metadata()?.permissions();
                permissions.set_mode(0o775);
                file.set_permissions(permissions)?;
            }
        }
        file.write_all(bytes)?;
        Ok(())
    })
    .await?;
    info!("Downloaded java in {}ms", start.elapsed().as_millis());

    // Finally create links
    for link in links {
        let to = &base_path.join(link.0);
        if !to.exists() {
            // Cant fail since the dirs were made before
            let dir_path = to.parent().unwrap().join(link.1);
            let from = dir_path.canonicalize()?;

            if from.is_dir() {
                debug!(
                    "Creating symlink between {} and {}",
                    from.display(),
                    to.display()
                );
                #[cfg(target_os = "linux")]
                {
                    use std::os::unix::fs::symlink;

                    // Create symlink FROM "target" TO "path"
                    symlink(from, to)?;
                }
            } else {
                debug!(
                    "Creating hard link between {} and {}",
                    from.display(),
                    to.display()
                );
                // Create hard link FROM "target" TO "path"
                fs::hard_link(from, to)?;
            }
        }
    }

    let java_path = base_path.join("bin").join("java");
    info!("Using java path: {:?}", java_path);
    Ok(java_path)
}

async fn download_java_version(java_dir: &Path, java: JavaVersion) -> ManifestResult<PathBuf> {
    info!("Downloading java version manifest");
    let java_version_manifest: HashMap<String, JavaManifest> =
        download_json_object_from_url(JAVA_VERSION_MANIFEST_URL).await?;
    let manifest_key = determine_key_for_java_manifest(&java_version_manifest);

    let java_manifest = &java_version_manifest.get(manifest_key).unwrap();
    let runtime_opt = match java.component.as_str() {
        "java-runtime-alpha" => &java_manifest.java_runtime_alpha,
        "java-runtime-beta" => &java_manifest.java_runtime_beta,
        "java-runtime-gamma" => &java_manifest.java_runtime_gamma,
        "jre-legacy" => &java_manifest.jre_legacy,
        "minecraft-java-exe" => &java_manifest.minecraft_java_exe,
        _ => unreachable!(
            "No such runtime found for java component: {}",
            &java.component
        ),
    };
    info!("Downloading runtime: {:#?}", runtime_opt);
    match runtime_opt {
        Some(runtime) => {
            // let runtime_manifest = &runtime.manifest;
            Ok(download_java_from_runtime_manifest(java_dir, runtime).await?)
        }
        None => {
            let s = format!("Java runtime is empty for component {}", &java.component);
            error!("{}", s);
            Err(ManifestError::VersionRetrievalError(s))
        }
    }
}

type PatchingResult<T> = Result<T, PatchingError>;

#[derive(Debug)]
enum PatchingError {
    Parse(xmltree::ParseError),
    Write(xmltree::Error),
    ElementAccess(String),
}

impl From<xmltree::ParseError> for PatchingError {
    fn from(error: xmltree::ParseError) -> Self {
        PatchingError::Parse(error)
    }
}

impl From<xmltree::Error> for PatchingError {
    fn from(error: xmltree::Error) -> Self {
        PatchingError::Write(error)
    }
}

/// Patch the logging configurations to replace LegacyXMLLayout with PatternLayout
fn patch_logging_configuration(bytes: &Bytes) -> PatchingResult<Bytes> {
    info!("Patching logging configuration");
    let mut root = Element::parse(&*bytes.to_vec())?;

    let pattern_layout = root
        .get_child("Appenders")
        .and_then(|element| element.get_child("RollingRandomAccessFile"))
        .and_then(|element| element.get_child("PatternLayout"))
        .ok_or(PatchingError::ElementAccess(
            "Error trying to access PatternLayout in logging configuration".into(),
        ))?
        .clone();

    let console = root
        .get_mut_child("Appenders")
        .and_then(|element| element.get_mut_child("Console"))
        .ok_or(PatchingError::ElementAccess(
            "Error trying to access Console in logging configuration".into(),
        ))?;

    console.children.clear();
    console.children.push(XMLNode::Element(pattern_layout));

    let mut result: Vec<u8> = Vec::new();
    root.write(&mut result)?;

    Ok(Bytes::from(result))
}

/// Downloads a logging configureation into ${app_dir}/assets/objects/**first two hash chars**/${logging_configuration.id}
async fn download_logging_configurations(
    asset_objects_dir: &Path,
    logging: &Logging,
) -> ManifestResult<(String, PathBuf)> {
    let client_logger = &logging.client;
    info!(
        "Downloading logging configuration {}",
        client_logger.file_id()
    );
    let original_bytes = download_bytes_from_url(client_logger.file_url()).await?;

    let patched_bytes = match patch_logging_configuration(&original_bytes) {
        Ok(b) => b,
        Err(e) => {
            warn!("{:#?}", e);
            original_bytes
        }
    };
    let hash = hash_bytes_sha1(&patched_bytes);
    let first_two_chars = hash.split_at(2);
    let objects_dir = &asset_objects_dir.join(first_two_chars.0);
    fs::create_dir_all(objects_dir)?;

    let path = objects_dir.join(client_logger.file_id());
    let mut file = File::create(&path)?;
    file.write_all(&patched_bytes)?;
    Ok((client_logger.argument.clone(), path))
}

async fn download_assets(
    instance_dir: &Path,
    asset_dir: &Path,
    asset_index: &AssetIndex,
) -> ManifestResult<String> {
    let metadata = &asset_index.metadata;
    let asset_object: AssetObject = download_json_object_from_url(metadata.url()).await?;
    let asset_index_dir = asset_dir.join("indexes");
    let index_bytes = download_bytes_from_url(metadata.url()).await?;
    fs::create_dir_all(&asset_index_dir)?;

    info!("Asset Index ID: {:?}", &asset_index);

    let asset_index_name = format!("{}.json", asset_index.id);
    let index_path = &asset_index_dir.join(&asset_index_name);
    let mut index_file = File::create(index_path)?;
    index_file.write_all(&index_bytes)?;
    info!("Downloading {} assets", &asset_object.objects.len());

    let start = Instant::now();

    // TODO: Dont download resources into each instance path directly, instead download once into
    // ${assets_dir}/resources and copy into instance dir.
    let asset_objects_dir = if asset_index.id == "legacy" {
        asset_dir.join("virtual").join("legacy")
    } else if asset_index.id == "pre-1.6" {
        instance_dir.join("resources")
    } else {
        asset_dir.join("objects")
    };

    fs::create_dir_all(&asset_objects_dir)?;

    let x = buffered_download_stream(&asset_object.objects, &asset_objects_dir, |bytes, asset| {
        if !validate_hash_sha1(bytes, asset.hash()) {
            let err = format!(
                "Error downloading asset {}, expected {} but got {}",
                &asset.name(),
                &asset.hash(),
                hash_bytes_sha1(bytes)
            );
            error!("{}", err);
            return Err(DownloadError::InvalidFileHash(err));
        }
        let path = asset.path(&asset_objects_dir);

        fs::create_dir_all(path.parent().unwrap())?;

        debug!("Bulk Download asset path: {:#?}", &path);
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        Ok(())
    })
    .await;
    info!(
        "Finished downloading assets in {}ms - {:#?}",
        start.elapsed().as_millis(),
        &x
    );
    Ok(asset_index.id.clone())
}

fn extract_natives(
    instance_dir: &Path,
    libraries_dir: &Path,
    classifiers: Vec<DownloadableClassifier>,
) -> ManifestResult<()> {
    debug!("Extracting Natives");
    for classifier in classifiers {
        debug!("Classifier: {:#?}", classifier);
        let classifier_path = classifier.path(libraries_dir);
        let natives_path = instance_dir.join("natives");
        let jar_file = File::open(&classifier_path);
        debug!("Jar File: {:#?} at {}", jar_file, classifier_path.display());
        let mut archive = ZipArchive::new(jar_file.unwrap())?;

        'zip: for i in 0..archive.len() {
            debug!("In loop");
            if let Ok(mut file) = archive.by_index(i) {
                debug!("File is ok");
                if file.is_dir() {
                    continue;
                }
                let zip_path = match file.enclosed_name() {
                    Some(name) => name.to_owned(),
                    None => continue,
                };

                debug!("ZipArchive Path: {}", zip_path.display());
                // If the zip path starts with (or is) an excluded path, dont extract it.
                if let Some(extraction_rule) = &classifier.extraction_rule {
                    for exclusion in &extraction_rule.exclude {
                        if zip_path.starts_with(exclusion) {
                            debug!("Excluding {}", exclusion);
                            continue 'zip;
                        }
                    }
                }
                let path = natives_path.join(zip_path);
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                debug!("Copy from {:#?} to {:#?}", file.name(), path.display());
                let mut output_file = File::create(&path)?;
                io::copy(&mut file, &mut output_file)?;
            }
        }
    }
    Ok(())
}

/// Applies library rules from the manifest and also patches
/// forge universal library where the url is empty.
fn apply_library_rules(libraries: Vec<Library>) -> Vec<Library> {
    libraries
        .into_iter()
        .filter_map(|lib| {
            // If we have any rules...
            if let Some(rules) = &lib.rules {
                // and the rules dont match
                if !rules_match(rules) {
                    // remove
                    None
                } else {
                    // Otherwise keep lib in download list
                    Some(lib)
                }
            } else {
                // Otherwise keep lib in download list
                Some(lib)
            }
        })
        .collect()
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub enum ModloaderType {
    Forge,
    Fabric,
    None,
}

impl From<&str> for ModloaderType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "forge" => ModloaderType::Forge,
            "fabric" => ModloaderType::Fabric,
            _ => ModloaderType::None,
        }
    }
}

impl ToString for ModloaderType {
    fn to_string(&self) -> String {
        match &self {
            ModloaderType::Forge => "forge".into(),
            ModloaderType::Fabric => "fabric".into(),
            ModloaderType::None => "".into(),
        }
    }
}

/// Seperates libraries with empty url into a different vec.
/// Returns a tuple of (<empty url>, <remaining libraries>)
fn seperate_nondownloadables(libraries: Vec<Library>) -> (Vec<Library>, Vec<Library>) {
    // Pull out forge libraries with empty url's so they can be extracted from the installer
    libraries.into_iter().partition(|library| {
        if let Some(artifact) = &library.downloads.artifact {
            artifact.url().is_empty()
        } else {
            false
        }
    })
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSettings {
    pub instance_name: String,
    pub vanilla_version: String,
    #[serde(deserialize_with = "as_modloader_type")]
    pub modloader_type: ModloaderType,
    pub modloader_version: String,
    pub instance_icon: Option<PathBuf>,
    additional_jvm_arguments: String,
    java_path_override: String,
    resolution_width: String,
    resolution_height: String,
    start_window_maximized: bool,
    record_playtime: bool,
    show_recorded_playtime: bool,
    override_options_txt: bool,
    override_servers_dat: bool,
}

fn as_modloader_type<'de, D>(deserializer: D) -> Result<ModloaderType, D::Error>
where
    D: Deserializer<'de>,
{
    let modloader_str: String = Deserialize::deserialize(deserializer)?;
    Ok(ModloaderType::from(modloader_str.as_str()))
}

impl InstanceSettings {
    pub fn new(
        instance_name: String,
        vanilla_version: String,
        modloader_type: ModloaderType,
        modloader_version: String,
        instance_icon: Option<PathBuf>,
    ) -> Self {
        Self {
            instance_name,
            vanilla_version,
            modloader_type,
            modloader_version,
            instance_icon,
            additional_jvm_arguments: "".into(),
            java_path_override: "".into(),
            resolution_width: "800".into(),
            resolution_height: "600".into(),
            start_window_maximized: false,
            record_playtime: true,
            show_recorded_playtime: true,
            override_options_txt: false,
            override_servers_dat: false,
        }
    }
}

pub async fn create_instance(
    settings: InstanceSettings,
    app_handle: &AppHandle<Wry>,
    author: Option<&str>,
) -> ManifestResult<()> {
    let resource_manager = ResourceManager::from_app_handle(&app_handle).await;
    let start = Instant::now();

    let version: VanillaVersion = resource_manager
        .download_vanilla_version(&settings.vanilla_version)
        .await?;

    // java versions is optional for versions 1.6.4 and older. We select java 8 for them by default.
    let java_version = match version.java_version {
        Some(version) => version,
        None => JavaVersion {
            component: "jre-legacy".into(),
            major_version: 8,
        },
    };

    let java_path = if settings.java_path_override.is_empty() {
        download_java_version(&resource_manager.java_dir(), java_version).await?
    } else {
        PathBuf::from(settings.java_path_override)
    };

    // Init vec of libraries to download.
    let mut all_libraries: Vec<Box<dyn Downloadable + Send + Sync>> = Vec::new();

    let vanilla_libraries = apply_library_rules(version.libraries);

    let mut vanilla_arguments = version.arguments;

    let library_data = separate_classifiers_from_libraries(vanilla_libraries);
    all_libraries.extend(library_data.downloadables);

    let mut main_class = version.main_class;

    let mut library_paths: Vec<PathBuf> = Vec::new();

    let game_jar_path = download_game_jar(
        &resource_manager.version_dir(),
        JarType::Client,
        &version.downloads.client,
        &version.id,
    )
    .await?;

    // Future that, if present, will be executed after all libraries have been downloaded.
    let mut deferred_forge_patcher: Option<BoxFuture<Result<(), io::Error>>> = None;

    // Temp dir for extracting forge installer into, closed/deleted at end of function.
    let tmp_dir = TempDir::new("temp")?;

    let modloader_launch_arguments = match settings.modloader_type {
        ModloaderType::Fabric => {
            let profile =
                download_fabric_profile(&settings.vanilla_version, &settings.modloader_version)
                    .await?;
            main_class = profile.main_class;
            for fabric_library in obtain_fabric_library_hashes(&profile.libraries).await? {
                all_libraries.push(Box::new(fabric_library));
            }
            Some(profile.arguments)
        }
        ModloaderType::Forge => {
            let forge_hashes = download_forge_hashes(&settings.modloader_version).await?;
            let forge_installer_profile = download_forge_version(
                &settings.modloader_version,
                &settings.vanilla_version,
                forge_hashes.installer_hash(),
                &resource_manager.version_dir(),
                tmp_dir.path(),
            )
            .await?;

            let arguments: Option<LaunchArguments> = match forge_installer_profile {
                ForgeInstallerProfile::Profile112 { version, profile } => {
                    main_class = version.metadata.main_class;
                    // Find the path to the forge universal jar from the profile jars list
                    let forge_universal_path = profile
                        .libraries
                        .iter()
                        .map(|library| library.name.clone())
                        .find(|name| name.starts_with("net.minecraftforge:forge:"));

                    debug!("forge_universal_path: {:#?}", forge_universal_path);

                    // Pull out forge libraries with empty url's so they can be extracted from the installer
                    let (forge_version_jars, remaining_version_libraries) =
                        seperate_nondownloadables(version.libraries);
                    let (forge_profile_jars, remaining_profile_libraries) =
                        seperate_nondownloadables(profile.libraries);

                    if remaining_version_libraries
                        .iter()
                        .any(|library| library.name.contains("log4j"))
                    {
                        // Filter out log4j-core and log4j-api versions from minecraft.
                        // This fixes an issue with minecraft providing different versions of log4j-core and log4j-api which
                        // conflict with the forge log4j libraries in the classpath.
                        all_libraries.retain(|library| {
                            let url = library.url();
                            !(url.contains("log4j") && url.contains("libraries.minecraft.net"))
                        });
                    }

                    // Pull jars out of extracted installer
                    for jar in forge_version_jars
                        .into_iter()
                        .chain(forge_profile_jars.into_iter())
                    {
                        // Can unwrap here since the option was checked in the partition above
                        let artifact = jar.downloads.artifact.unwrap();
                        // Create path to jar in the extracted installer
                        let archive_path = artifact.path(&tmp_dir.path().join("maven"));
                        let library_path = artifact.path(&resource_manager.libraries_dir());
                        if let Some(parent) = library_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        fs::copy(archive_path, &library_path)?;
                        library_paths.push(library_path);
                    }

                    let filtered_libraries = apply_library_rules(remaining_version_libraries);
                    // If it is possible for forge libraries to have classifiers we are ignoring them here.
                    let forge_library_data =
                        separate_classifiers_from_libraries(filtered_libraries);

                    all_libraries.extend(forge_library_data.downloadables);

                    // Download libraries used for forge processors without adding them to game's classpath
                    download_libraries(
                        &resource_manager.libraries_dir(),
                        &separate_classifiers_from_libraries(remaining_profile_libraries)
                            .downloadables,
                    )
                    .await?;

                    let forge_installer_paths = InstallerArgumentPaths {
                        libraries_path: resource_manager.libraries_dir(),
                        versions_dir_path: resource_manager.version_dir(),
                        minecraft_version: settings.vanilla_version.clone(),
                        forge_loader_version: settings.modloader_version.clone(),
                        tmp_dir: tmp_dir.path().to_path_buf(),
                    };

                    deferred_forge_patcher = Some(Box::pin(async {
                        patch_forge(
                            &java_path.clone(),
                            profile.processors,
                            profile.data,
                            forge_universal_path,
                            forge_installer_paths,
                        )
                    }));
                    Some(version.metadata.arguments)
                }
                ForgeInstallerProfile::Profile111(profile) => {
                    let version = profile.version_info;

                    for library in version.libraries {
                        all_libraries.push(Box::new(library));
                    }

                    // Forge versions <= 1.11 supply the entire launch argument string, including
                    // the vanilla arguments. We can overwrite the vanilla arguments and return no
                    // modloader arguments.
                    vanilla_arguments = version.metadata.arguments;
                    None
                }
            };

            arguments
        }
        _ => None,
    };

    library_paths.extend(
        download_libraries(&resource_manager.libraries_dir(), &all_libraries)
            .await?
            .drain(..)
            .collect::<HashSet<_>>()
            .into_iter()
            .filter(|path| {
                // Filter out the classifier paths from the library paths since they were all donwloaded together but cannot be part of the
                // launch argument's classpath.
                let found = library_data.classifiers.iter().find(|classifier| {
                    let classifier_path = classifier.path(&resource_manager.libraries_dir());
                    classifier_path == *path
                });
                found.is_none()
            })
            .collect::<Vec<PathBuf>>(),
    );

    if let Some(future) = deferred_forge_patcher {
        future.await?;
    }

    let logging: Option<_> = if let Some(logging_config) = version.logging {
        Some(
            download_logging_configurations(&resource_manager.asset_objects_dir(), &logging_config)
                .await?,
        )
    } else {
        None
    };
    let instance_dir = resource_manager
        .instances_dir()
        .join(&settings.instance_name);
    fs::create_dir_all(&instance_dir)?;

    let asset_index = download_assets(
        &instance_dir,
        &resource_manager.assets_dir(),
        &version.asset_index,
    )
    .await?;

    let mc_version_manifest =
        resource_manager.get_vanilla_manifest_from_version(&settings.vanilla_version);
    if mc_version_manifest.is_none() {
        warn!(
            "Could not retrieve manifest for unknown version: {}.",
            &settings.vanilla_version
        );
    }
    let persitent_arguments = construct_arguments(
        main_class,
        settings.additional_jvm_arguments,
        (settings.resolution_width, settings.resolution_height),
        &vanilla_arguments,
        modloader_launch_arguments,
        &settings.modloader_type,
        mc_version_manifest.unwrap(),
        &asset_index,
        LaunchArgumentPaths {
            logging,
            library_paths,
            instance_path: instance_dir.clone(),
            jar_path: game_jar_path,
            asset_dir_path: resource_manager.assets_dir(),
            library_directory: resource_manager.libraries_dir(),
        },
    );
    debug!("Persistent Arguments: {}", &persitent_arguments.join(" "));

    let instance_manager = InstanceManager::from_app_handle(&app_handle).await;

    // If there is no modloader, then set the "modloader_version" to the vanilla version for displaying
    // on the instances screen
    let instance_version = if settings.modloader_type == ModloaderType::None {
        settings.vanilla_version
    } else {
        settings.modloader_version
    };

    instance_manager.add_instance(InstanceConfiguration {
        instance_name: settings.instance_name,
        jvm_path: java_path.clone(),
        arguments: persitent_arguments,
        modloader_type: settings.modloader_type,
        modloader_version: instance_version,
        author: author.unwrap_or("You").into(),
        instance_icon: None,
        playtime: 0,
    })?;
    debug!("After persistent args");
    extract_natives(
        &instance_dir,
        &resource_manager.libraries_dir(),
        library_data.classifiers,
    )?;
    info!(
        "Finished download instance in {}ms",
        start.elapsed().as_millis()
    );
    tmp_dir.close()?;
    app_handle.emit_to("main", "instance-done", "").unwrap();
    Ok(())
}
