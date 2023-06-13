use serde_json::Value;
use std::{
    env::{self},
    fs::{self, File},
    io::{Read, Write},
    os::unix::prelude::PermissionsExt,
    path::Path,
    process::Command,
};

use crate::backend::installer::downloadjava;

pub(crate) mod installer;

pub async fn start(
    player: &str,
    game_version: &str,
    jvm: &str,
    jvmargs: Vec<String>,
    ram: f64,
    gamemode: bool,
    pdirectory: String,
    autojava: bool,
) -> Result<(), String> {
    let result = std::panic::catch_unwind(|| {
        let operationalsystem = std::env::consts::OS;
        let player = player;
        let mc_dir = match std::env::consts::OS {
            "linux" => format!("{}/.minecraft", std::env::var("HOME").unwrap()),
            "windows" => format!(
                "{}/AppData/Roaming/.minecraft",
                std::env::var("USERPROFILE").unwrap().replace('\\', "/")
            ),
            _ => panic!("System not supported."),
        };

        let autojavapaths = if std::env::consts::OS == "windows" {
            vec![
                format!("{}/java/java17/bin/java.exe", mc_dir),
                format!("{}/java/java18/bin/java.exe", mc_dir),
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
        let uuid = "9791bffa968538928aa0b3ff397fd54f";

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

        let jvm = match autojava {
            true => {
                if p["javaVersion"]["majorVersion"].as_i64().unwrap() > 8 {
                    if Path::new(&autojavapaths[0]).exists() {
                        jvmargs = "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+UseNUMA -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=400M -XX:NonNMethodCodeHeapSize=12M -XX:ProfiledCodeHeapSize=194M -XX:NonProfiledCodeHeapSize=194M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseVectorCmov -XX:+PerfDisableSharedMem -XX:+UseFastUnorderedTimeStamps -XX:+UseCriticalJavaThreadPriority -XX:ThreadPriorityPolicy=1 -XX:AllocatePrefetchStyle=3 -XX:+UseShenandoahGC -XX:ShenandoahGCMode=iu -XX:ShenandoahGuaranteedGCInterval=1000000 -XX:AllocatePrefetchStyle=1"
                        .split(' ').map(|s| s.to_owned()).collect();

                        autojavapaths[0].as_str()
                    } else {
                        downloadjava(true).unwrap();
                        if std::env::consts::OS == "linux" {
                            let mut permission =
                                fs::metadata(&autojavapaths[0]).unwrap().permissions();
                            permission.set_mode(0o755);
                            fs::set_permissions(&autojavapaths[0], permission).unwrap();
                        }

                        autojavapaths[0].as_str()
                    }
                } else if Path::new(&autojavapaths[1]).exists() {
                    jvmargs = "-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+ParallelRefProcEnabled -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:+PerfDisableSharedMem -XX:+AggressiveOpts -XX:+UseFastAccessorMethods -XX:MaxInlineLevel=15 -XX:MaxVectorSize=32 -XX:+UseCompressedOops -XX:ThreadPriorityPolicy=1 -XX:+UseNUMA -XX:+UseDynamicNumberOfGCThreads -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=350M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseFPUForSpilling -Dgraal.CompilerConfiguration=community -XX:+UseG1GC -XX:MaxGCPauseMillis=37 -XX:+PerfDisableSharedMem -XX:G1HeapRegionSize=16M -XX:G1NewSizePercent=23 -XX:G1ReservePercent=20 -XX:SurvivorRatio=32 -XX:G1MixedGCCountTarget=3 -XX:G1HeapWastePercent=20 -XX:InitiatingHeapOccupancyPercent=10 -XX:G1RSetUpdatingPauseTimePercent=0 -XX:MaxTenuringThreshold=1 -XX:G1SATBBufferEnqueueingThresholdPercent=30 -XX:G1ConcMarkStepDurationMillis=5.0 -XX:G1ConcRSHotCardLimit=16 -XX:G1ConcRefinementServiceIntervalMillis=150 -XX:GCTimeRatio=99"
                        .split(' ').map(|s| s.to_owned()).collect();

                    autojavapaths[1].as_str()
                } else {
                    downloadjava(false).unwrap();
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

        let mainclass = &p["mainClass"].as_str().unwrap();
        let mut assetindex = p["assets"].to_string();
        let nativedir = format!("{}/versions/{}/natives", &mc_dir, game_version);

        let mut libraries_list = libmanager(&p, operationalsystem, &mc_dir);

        if game_version.to_lowercase().contains("fabric-loader")
            || game_version.to_lowercase().contains("forge")
        {
            let vanillaversion = p["inheritsFrom"].as_str().unwrap();
            let vanillaversionpathstring = format!(
                "{}/versions/{}/{}.jar",
                &mc_dir, vanillaversion, vanillaversion
            );
            let vanillajsonpathstring = format!(
                "{}/versions/{}/{}.json",
                &mc_dir, vanillaversion, vanillaversion
            );
            let vanillajsonfilepath = Path::new(&vanillajsonpathstring);
            if !vanillajsonfilepath.exists() {
                println!("{} needs to be installed.", vanillaversion);
                installer::installversion(vanillaversion.to_string()).unwrap();
            }

            let mut vanillaversionfile = File::open(vanillaversionpathstring).unwrap();
            let mut buffer = Vec::new();
            vanillaversionfile.read_to_end(&mut buffer).unwrap();
            let mut modver_towrite = File::create(&versionpath).unwrap();
            modver_towrite.write_all(&buffer).unwrap();

            fs::create_dir_all(format!("{}/versions/{}/natives", &mc_dir, game_version)).unwrap();

            if let Ok(vanillanatives) =
                fs::read_dir(format!("{}/versions/{}/natives", &mc_dir, vanillaversion))
            {
                for i in vanillanatives {
                    if !i.as_ref().unwrap().file_type().unwrap().is_dir() {
                        fs::copy(
                            i.as_ref().unwrap().path(),
                            format!(
                                "{}/versions/{}/natives/{}",
                                &mc_dir,
                                game_version,
                                i.as_ref().unwrap().file_name().to_string_lossy()
                            ),
                        )
                        .unwrap();
                    }
                }
            }

            let mut vanillajson = File::open(&vanillajsonpathstring).unwrap();

            let mut vjsoncontent = String::new();
            vanillajson.read_to_string(&mut vjsoncontent).unwrap();
            let vjson: Value = serde_json::from_str(&vjsoncontent).unwrap();
            libraries_list.push_str(&libmanager(&vjson, operationalsystem, &mc_dir));
            assetindex = vjson["assets"].to_string();
        }

        let isforge = game_version.to_lowercase().contains("forge");
        let mut forgejvmargs: Vec<String> = vec![];
        let mut forgegameargs: Vec<String> = vec![];
        if isforge {
            if !p["arguments"].is_null() {
                if let Some(forgejvmarguments) = p["arguments"]["jvm"].as_array() {
                    for i in forgejvmarguments {
                        forgejvmargs.push(i.as_str().unwrap().to_string());
                    }
                };

                if let Some(forgegamearguments) = p["arguments"]["game"].as_array() {
                    for i in forgegamearguments {
                        forgegameargs.push(i.as_str().unwrap().to_string());
                    }
                }
            } else if !p["minecraftArguments"].is_null() {
                forgegameargs.push(String::from("--tweakClass"));
                forgegameargs.push(String::from(
                    "net.minecraftforge.fml.common.launcher.FMLTweaker",
                ))
            }
        }

        assetindex = assetindex.replace('\"', "");
        libraries_list.push_str(&format!(
            "{}/versions/{}/{}.jar",
            &mc_dir, game_version, game_version
        ));
        if pdirectory.is_empty() {
            env::set_current_dir(&mc_dir).expect("Failed to open profile folder!");
        } else {
            fs::create_dir_all(&pdirectory).unwrap();
            env::set_current_dir(&pdirectory).expect("Failed to open profile folder!");
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
            .arg(format!("-Djava.library.path={}", nativedir))
            .arg("-cp")
            .arg(libraries_list);
        if isforge {
            mineprogram.args(forgejvmargs);
        }
        mineprogram.arg(mainclass).args([
            "--username",
            player,
            "--version",
            game_version,
            "--accessToken",
            "[pro]",
            "--userProperties",
            "{}",
            "--gameDir",
            &pdirectory,
            "--assetsDir",
            &assets_dir,
            "--assetIndex",
            &assetindex,
            "--uuid",
            uuid,
            "--userType",
            "legacy",
        ]);
        if isforge {
            mineprogram.args(forgegameargs);
        }
        println!("{:?}", mineprogram);
        mineprogram.spawn().expect("Failed to execute Minecraft!");
    });

    match result {
        Ok(_) => Ok(()),
        Err(_) => {
            Err("A panic occurred. Maybe there is something wrong with your options.".to_string())
        }
    }
}

#[tokio::main]
async fn libmanager(p: &Value, os: &str, mc_dir: &String) -> String {
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
