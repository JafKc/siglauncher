use serde_json::Value;
use std::{
    env::{self},
    fs::{self, File},
    io::Read,
    path::Path,
    process::Command,
};

#[cfg(target_os = "linux")]
use std::os::unix::prelude::PermissionsExt;

use crate::backend::installer::downloadjava;

pub(crate) mod installer;

#[tokio::main]
pub async fn start(
    player: &str,
    game_version: &str,
    jvm: &str,
    jvmargs: Vec<String>,
    ram: f64,
    gamemode: bool,
    gamedirectory: String,
    autojava: bool,
) -> std::io::Result<()> {
    let operationalsystem = std::env::consts::OS;
    let player = player;
    let mc_dir = get_minecraft_dir();

    let gamedir = if gamedirectory == *"Default" {
        env::set_current_dir(&mc_dir).expect("Failed to open profile folder!");
        mc_dir.clone()
    } else {
        let gamedirpath = format!("{}/siglauncher_profiles/{}", mc_dir, gamedirectory);
        fs::create_dir_all(&gamedirpath).unwrap();
        env::set_current_dir(&gamedirpath).expect("Failed to open profile folder!");
        gamedirpath
    };

    let autojavapaths = if std::env::consts::OS == "windows" {
        vec![
            format!("{}/java/java17/bin/java.exe", mc_dir),
            format!("{}/java/java8/bin/java.exe", mc_dir),
        ]
    } else {
        vec![
            format!("{}/java/java17/bin/java", mc_dir),
            format!("{}/java/java8/bin/java", mc_dir),
        ]
    };
    let mut jvmargs = jvmargs;

    println!("{}", &mc_dir);
    let assets_dir = format!("{}/assets", &mc_dir);
    let game_version = game_version;
    let uuid = "0";

    let versionpath = format!("{}/versions/{}/{}.jar", &mc_dir, game_version, game_version);

    let jpathstring = format!(
        "{}/versions/{}/{}.json",
        &mc_dir, game_version, game_version
    );
    let jsonpath = Path::new(&jpathstring);

    let mut file = File::open(jsonpath).unwrap();
    let mut fcontent = String::new();
    file.read_to_string(&mut fcontent).unwrap();
    let content = serde_json::from_str(&fcontent);

    let p: Value = content.unwrap();

    let mainclass = &p["mainClass"].as_str().unwrap();
    let mut assetindex = p["assets"].as_str().unwrap_or("").to_string();
    let nativedir = format!("{}/versions/{}/natives", &mc_dir, game_version);

    let mut libraries_list = libmanager(&p, operationalsystem, &mc_dir);
    let mut version_game_args = vec![];
    let mut moddedgameargs = vec![];

    let mut ismodded = false;
    //for modded versions
    if game_version.to_lowercase().contains("fabric")
        || game_version.to_lowercase().contains("forge")
    {
        let vanillaversion = p["inheritsFrom"].as_str().unwrap_or(game_version);
        let vanillajsonpathstring = format!(
            "{}/versions/{}/{}.json",
            &mc_dir, game_version, vanillaversion
        );
        let vanillajsonfilepath = Path::new(&vanillajsonpathstring);
        if !vanillajsonfilepath.exists() {
            installer::downloadversionjson(
                1,
                &vanillaversion.to_owned(),
                &format!("{}/versions/{}", &mc_dir, game_version),
            )
            .await
            .unwrap()
        }

        let mut vanillajson = File::open(&vanillajsonpathstring).unwrap();

        let mut vjsoncontent = String::new();
        vanillajson.read_to_string(&mut vjsoncontent).unwrap();
        let vjson: Value = serde_json::from_str(&vjsoncontent).unwrap();
        let asseti = vjson["assets"].as_str().unwrap().to_string();

        assetindex = asseti;
        ismodded = true;

        if let Some(arguments) = vjson["arguments"]["game"].as_array() {
            for i in arguments {
                moddedgameargs.push(i.to_owned())
            }
        }

        libraries_list.push_str(&libmanager(&vjson, operationalsystem, &mc_dir));

        if !Path::new(&versionpath).exists() || fs::metadata(&versionpath).unwrap().len() == 0 {
            installer::downloadversionjar(
                1,
                &vjson,
                &format!("{}/versions/{}", &mc_dir, &game_version),
                &game_version.to_owned(),
            )
            .await
            .unwrap();

            if let Some(libraries) = vjson["libraries"].as_array() {
                installer::downloadlibraries(
                    &mc_dir,
                    operationalsystem,
                    libraries,
                    &format!("{}/versions/{}", &mc_dir, &game_version),
                )
                .await
                .unwrap()
            }

            installer::downloadassets(&mc_dir, &vjson).await.unwrap();
        }
    }
    //for custom versions
    if !Path::new(&versionpath).exists() || fs::metadata(&versionpath).unwrap().len() == 0 {
        installer::downloadversionjar(
            1,
            &p,
            &format!("{}/versions/{}", &mc_dir, &game_version),
            &game_version.to_owned(),
        )
        .await
        .unwrap();
        println!("Version jar has been downloaded.");

        if let Some(libraries) = p["libraries"].as_array() {
            installer::downloadlibraries(
                &mc_dir,
                operationalsystem,
                libraries,
                &format!("{}/versions/{}", &mc_dir, &game_version),
            )
            .await
            .unwrap()
        }

        installer::downloadassets(&mc_dir, &p).await.unwrap();
    }

    let jvm = match autojava {
        true => {
            let mut p = p.clone();
            if ismodded {
                let vanillaversion = p["inheritsFrom"].as_str().unwrap_or(game_version);
                let vanillajsonpathstring = format!(
                    "{}/versions/{}/{}.json",
                    &mc_dir, game_version, vanillaversion
                );

                let mut vanillajson = File::open(vanillajsonpathstring).unwrap();

                let mut vjsoncontent = String::new();
                vanillajson.read_to_string(&mut vjsoncontent).unwrap();
                p = serde_json::from_str(&vjsoncontent).unwrap();
            }
            let requiredjavaversion = p["javaVersion"]["majorVersion"].as_i64().unwrap_or(1);
            if requiredjavaversion > 8 || requiredjavaversion == 0 {
                if Path::new(&autojavapaths[0]).exists() {
                    jvmargs = "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+UseNUMA -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=400M -XX:NonNMethodCodeHeapSize=12M -XX:ProfiledCodeHeapSize=194M -XX:NonProfiledCodeHeapSize=194M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseVectorCmov -XX:+PerfDisableSharedMem -XX:+UseFastUnorderedTimeStamps -XX:+UseCriticalJavaThreadPriority -XX:ThreadPriorityPolicy=1 -XX:AllocatePrefetchStyle=3 -XX:+UseShenandoahGC -XX:ShenandoahGCMode=iu -XX:ShenandoahGuaranteedGCInterval=1000000 -XX:AllocatePrefetchStyle=1"
                        .split(' ').map(|s| s.to_owned()).collect();

                    autojavapaths[0].as_str()
                } else {
                    downloadjava(true).await.unwrap();
                    #[cfg(target_os = "linux")]
                    if std::env::consts::OS == "linux" {
                        let mut permission = fs::metadata(&autojavapaths[0]).unwrap().permissions();
                        permission.set_mode(0o755);
                        fs::set_permissions(&autojavapaths[0], permission).unwrap();
                    };

                    jvmargs = "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+UseNUMA -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=400M -XX:NonNMethodCodeHeapSize=12M -XX:ProfiledCodeHeapSize=194M -XX:NonProfiledCodeHeapSize=194M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseVectorCmov -XX:+PerfDisableSharedMem -XX:+UseFastUnorderedTimeStamps -XX:+UseCriticalJavaThreadPriority -XX:ThreadPriorityPolicy=1 -XX:AllocatePrefetchStyle=3 -XX:+UseShenandoahGC -XX:ShenandoahGCMode=iu -XX:ShenandoahGuaranteedGCInterval=1000000 -XX:AllocatePrefetchStyle=1"
                        .split(' ').map(|s| s.to_owned()).collect();

                    autojavapaths[0].as_str()
                }
            } else if Path::new(&autojavapaths[1]).exists() {
                jvmargs = "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+ParallelRefProcEnabled -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:+PerfDisableSharedMem -XX:+AggressiveOpts -XX:+UseFastAccessorMethods -XX:MaxInlineLevel=15 -XX:MaxVectorSize=32 -XX:+UseCompressedOops -XX:ThreadPriorityPolicy=1 -XX:+UseNUMA -XX:+UseDynamicNumberOfGCThreads -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=350M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseFPUForSpilling -Dgraal.CompilerConfiguration=community -XX:+UseG1GC -XX:MaxGCPauseMillis=37 -XX:+PerfDisableSharedMem -XX:G1HeapRegionSize=16M -XX:G1NewSizePercent=23 -XX:G1ReservePercent=20 -XX:SurvivorRatio=32 -XX:G1MixedGCCountTarget=3 -XX:G1HeapWastePercent=20 -XX:InitiatingHeapOccupancyPercent=10 -XX:G1RSetUpdatingPauseTimePercent=0 -XX:MaxTenuringThreshold=1 -XX:G1SATBBufferEnqueueingThresholdPercent=30 -XX:G1ConcMarkStepDurationMillis=5.0 -XX:G1ConcRSHotCardLimit=16 -XX:G1ConcRefinementServiceIntervalMillis=150 -XX:GCTimeRatio=99"
                        .split(' ').map(|s| s.to_owned()).collect();

                autojavapaths[1].as_str()
            } else {
                downloadjava(false).await.unwrap();
                #[cfg(target_os = "linux")]
                if std::env::consts::OS == "linux" {
                    let mut permission = fs::metadata(&autojavapaths[1]).unwrap().permissions();
                    permission.set_mode(0o755);
                    fs::set_permissions(&autojavapaths[1], permission).unwrap();
                }

                jvmargs = "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+ParallelRefProcEnabled -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:+PerfDisableSharedMem -XX:+AggressiveOpts -XX:+UseFastAccessorMethods -XX:MaxInlineLevel=15 -XX:MaxVectorSize=32 -XX:+UseCompressedOops -XX:ThreadPriorityPolicy=1 -XX:+UseNUMA -XX:+UseDynamicNumberOfGCThreads -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=350M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseFPUForSpilling -Dgraal.CompilerConfiguration=community -XX:+UseG1GC -XX:MaxGCPauseMillis=37 -XX:+PerfDisableSharedMem -XX:G1HeapRegionSize=16M -XX:G1NewSizePercent=23 -XX:G1ReservePercent=20 -XX:SurvivorRatio=32 -XX:G1MixedGCCountTarget=3 -XX:G1HeapWastePercent=20 -XX:InitiatingHeapOccupancyPercent=10 -XX:G1RSetUpdatingPauseTimePercent=0 -XX:MaxTenuringThreshold=1 -XX:G1SATBBufferEnqueueingThresholdPercent=30 -XX:G1ConcMarkStepDurationMillis=5.0 -XX:G1ConcRSHotCardLimit=16 -XX:G1ConcRefinementServiceIntervalMillis=150 -XX:GCTimeRatio=99"
                        .split(' ').map(|s| s.to_owned()).collect();

                autojavapaths[1].as_str()
            }
        }
        false => jvm,
    };

    libraries_list.push_str(&format!(
        "{}/versions/{}/{}.jar",
        &mc_dir, game_version, game_version
    ));

    let mut version_jvm_args = vec![];
    if let Some(arguments) = p["arguments"]["jvm"].as_array() {
        for i in arguments {
            if i.is_string() {
                let value = i.as_str().unwrap();
                if value.contains("${natives_directory}") {
                    let value = value.replace("${natives_directory}", &nativedir);
                    version_jvm_args.push(value);
                } else {
                    version_jvm_args.push(value.to_owned())
                }
            }
        }
    } else {
        version_jvm_args.push(format!("-Djava.library.path={}", nativedir))
    }
    if let Some(arguments) = p["arguments"]["game"].as_array() {
        let gamedata = vec![
            player.to_owned(),
            game_version.to_string(),
            gamedir.to_string(),
            assets_dir,
            assetindex,
            uuid.to_owned(),
            String::from("[pro]"),
            String::from("{}"),
            String::from("legacy"),
            String::from("release"),
        ];
        let mut str_arguments = vec![];
        let mut str_moddedgameargs = vec![];
        let mut needs_user_properties = true;
        for i in arguments {
            if i.is_string() {
                if i.as_str().unwrap().contains("--userProperties") {
                    needs_user_properties = false;
                }
                str_arguments.push(i.as_str().unwrap_or("").to_owned())
            } else {
                if i["value"]
                    .as_str()
                    .unwrap_or("")
                    .contains("--userProperties")
                {
                    needs_user_properties = false;
                }
                if i["value"].is_string() {
                    str_arguments.push(i["value"].as_str().unwrap().to_owned())
                }
            }
        }
        for i in moddedgameargs {
            if i.is_string() {
                str_moddedgameargs.push(i.as_str().unwrap_or("").to_owned())
            }
        }
        if needs_user_properties {
            str_moddedgameargs.push("--userProperties".to_string());
            str_moddedgameargs.push("{}".to_string());
        }
        version_game_args = getgameargs(str_arguments, &gamedata);
        version_game_args.extend_from_slice(&getgameargs(str_moddedgameargs, &gamedata))
    } else if let Some(arguments) = p["minecraftArguments"].as_str() {
        let gamedata = vec![
            player.to_owned(),
            game_version.to_string(),
            gamedir.to_string(),
            assets_dir,
            assetindex,
            uuid.to_owned(),
            String::from("[pro]"),
            String::from("{}"),
            String::from("legacy"),
            String::from("Release"),
            String::from("Modified"),
        ];
        //let oldmoddedargs: Vec<String> = oldmoddedgameargs_str.split_whitespace().map(String::from).collect();
        let oldargs: Vec<String> = arguments
            .to_string()
            .split_whitespace()
            .map(String::from)
            .collect();

        //version_game_args = getgameargs(oldmoddedargs, &gamedata.clone());
        version_game_args.extend_from_slice(&getgameargs(oldargs, &gamedata))
    }

    let mut mineprogram = if gamemode {
        Command::new("gamemoderun")
    } else {
        Command::new(jvm)
    };

    if gamemode {
        mineprogram.arg(jvm);
    }
    mineprogram
        .arg(format!("-Xmx{}M", ram * 1024.))
        .args(jvmargs)
        .arg("-cp")
        .arg(libraries_list)
        .args(version_jvm_args);

    mineprogram.arg(mainclass).args(version_game_args);

    println!(
        "Launching with the following command: \n{:?}\n\n",
        mineprogram
    );
    mineprogram.spawn().expect("Failed to execute Minecraft!");

    Ok(())
}

