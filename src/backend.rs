use serde_json::Value;
use std::{
    env::{self},
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::Command,
};

pub(crate) mod version_installer;

pub async fn start(
    player: &str,
    game_version: &str,
    jvm: &str,
    jvmargs: Vec<String>,
    ram: f64,
    gamemode: bool,
    pdirectory: String,
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
        let mainclass = &p["mainClass"].as_str().unwrap();
        let mut assetindex = p["assets"].to_string();
        let nativedir = format!("{}/versions/{}/natives", &mc_dir, game_version);

        let mut libraries_list = libmanager(&p, operationalsystem, &mc_dir);

        if game_version.contains("fabric-loader") {
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
                version_installer::installversion(vanillaversion.to_string()).unwrap();

                fs::remove_file(&versionpath).unwrap();
                let mut vanillaversionfile = File::open(vanillaversionpathstring).unwrap();
                let mut buffer = Vec::new();
                vanillaversionfile.read_to_end(&mut buffer).unwrap();
                let mut fabric_towrite = File::create(&versionpath).unwrap();
                fabric_towrite.write_all(&buffer).unwrap();
            }

            let mut vanillajson = File::open(&vanillajsonpathstring).unwrap();

            let mut vjsoncontent = String::new();
            vanillajson.read_to_string(&mut vjsoncontent).unwrap();
            let vjson: Value = serde_json::from_str(&vjsoncontent).unwrap();
            libraries_list.push_str(&libmanager(&vjson, operationalsystem, &mc_dir));
            assetindex = vjson["assets"].to_string();
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
            .arg(libraries_list)
            .arg(mainclass)
            .args([
                "--username",
                player,
                "--version",
                game_version,
                "--accessToken",
                "[pro]",
                "--userProperties",
                "{}",
                "--gamedir",
                &mc_dir,
                "--assetsDir",
                &assets_dir,
                "--assetIndex",
                &assetindex,
                "--uuid",
                uuid,
                "--userType",
                "legacy",
            ]);
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
