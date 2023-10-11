use iced::subscription;
use serde_json::Value;
use std::{
    env,
    fs::{self, File},
    hash::Hash,
    io::{BufRead, BufReader, Read},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread::{self, JoinHandle},
};

pub enum State {
    Checking(Option<GameSettings>),
    Launching(GameSettings),
    GettingLogs((Receiver<String>, JoinHandle<()>)),
    Idle,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Progress {
    Checked(Option<Missing>),
    Started,
    GotLog(String),
    Finished,
    Errored(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Missing {
    Java8,
    Java17,
    VersionFiles(Vec<String>),
    VanillaVersion(String, String),
    VanillaJson(String, String),
}

pub fn start<I: 'static + Hash + Copy + Send + Sync>(
    id: I,
    game_settings: Option<&GameSettings>,
) -> iced::Subscription<(I, Progress)> {
    subscription::unfold(
        id,
        State::Checking(game_settings.to_owned().clone().cloned()),
        move |state| launcher(id, state),
    )
}
#[derive(Debug, PartialEq, Clone)]
pub struct GameSettings {
    pub username: String,
    pub game_version: String,
    pub jvm: String,
    pub jvmargs: Vec<String>,
    pub ram: f64,
    pub game_directory: String,
    pub autojava: bool,
    pub game_wrapper_commands: Vec<String>,
}
async fn launcher<I: Copy>(id: I, state: State) -> ((I, Progress), State) {
    match state {
        State::Checking(game_settings) => {
            let game_settings = game_settings.unwrap();
            let minecraft_dir = get_minecraft_dir();

            // game json file
            let jsonpathstring = format!(
                "{}/versions/{}/{}.json",
                &minecraft_dir, game_settings.game_version, game_settings.game_version
            );
            let jsonpath = Path::new(&jsonpathstring);
            let mut json_file = match File::open(jsonpath) {
                Ok(file) => file,
                Err(err) => {
                    println!("Failed to open {:?}: {:?}", jsonpath, err);
                    panic!("no!!!")
                }
            };
            let mut json_file_content = String::new();
            json_file.read_to_string(&mut json_file_content).unwrap();
            let content = serde_json::from_str(&json_file_content);
            let mut p: Value = content.unwrap();

            // check for missing json
            if let Some(vanilla_ver) = p["inheritsFrom"].as_str() {
                if !Path::new(&format!(
                    "{}/versions/{}/{}.json",
                    minecraft_dir, game_settings.game_version, vanilla_ver
                ))
                .exists()
                {
                    println!("Vanilla Json needs to be downloaded.");
                    return (
                        (
                            id,
                            Progress::Checked(Some(Missing::VanillaJson(
                                vanilla_ver.to_string(),
                                format!(
                                    "{}/versions/{}",
                                    minecraft_dir, game_settings.game_version
                                ),
                            ))),
                        ),
                        State::Idle,
                    );
                }
            }

            let modded = !p["inheritsFrom"].is_null();
            if modded {
                let mut vanilla_json_content = String::new();

                let mut vanilla_json_file = match File::open(format!(
                    "{}/versions/{}/{}.json",
                    minecraft_dir,
                    game_settings.game_version,
                    p["inheritsFrom"].as_str().unwrap()
                )) {
                    Ok(ok) => ok,
                    Err(_) => panic!("no!!!"),
                };

                vanilla_json_file
                    .read_to_string(&mut vanilla_json_content)
                    .unwrap();

                let content = serde_json::from_str(&vanilla_json_content);
                p = content.unwrap();

                let version_jar_file_path = format!(
                    "{}/versions/{}/{}.jar",
                    minecraft_dir, game_settings.game_version, game_settings.game_version
                );
                let version_jar_file = match File::open(&version_jar_file_path) {
                    Ok(ok) => ok,
                    Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
                };

                if !Path::new(&version_jar_file_path).exists()
                    || version_jar_file.metadata().unwrap().len() == 0
                {
                    return (
                        (
                            id,
                            Progress::Checked(Some(Missing::VanillaVersion(
                                p["downloads"]["client"]["url"]
                                    .as_str()
                                    .unwrap()
                                    .to_string(),
                                version_jar_file_path,
                            ))),
                        ),
                        State::Idle,
                    );
                }
            }

            // check for java
            if game_settings.autojava {
                if p["javaVersion"]["majorVersion"].as_i64().unwrap_or(0) > 8
                    && !Path::new(&format!("{}/siglauncher_java/java17", minecraft_dir)).exists()
                {
                    return ((id, Progress::Checked(Some(Missing::Java17))), State::Idle);
                } else if p["javaVersion"]["majorVersion"].as_i64().unwrap_or(0) == 8
                    && !Path::new(&format!("{}/siglauncher_java/java8", minecraft_dir)).exists()
                {
                    return ((id, Progress::Checked(Some(Missing::Java8))), State::Idle);
                }
            }

            (
                (id, Progress::Checked(None)),
                State::Launching(game_settings),
            )
        }
        State::Launching(game_settings) => {
            println!("Creating game command.");
            let minecraft_directory = get_minecraft_dir();

            let game_dir = if game_settings.game_directory == *"Default" {
                env::set_current_dir(&minecraft_directory).expect("Failed to open profile folder!");
                minecraft_directory.clone()
            } else {
                let gamedirpath = format!(
                    "{}/siglauncher_profiles/{}",
                    minecraft_directory, game_settings.game_directory
                );
                fs::create_dir_all(&gamedirpath).unwrap();
                env::set_current_dir(&gamedirpath).expect("Failed to open profile folder!");
                gamedirpath
            };

            let assets_dir = format!("{}/assets", &minecraft_directory);

            let version_path = format!(
                "{}/versions/{}/{}.jar",
                &minecraft_directory, game_settings.game_version, game_settings.game_version
            );

            // json file {
            let jsonpathstring = format!(
                "{}/versions/{}/{}.json",
                &minecraft_directory, game_settings.game_version, game_settings.game_version
            );
            let jsonpath = Path::new(&jsonpathstring);
            let mut json_file = match File::open(jsonpath) {
                Ok(ok) => ok,
                Err(e) => return ((id, Progress::Errored(e.to_string())), State::Idle),
            };
            let mut json_file_content = String::new();
            json_file.read_to_string(&mut json_file_content).unwrap();
            let content = serde_json::from_str(&json_file_content);
            let p: Value = content.unwrap();
            // } json file

            let main_class = &p["mainClass"].as_str().unwrap();
            let asset_index = p["assets"].as_str().unwrap_or("").to_string();
            let native_directory = format!(
                "{}/versions/{}/natives",
                &minecraft_directory, game_settings.game_version
            );

            let mut library_list = libmanager(&p);

            let mut version_jvm_args = get_game_jvm_args(&p, &native_directory);

            //
            let mut version_game_args = vec![];

            // this is used to get game args.
            let gamedata = vec![
                game_settings.username,
                game_settings.game_version.clone(),
                game_dir.to_string(),
                assets_dir,
                asset_index,
                String::from("000"),
                String::from("[pro]"),
                String::from("{}"),
                String::from("legacy"),
                String::from("Release"),
                String::from("Modified"),
                library_list.clone(),
            ];

            let is_modded = if game_settings.game_version.to_lowercase().contains("fabric")
                || game_settings.game_version.to_lowercase().contains("forge")
                || !p["inheritsFrom"].is_null()
            {
                let (modded_jvm_args, modded_game_args, vanilla_version_library_list) =
                    modded(&p, &game_settings.game_version, gamedata.clone());
                version_jvm_args.extend(modded_jvm_args);
                library_list.push_str(&vanilla_version_library_list);

                version_game_args = modded_game_args;
                true
            } else {
                false
            };

            let (java_path, java_args) = match game_settings.autojava {
                true => automatic_java(p.clone(), &game_settings.game_version, is_modded),
                false => (game_settings.jvm, game_settings.jvmargs),
            };

            library_list.push_str(&format!(
                "{}/versions/{}/{}.jar",
                &minecraft_directory, game_settings.game_version, game_settings.game_version
            ));

            if let Some(arguments) = p["arguments"]["game"].as_array() {
                let mut str_arguments = vec![];
                for i in arguments {
                    if i.is_string() {
                        str_arguments.push(i.as_str().unwrap_or("").to_owned())
                    } else {
                        if i["value"].is_string() {
                            str_arguments.push(i["value"].as_str().unwrap().to_owned())
                        }
                    }
                }

                version_game_args.extend_from_slice(&get_game_args(str_arguments, &gamedata));
            } else if let Some(arguments) = p["minecraftArguments"].as_str() {
                let oldargs: Vec<String> = arguments
                    .to_string()
                    .split_whitespace()
                    .map(String::from)
                    .collect();

                version_game_args.extend_from_slice(&get_game_args(oldargs, &gamedata))
            }

            let mut wrapper_commands = game_settings.game_wrapper_commands;
            let has_wrapper_commands;

            let mut game_command = if !wrapper_commands.is_empty() {
                has_wrapper_commands = true;
                Command::new(wrapper_commands.remove(0))
            } else {
                has_wrapper_commands = false;
                Command::new(&java_path)
            };

            if has_wrapper_commands {
                game_command.args(wrapper_commands).arg(&java_path);
            }

            game_command
                .arg(format!("-Xmx{}M", game_settings.ram * 1024.))
                .args(java_args.clone())
                .arg("-cp")
                .arg(library_list.clone())
                .args(version_jvm_args.clone())
                .arg(main_class)
                .args(version_game_args.clone());

            println!("{:?}", game_command);

            println!("Launching game process");
            if command_exists(game_command.get_program().to_str().unwrap()) {
                let game_process_receiver = run_and_log_game(game_command);
                return if let Ok(game_pr_rec) = game_process_receiver.await {
                    ((id, Progress::Started), State::GettingLogs(game_pr_rec))
                } else {
                    (
                        (
                            id,
                            Progress::Errored("Failed to start game process.".to_owned()),
                        ),
                        State::Idle,
                    )
                };
            } else {
                (
                    (
                        id,
                        Progress::Errored("Java or wrapper doesn't exist".to_owned()),
                    ),
                    State::Idle,
                )
            }
        }

        State::GettingLogs(receiver) => {
            return if let Ok(log_line) = receiver.0.recv() {
                (
                    (id, Progress::GotLog(log_line)),
                    State::GettingLogs(receiver),
                )
            } else {
                receiver.1.join().expect("Failed to join child thread");
                ((id, Progress::Finished), State::Idle)
            }
        }

        State::Idle => iced::futures::future::pending().await,
    }
}

async fn run_and_log_game(
    mut game_command: Command,
) -> std::io::Result<(Receiver<String>, JoinHandle<()>)> {
    let (sender, receiver) = mpsc::channel();

    let child_thread = thread::spawn(move || {
        let mut child = game_command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start game process.");

        if let Some(stdout) = child.stdout.take() {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        sender.send(line).expect("Failed to send log line");
                    }
                    Err(err) => eprintln!("Error reading child output: {}", err),
                }
            }
        }

        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        sender.send(line).expect("Failed to send log line");
                    }
                    Err(err) => eprintln!("Error reading child output: {}", err),
                }
            }
        }

        let status = child.wait().expect("Failed to wait for child process");
        println!("Child process exited with: {}", status);
    });

    Ok((receiver, child_thread))
}