fn libmanager(p: &Value, os: &str, mc_dir: &String) -> String {
    let mut libraries_list = String::new();

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

                    libraries_list.push_str(&libpath);
                    libraries_list.push(separator);
                } else if library["natives"][os].is_null() {
                    let libpath = format!(
                        "{}{}/{}/{}-{}.jar",
                        lib_dir,
                        &firstpiece,
                        &lpieces.join("/"),
                        &lpieces[&lpieces.len() - 2],
                        &lpieces[&lpieces.len() - 1]
                    );

                    libraries_list.push_str(&libpath);
                    libraries_list.push(separator);
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

                    libraries_list.push_str(&libpath);
                    libraries_list.push(separator);
                }
            }
        }
    }
    libraries_list
}

pub fn getinstalledversions() -> Vec<String> {
    let versions_dir = match std::env::consts::OS {
        "linux" => format!("{}/.minecraft/versions", std::env::var("HOME").unwrap()),
        "windows" => format!(
            "{}/AppData/Roaming/.minecraft/versions",
            std::env::var("USERPROFILE").unwrap().replace('\\', "/")
        ),
        _ => panic!("System not supported."),
    };
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

fn getgameargs(arguments: Vec<String>, gamedata: &[String]) -> Vec<String> {
    let mut version_game_args = vec![];
    for i in arguments {
        match i.as_str() {
            "${auth_player_name}" => version_game_args.push(gamedata[0].clone()),
            "${version_name}" => version_game_args.push(gamedata[1].clone()),
            "${game_directory}" => version_game_args.push(gamedata[2].clone()),
            "${assets_root}" => version_game_args.push(gamedata[3].clone()),
            "${assets_index_name}" => version_game_args.push(gamedata[4].clone()),
            "${auth_uuid}" => version_game_args.push(gamedata[5].clone()),
            "${clientid}" => version_game_args.push(gamedata[5].clone()),
            "${auth_xuid}" => version_game_args.push(gamedata[5].clone()),
            "${auth_access_token}" => version_game_args.push(gamedata[6].clone()),
            "${user_properties}" => version_game_args.push(gamedata[7].clone()),
            "${user_type}" => version_game_args.push(gamedata[8].clone()),
            "${version_type}" => version_game_args.push(gamedata[9].clone()),

            "--demo" => {}
            _ => version_game_args.push(i.to_owned()),
        }
    }
    version_game_args
}

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
