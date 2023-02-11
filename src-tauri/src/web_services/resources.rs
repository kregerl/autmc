use std::{
    collections::{HashMap, HashSet},
    env,
    fs::{self, File},
    hash::{self, Hash},
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use bytes::Bytes;
use chrono::format;
use log::{debug, error, info, warn};
use tauri::{AppHandle, Manager, State, Wry};
use tempdir::TempDir;
use xmltree::{Element, XMLNode};
use zip::ZipArchive;

use crate::{
    consts::{JAVA_VERSION_MANIFEST, LAUNCHER_NAME, LAUNCHER_VERSION},
    state::{
        account_manager::Account,
        instance_manager::{InstanceConfiguration, InstanceState},
        resource_manager::{ManifestError, ManifestResult, ResourceState},
    },
    web_services::{
        downloader::{
            boxed_buffered_download_stream, buffered_download_stream, download_bytes_from_url,
            download_json_object, validate_hash, DownloadError, Downloadable,
        },
        manifest::{
            fabric::{download_fabric_profile, obtain_fabric_library_hashes},
            forge::{
                self, download_forge_hashes, download_forge_version, patch_forge,
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
    downloader::{hash_bytes, validate_file_hash},
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
            error!("Implement feature rules for arguments");
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

// HACK: This key generation to get the java version is not optimal and could
//       use to be redone. This uses architecture to map to known java manifest versions.
//       If the manifest ever changes this function most likely needs to be updated.
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
                "Unknown java version os: {}. Expected `linux`, `mac-os` or `windows`",
                key
            )
        }
    }
}
struct LaunchArgumentPaths {
    logging: (String, PathBuf),
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

    for jvm_arg in arguments.jvm.iter() {
        match jvm_arg {
            // For normal arguments, check if it has something that should be replaced and replace it
            Argument::Arg(value) => {
                let sub_arg = substitute_jvm_arguments(&value, mc_version, &argument_paths);
                formatted_arguments.push(match sub_arg {
                    Some(argument) => argument,
                    None => value.into(),
                });
            }
            // For conditional args, check their rules before adding to formatted_arguments vec
            Argument::ConditionalArg { rules, values } => {
                if !rules_match(&rules) {
                    continue;
                }
                for value in values {
                    let sub_arg = substitute_jvm_arguments(&value, mc_version, &argument_paths);
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
    let mut formatted_arguments = Vec::new();

    formatted_arguments.push(
        substitute_jvm_arguments(
            "-Djava.library.path=${natives_directory}",
            mc_version,
            &argument_paths,
        )
        .unwrap(),
    );
    formatted_arguments.push("-cp".into());
    formatted_arguments
        .push(substitute_jvm_arguments("${classpath}", mc_version, &argument_paths).unwrap());

    formatted_arguments
}

fn construct_arguments(
    main_class: String,
    arguments: &LaunchArguments,
    modloader_arguments: Option<LaunchArguments>,
    mc_version: &VanillaManifestVersion,
    asset_index: &str,
    argument_paths: LaunchArgumentPaths,
) -> Vec<String> {
    // IDEA: Vec could be 'with_capacity' if we calculate capacity first.
    let mut formatted_arguments: Vec<String> = Vec::new();
    let mut game_args: Vec<Argument> = Vec::new();

    // Create game arguments from the launch arguments.
    game_args.append(&mut match arguments {
        // Substitute values in for placeholders in the jvm arguments.

        // Versions <= 1.12  use a string of game arguments and do not provide any jvm arguments.
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
                arguments,
                &argument_paths,
                &mc_version.id,
            ));
            arguments.game.iter().map(|arg| arg.clone()).collect()
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
                arguments.game.iter().map(|arg| arg.clone()).collect()
            }
        });
    }

    // Construct the logging configuration argument
    if let Some(substr) = get_arg_substring(&argument_paths.logging.0) {
        formatted_arguments.push(
            argument_paths
                .logging
                .0
                .replace(substr, path_to_utf8_str(&argument_paths.logging.1)),
        );
    }
    // Add main class
    formatted_arguments.push(main_class);

    // Substitute values in for placeholders in the game arguments, skipping account-specific arguments.
    for game_arg in game_args.iter() {
        match game_arg {
            // For normal arguments, check if it has something that should be replaced and replace it
            Argument::Arg(value) => {
                let sub_arg =
                    substitute_game_arguments(&value, &mc_version, asset_index, &argument_paths);
                formatted_arguments.push(match sub_arg {
                    Some(argument) => argument,
                    None => value.into(),
                });
            }
            // For conditional args, check their rules before adding to formatted_arguments vec
            Argument::ConditionalArg { rules, values } => {
                if !rules_match(&rules) {
                    continue;
                }
                for value in values {
                    let sub_arg = substitute_game_arguments(
                        &value,
                        &mc_version,
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
    let classpath_strs: Vec<&str> = (&argument_paths.library_paths)
        .into_iter()
        .map(|path| path_to_utf8_str(&path))
        .collect();

    let mut formatted_argument: Option<String> = None;
    // Iterate here since some arguments(forge) can have multiple substitutions in them
    loop {
        let arg_to_replace = if let Some(argument) = &formatted_argument {
            argument.as_str()
        } else {
            arg
        };

        let substring = get_arg_substring(&arg_to_replace);

        if let Some(substr) = substring {
            info!("Substituting {} for jvm arguments", &substr);
            formatted_argument = match substr {
                "${natives_directory}" => Some(arg_to_replace.replace(
                    substr,
                    &format!(
                        "{}",
                        path_to_utf8_str(&argument_paths.instance_path.join("natives"))
                    ),
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
                "${version_name}" => Some(arg_to_replace.replace(substr, &mc_version)),
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
    mc_version: &VanillaManifestVersion,
    asset_index: &str,
    argument_paths: &LaunchArgumentPaths,
) -> Option<String> {
    let substring = get_arg_substring(arg);

    if let Some(substr) = substring {
        info!("Substituting {} for game arguments", &substr);
        match substr {
            "${version_name}" => Some(arg.replace(substr, &mc_version.id)),
            "${game_directory}" => Some(arg.replace(
                substr,
                &format!("{}", path_to_utf8_str(&argument_paths.instance_path)),
            )),
            "${assets_root}" => Some(arg.replace(
                substr,
                &format!("{}", path_to_utf8_str(&argument_paths.asset_dir_path)),
            )),
            "${assets_index_name}" => Some(arg.replace(substr, &asset_index)),
            "${user_type}" => Some(arg.replace(substr, "mojang")), // TODO: Unknown but hardcoded to "mojang" as thats what the gdlauncher example shows
            "${version_type}" => Some(arg.replace(substr, &mc_version.version_type)),
            "${resolution_width}" => None, // TODO: Launcher option specific
            "${resolution_height}" => None, // TODO: Launcher option specific
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
    active_account: &Account,
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

fn separate_classifiers_from_libraries(libraries: &[Library]) -> LibraryData {
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
        fs::create_dir(&libraries_dir)?;
    }
    let start = Instant::now();
    // Perform one buffered download for all libraries, including classifiers
    boxed_buffered_download_stream(&libraries, &libraries_dir, |bytes, artifact| {
        // FIXME: Removing file hashing makes the downloads MUCH faster. Only because of a couple slow hashes, upwards of 1s each
        if !validate_hash(&bytes, &artifact.hash()) {
            let err = format!("Error downloading {}, invalid hash.", &artifact.url());
            error!("{}", err);
            return Err(DownloadError::InvalidFileHashError(err));
        }
        debug!("Downloading library: {}", artifact.name());
        // Windows only?
        // let artifact_path = str::replace(artifact.name(), "/", "\\");
        let path = artifact.path(&libraries_dir);
        let mut file = File::create(&path)?;
        file.write_all(&bytes)?;
        Ok(())
    })
    .await?;
    info!(
        "Successfully downloaded libraries in {}ms",
        start.elapsed().as_millis()
    );
    let mut file_paths: Vec<PathBuf> = Vec::with_capacity(libraries.len());
    for library in libraries {
        file_paths.push(library.path(&libraries_dir));
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
    let dir_path = &versions_dir.join(version_id);
    fs::create_dir_all(dir_path)?;

    let path = dir_path.join(format!("{}.jar", &jar_str));
    let valid_hash = download.hash();
    // Check if the file exists and the hash matches the download's sha1.
    if !validate_file_hash(&path, valid_hash) {
        info!("Downloading {} {} jar", version_id, jar_str);
        let bytes = download_bytes_from_url(download.url()).await?;
        if !validate_hash(&bytes, valid_hash) {
            let err = format!(
                "Error downloading {} {} jar, invalid hash.",
                version_id, jar_str
            );
            error!("{}", err);
            return Err(ManifestError::InvalidFileDownload(err));
        }
        let mut file = File::create(&path)?;
        file.write_all(&bytes)?;
    }
    Ok(path)
}

// FIXME: Use an indexmap instead of a hashmap. Complete this process in a single pass since the index map is ordered correctly.
//        The correct order is important since it will create dirs before creating files in those dirs.
async fn download_java_from_runtime_manifest(
    java_dir: &Path,
    manifest: &JavaRuntime,
) -> ManifestResult<PathBuf> {
    info!("Downloading java runtime manifset");
    let version_manifest: JavaRuntimeManifest =
        download_json_object(&manifest.manifest.url()).await?;
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
    buffered_download_stream(&files, &base_path, |bytes, jrt| {
        if !validate_hash(&bytes, &jrt.hash()) {
            let err = format!("Error downloading {}, invalid hash.", &jrt.url());
            error!("{}", err);
            return Err(DownloadError::InvalidFileHashError(err));
        }
        let path = jrt.path(&base_path);
        let mut file = File::create(&path)?;
        // TODO: Change from target_os ="linux" to unix
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::prelude::PermissionsExt;

            // Markt the file as executable on unix os's
            if jrt.executable {
                let mut permissions = file.metadata()?.permissions();
                permissions.set_mode(0o775);
                file.set_permissions(permissions)?;
            }
        }
        file.write_all(&bytes)?;
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
        download_json_object(JAVA_VERSION_MANIFEST).await?;
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
            Ok(download_java_from_runtime_manifest(&java_dir, &runtime).await?)
        }
        None => {
            let s = format!("Java runtime is empty for component {}", &java.component);
            error!("{}", s);
            //TODO: New error type?
            return Err(ManifestError::VersionRetrievalError(s));
        }
    }
}

type PatchingResult<T> = Result<T, PatchingError>;

#[derive(Debug)]
enum PatchingError {
    XmlParseError(xmltree::ParseError),
    XmlWriteError(xmltree::Error),
    XmlElementAccessError(String),
}

impl From<xmltree::ParseError> for PatchingError {
    fn from(error: xmltree::ParseError) -> Self {
        PatchingError::XmlParseError(error)
    }
}

impl From<xmltree::Error> for PatchingError {
    fn from(error: xmltree::Error) -> Self {
        PatchingError::XmlWriteError(error)
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
        .ok_or(PatchingError::XmlElementAccessError(
            "Error trying to access PatternLayout in logging configuration".into(),
        ))?
        .clone();

    let console = root
        .get_mut_child("Appenders")
        .and_then(|element| element.get_mut_child("Console"))
        .ok_or(PatchingError::XmlElementAccessError(
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
    let original_bytes = download_bytes_from_url(&client_logger.file_url()).await?;

    let patched_bytes = match patch_logging_configuration(&original_bytes) {
        Ok(b) => b,
        Err(e) => {
            warn!("{:#?}", e);
            original_bytes
        }
    };
    let hash = hash_bytes(&patched_bytes);
    let first_two_chars = hash.split_at(2);
    let objects_dir = &asset_objects_dir.join(first_two_chars.0);
    fs::create_dir_all(&objects_dir)?;

    let path = objects_dir.join(format!("{}", &client_logger.file_id()));
    let mut file = File::create(&path)?;
    file.write_all(&patched_bytes)?;
    Ok((client_logger.argument.clone(), path))
}

//TODO: This probably needs to change a little to support "legacy" versions < 1.7
async fn download_assets(
    asset_dir: &Path,
    asset_objects_dir: &Path,
    asset_index: &AssetIndex,
) -> ManifestResult<String> {
    let metadata = &asset_index.metadata;
    let asset_object: AssetObject = download_json_object(metadata.url()).await?;
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

    fs::create_dir_all(&asset_objects_dir)?;

    let x = buffered_download_stream(&asset_object.objects, &asset_objects_dir, |bytes, asset| {
        if !validate_hash(&bytes, &asset.hash()) {
            let err = format!(
                "Error downloading asset {}, expected {} but got {}",
                &asset.name(),
                &asset.hash(),
                hash_bytes(&bytes)
            );
            error!("{}", err);
            return Err(DownloadError::InvalidFileHashError(err));
        }
        fs::create_dir_all(&asset.path(&asset_objects_dir).parent().unwrap())?;

        debug!(
            "Bulk Download asset path: {:#?}",
            &asset.path(&asset_objects_dir)
        );
        let mut file = File::create(&asset.path(&asset_objects_dir))?;
        file.write_all(&bytes)?;
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

fn apply_library_rules(libraries: Vec<Library>) -> Vec<Library> {
    libraries
        .into_iter()
        .filter_map(|lib| {
            // If we have any rules...
            if let Some(rules) = &lib.rules {
                // and the rules dont match
                if !rules_match(&rules) {
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

pub async fn create_instance(
    vanilla_version: String,
    modloader_type: String,
    modloader_version: String,
    instance_name: String,
    app_handle: &AppHandle<Wry>,
) -> ManifestResult<()> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;
    let start = Instant::now();

    let version: VanillaVersion = resource_manager
        .download_vanilla_version(&vanilla_version)
        .await?;

    // java versions is optional for versions 1.6.4 and older. We select java 8 for them by default.
    let java_version = match version.java_version {
        Some(version) => version,
        None => JavaVersion {
            component: "jre-legacy".into(),
            major_version: 8,
        },
    };

    let java_path = download_java_version(&resource_manager.java_dir(), java_version).await?;

    let mut all_libraries: Vec<Box<dyn Downloadable + Send + Sync>> = Vec::new();

    let vanilla_libraries = apply_library_rules(version.libraries);

    let library_data = separate_classifiers_from_libraries(&vanilla_libraries);
    all_libraries.extend(library_data.downloadables);

    let mut main_class = version.main_class;

    let mut library_paths: Vec<PathBuf> = Vec::new();

    let modloader_launch_arguments = match modloader_type.as_str() {
        "Fabric" => {
            let profile = download_fabric_profile(&vanilla_version, &modloader_version).await?;
            main_class = profile.main_class;
            for fabric_library in obtain_fabric_library_hashes(&profile.libraries).await? {
                all_libraries.push(Box::new(fabric_library));
            }
            Some(profile.arguments)
        }
        "Forge" => {
            let tmp_dir = TempDir::new("temp")?;
            // TODO: Make a tempdir here that will be deleted when it goes out of scope. Extract actual forge jar into tempdir so relative paths will work with the processors.
            let forge_hashes = download_forge_hashes(&modloader_version).await?;
            let forge_profile = download_forge_version(
                &modloader_version,
                &vanilla_version,
                Some(forge_hashes.classifiers.installer),
                &resource_manager.version_dir(),
                tmp_dir.path(),
            )
            .await?;
            let forge_version = forge_profile.version;

            main_class = forge_version.main_class;

            let forge_libraries: Vec<Library> = forge_version
                .libraries
                .into_iter()
                .chain(forge_profile.profile.libraries)
                .collect();
            let filtered_libraries = apply_library_rules(forge_libraries);
            // If it is possible for forge libraries to have classifiers we are ignoring them here.
            let data = separate_classifiers_from_libraries(&filtered_libraries);

            library_paths.extend(
                download_libraries(&resource_manager.libraries_dir(), &data.downloadables).await?,
            );

            let forge_installer_paths = InstallerArgumentPaths {
                libraries_path: resource_manager.libraries_dir(),
                versions_dir_path: resource_manager.version_dir(),
                minecraft_version: vanilla_version.clone(),
                forge_loader_version: modloader_version.clone(),
                tmp_dir: tmp_dir.path().to_path_buf()
            };

            patch_forge(
                &java_path,
                &forge_profile.profile.processors,
                &forge_profile.profile.data,
                &filtered_libraries,
                forge_installer_paths,
            );

            Some(forge_version.arguments)
        }
        _ => None,
    };

    // FIXME: Filtering duplicated libraries here, should probably be done before attempting to download
    library_paths.extend(
        download_libraries(&resource_manager.libraries_dir(), &all_libraries)
            .await?
            .drain(..)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<PathBuf>>(),
    );

    let game_jar_path = download_game_jar(
        &resource_manager.version_dir(),
        JarType::Client,
        &version.downloads.client,
        &version.id,
    )
    .await?;

    let logging =
        download_logging_configurations(&resource_manager.asset_objects_dir(), &version.logging)
            .await?;

    let asset_index = download_assets(
        &resource_manager.assets_dir(),
        &resource_manager.asset_objects_dir(),
        &version.asset_index,
    )
    .await?;
    info!(
        "Finished download instance in {}ms",
        start.elapsed().as_millis()
    );

    let instance_dir = resource_manager.instances_dir().join(&instance_name);
    fs::create_dir_all(&instance_dir)?;

    let mc_version_manifest = resource_manager.get_vanilla_manifest_from_version(&vanilla_version);
    if mc_version_manifest.is_none() {
        warn!(
            "Could not retrieve manifest for unknown version: {}.",
            &vanilla_version
        );
    }
    let persitent_arguments = construct_arguments(
        main_class,
        &version.arguments,
        modloader_launch_arguments,
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

    let instance_state: State<InstanceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let instance_manager = instance_state.0.lock().await;

    instance_manager.add_instance(InstanceConfiguration {
        instance_name: instance_name.into(),
        jvm_path: java_path,
        arguments: persitent_arguments,
    })?;
    debug!("After persistent args");
    extract_natives(
        &instance_dir,
        &resource_manager.libraries_dir(),
        library_data.classifiers,
    )?;
    Ok(())
}