// Utility functions {
pub fn get_minecraft_dir() -> String {
    match std::env::consts::OS {
        "linux" => format!("{}/.minecraft", std::env::var("HOME").unwrap()),
        "windows" => format!(
            "{}/AppData/Roaming/.minecraft",
            std::env::var("USERPROFILE").unwrap().replace('\\', "/")
        ),
        _ => panic!("System not supported."),
    }
}

pub fn getinstalledversions() -> Vec<String> {
    let versions_dir = format!("{}/versions", get_minecraft_dir());

    if !Path::new(&versions_dir).exists() {
        fs::create_dir_all(&versions_dir).unwrap();
    }
    let entries = fs::read_dir(versions_dir).unwrap();

    entries
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            if path.is_dir() {
                Some(path.file_name().unwrap().to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}
// } Utility functions

// Launch functions {
fn get_game_args(arguments: Vec<String>, gamedata: &[String]) -> Vec<String> {
    let mut version_game_args = vec![];
    for i in arguments {
        match i.as_str() {
            "${auth_player_name}" => version_game_args.push(gamedata[0].clone()),
            "${version_name}" => version_game_args.push(gamedata[1].clone()),
            "${game_directory}" => version_game_args.push(gamedata[2].clone()),
            "${assets_root}" => version_game_args.push(gamedata[3].clone()),
            "${assets_index_name}" => version_game_args.push(gamedata[4].clone()),
            "${auth_uuid}" => version_game_args.push(gamedata[5].clone()),
            "${auth_session}" => version_game_args.push(gamedata[5].clone()),
            "${clientid}" => version_game_args.push(gamedata[5].clone()),
            "${auth_xuid}" => version_game_args.push(gamedata[5].clone()),
            "${auth_access_token}" => version_game_args.push(gamedata[6].clone()),
            "${user_properties}" => version_game_args.push(gamedata[7].clone()),
            "${user_type}" => version_game_args.push(gamedata[8].clone()),
            "${version_type}" => version_game_args.push(gamedata[9].clone()),
            "${classpath}" => version_game_args.push(gamedata[10].clone()),
            "${game_assets}" => {
                version_game_args.push(format!("{}/resources", get_minecraft_dir()))
            }

            "--demo" => {}
            _ => version_game_args.push(i.to_owned()),
        }
    }
    version_game_args
}

fn get_game_jvm_args(p: &Value, nativedir: &str) -> Vec<String> {
    let mut version_jvm_args = vec![];
    if let Some(arguments) = p["arguments"]["jvm"].as_array() {
        for i in arguments {
            if i.is_string() {
                let value = i.as_str().unwrap();

                if value.contains("${natives_directory}") {
                    let value = value.replace("${natives_directory}", &nativedir);
                    version_jvm_args.push(value);
                } else if value == "${classpath}" || value == "-cp" {
                } else {
                    version_jvm_args.push(value.to_owned())
                }
            }
        }
    } else {
        version_jvm_args.push(format!("-Djava.library.path={}", &nativedir))
    }
    version_jvm_args
}

fn automatic_java(mut p: Value, game_version: &String, ismodded: bool) -> (String, Vec<String>) {
    let mc_dir = get_minecraft_dir();

    let (autojava17path, autojava8path) = if std::env::consts::OS == "windows" {
        (
            format!("{}/siglauncher_java/java17/bin/javaw.exe", mc_dir),
            format!("{}/siglauncher_java/java8/bin/javaw.exe", mc_dir),
        )
    } else {
        (
            format!("{}/siglauncher_java/java17/bin/java", mc_dir),
            format!("{}/siglauncher_java/java8/bin/java", mc_dir),
        )
    };

    if ismodded {
        let vanillaversion = p["inheritsFrom"].as_str().unwrap_or(&game_version.as_str());
        let vanillajsonpathstring = format!(
            "{}/versions/{}/{}.json",
            &mc_dir, game_version, vanillaversion
        );

        let mut vanillajson = File::open(vanillajsonpathstring).unwrap();

        let mut vjsoncontent = String::new();
        vanillajson.read_to_string(&mut vjsoncontent).unwrap();
        p = serde_json::from_str(&vjsoncontent).unwrap();
    }
    let requiredjavaversion = p["javaVersion"]["majorVersion"].as_i64().unwrap_or(0);

    let java17args = "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+UseNUMA -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=400M -XX:NonNMethodCodeHeapSize=12M -XX:ProfiledCodeHeapSize=194M -XX:NonProfiledCodeHeapSize=194M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseVectorCmov -XX:+PerfDisableSharedMem -XX:+UseFastUnorderedTimeStamps -XX:+UseCriticalJavaThreadPriority -XX:ThreadPriorityPolicy=1 -XX:AllocatePrefetchStyle=3 -XX:+UseShenandoahGC -XX:ShenandoahGCMode=iu -XX:ShenandoahGuaranteedGCInterval=1000000 -XX:AllocatePrefetchStyle=1";
    let java8args = "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+ParallelRefProcEnabled -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:+AggressiveOpts -XX:MaxInlineLevel=15 -XX:MaxVectorSize=32 -XX:ThreadPriorityPolicy=1 -XX:+UseNUMA -XX:+UseDynamicNumberOfGCThreads -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=350M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -Dgraal.CompilerConfiguration=community";

    if requiredjavaversion > 8 || requiredjavaversion == 0 {
        (
            autojava17path,
            java17args.split(' ').map(|s| s.to_owned()).collect(),
        )
    } else {
        (
            autojava8path,
            java8args.split(' ').map(|s| s.to_owned()).collect(),
        )
    }
}

fn libmanager(p: &Value) -> String {
    let os = std::env::consts::OS;

    let mc_dir = get_minecraft_dir();
    let mut library_list = String::new();

    if let Some(libraries) = p["libraries"].as_array() {
        let lib_dir = format!("{}/libraries/", &mc_dir);
        let separator = match os {
            "linux" => ':',
            "windows" => ';',
            _ => panic!(),
        };

        for library in libraries {
            if library["rules"][0]["os"]["name"] == os
                || library["rules"][0]["os"]["name"].is_null()
            {
                let libraryname = library["name"].as_str().unwrap();
                let mut lpieces: Vec<&str> = libraryname.split(':').collect();
                let firstpiece = lpieces[0].replace('.', "/");
                lpieces.remove(0);

                if library["name"].as_str().unwrap().contains("natives") {
                    lpieces.remove(lpieces.len() - 1);

                    let libpath = format!(
                        "{}{}/{}/{}-{}-natives-{}.jar",
                        lib_dir,
                        &firstpiece,
                        &lpieces.join("/"),
                        &lpieces[&lpieces.len() - 2],
                        &lpieces[&lpieces.len() - 1],
                        os
                    );

                    library_list.push_str(&libpath);
                    library_list.push(separator);
                } else if library["natives"][os].is_null() {
                    let libpath = format!(
                        "{}{}/{}/{}-{}.jar",
                        lib_dir,
                        &firstpiece,
                        &lpieces.join("/"),
                        &lpieces[&lpieces.len() - 2],
                        &lpieces[&lpieces.len() - 1]
                    );

                    library_list.push_str(&libpath);
                    library_list.push(separator);
                } else {
                    let libpath = format!(
                        "{}{}/{}/{}-{}-natives-{}.jar",
                        lib_dir,
                        &firstpiece,
                        &lpieces.join("/"),
                        &lpieces[&lpieces.len() - 2],
                        &lpieces[&lpieces.len() - 1],
                        os
                    );

                    library_list.push_str(&libpath);
                    library_list.push(separator);
                }
            }
        }
    }
    library_list
}

fn modded(
    p: &Value,
    game_version: &String,
    mut gamedata: Vec<String>,
) -> (Vec<String>, Vec<String>, String) {
    let mc_dir = get_minecraft_dir();

    let mut modded_game_args = vec![];

    let vanillaversion = p["inheritsFrom"].as_str().unwrap();
    let vanillajsonpathstring = format!(
        "{}/versions/{}/{}.json",
        &mc_dir, game_version, vanillaversion
    );

    let mut vanillajson = File::open(&vanillajsonpathstring).unwrap();

    let mut vjsoncontent = String::new();
    vanillajson.read_to_string(&mut vjsoncontent).unwrap();
    let vjson: Value = serde_json::from_str(&vjsoncontent).unwrap();
    let new_asset_index = vjson["assets"].as_str().unwrap().to_string();
    gamedata[4] = new_asset_index;

    if let Some(arguments) = vjson["arguments"]["game"].as_array() {
        let mut base_arguments = Vec::new();
        for i in arguments {
            if i.is_string() {
                base_arguments.push(i.as_str().unwrap().to_string())
            } else if i["value"].is_string() {
                base_arguments.push(i["value"].as_str().unwrap().to_string())
            }
        }

        modded_game_args = get_game_args(base_arguments, &gamedata)
    }

    let vanilla_version_jvm_args = get_game_jvm_args(
        &vjson,
        &format!("{}/versions/{}/natives", &mc_dir, game_version),
    );

    let vanilla_library_list = &libmanager(&vjson);

    (
        vanilla_version_jvm_args,
        modded_game_args,
        vanilla_library_list.to_string(),
    )
}
// } Launch functions

fn command_exists(command_name: &str) -> bool {
    if let Ok(paths) = env::var("PATH") {
        let path_list: Vec<_> = env::split_paths(&paths).collect();

        for path in path_list {
            let command_path = path.join(command_name);

            if let Ok(metadata) = fs::metadata(&command_path) {
                if metadata.is_file() {
                    return true;
                }
            }
        }
    }

    false
}