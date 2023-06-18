use reqwest::Client;
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::Path,
};
use zip::read::ZipArchive;

#[tokio::main]
pub async fn installversion(version: String) -> Result<(), reqwest::Error> {
    let os = std::env::consts::OS;
    let mc_dir = match std::env::consts::OS {
        "linux" => format!("{}/.minecraft", std::env::var("HOME").unwrap()),
        "windows" => format!(
            "{}/AppData/Roaming/.minecraft",
            std::env::var("USERPROFILE").unwrap().replace('\\', "/")
        ),
        _ => panic!("System not supported."),
    };

    println!("Version is not installed. Preparing installation...");
    let versionlistjson = reqwest::Client::new()
        .get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await?
        .text()
        .await?;

    let content = serde_json::from_str(&versionlistjson);

    let p: Value = content.unwrap();
    if let Some(versions) = p["versions"].as_array() {
        for i in versions {
            if i["id"] == version {
                println!("Downloading version...");
                let versionjson = reqwest::Client::new()
                    .get(i["url"].as_str().unwrap())
                    .send()
                    .await?
                    .bytes()
                    .await?;

                let foldertosave = format!("{}/versions/{}", &mc_dir, version);
                let jfilelocation = format!("{}/{}.json", foldertosave, version);
                fs::create_dir_all(&foldertosave).unwrap();
                let mut jfile = File::create(jfilelocation).unwrap();

                jfile.write_all(&versionjson).unwrap();
                println!("Version downloaded successfully.");
                //
                println!("Downloading Json...");
                let jpathstring = format!("{}/versions/{}/{}.json", &mc_dir, version, version);
                let jsonpath = Path::new(&jpathstring);

                let mut file = File::open(jsonpath).unwrap();
                let mut fcontent = String::new();
                file.read_to_string(&mut fcontent).unwrap();
                let content = serde_json::from_str(&fcontent);

                let p: Value = content.unwrap();
                println!("{}", p["downloads"]["client"]["url"]);
                let versionjar = reqwest::Client::new()
                    .get(p["downloads"]["client"]["url"].as_str().unwrap())
                    .send()
                    .await?
                    .bytes()
                    .await?;

                let verfilelocation = format!("{}/{}.jar", foldertosave, version);
                let mut verfile = File::create(verfilelocation).unwrap();
                verfile.write_all(&versionjar).unwrap();
                println!("Json file downloaded successfully.");

                //assetindex and assets
                println!("Downloading asset index...");
                let versionindex = reqwest::Client::new()
                    .get(p["assetIndex"]["url"].as_str().unwrap())
                    .send()
                    .await?
                    .bytes()
                    .await?;
                let indexpath = format!(
                    "{}/assets/indexes/{}.json",
                    &mc_dir,
                    p["assets"].as_str().unwrap()
                );

                fs::create_dir_all(format!("{}/assets/indexes", &mc_dir)).unwrap();

                let mut indexfile = File::create(&indexpath).unwrap();
                indexfile.write_all(&versionindex).unwrap();
                drop(indexfile);
                println!("Asset index file downloaded successfully.");
                println!("Downloading assets...");
                let mut indexfile = File::open(&indexpath).unwrap();
                let mut idxfilecontent = String::new();
                indexfile.read_to_string(&mut idxfilecontent).unwrap();
                let idx: Value = serde_json::from_str(&idxfilecontent).unwrap();
                let assetdownloader = reqwest::Client::new();

                if let Some(assets) = idx["objects"].as_object() {
                    let assetdir = format!("{}/assets/objects/", &mc_dir);
                    for (key, value) in assets.iter() {
                        if let Some(hash) = value["hash"].as_str() {
                            if !Path::new(&format!("{}/{}/{}", &assetdir, &hash[0..2], &hash))
                                .exists()
                            {
                                let a = assetdownloader
                                    .get(format!(
                                        "https://resources.download.minecraft.net/{}/{}",
                                        &hash[0..2],
                                        hash
                                    ))
                                    .send()
                                    .await?
                                    .bytes()
                                    .await?;
                                fs::create_dir_all(format!("{}/{}", &assetdir, &hash[0..2]))
                                    .unwrap();
                                let mut assetfile = File::create(format!(
                                    "{}/{}/{}",
                                    &assetdir,
                                    &hash[0..2],
                                    &hash
                                ))
                                .unwrap();
                                assetfile.write_all(&a).unwrap();
                                println!("Downloaded asset {}", key);
                            } else {
                                println!("Asset {} already exists. Skipping", key)
                            }
                        }
                    }
                    println!("Assets downloaded successfully.");
                }

                if let Some(libraries) = p["libraries"].as_array() {
                    //libraries and natives
                    let lib_dir = format!("{}/libraries/", mc_dir);

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

                                if !Path::exists(Path::new(&libpath)) {
                                    println!("Downloading library to {}", &libpath);
                                    let libtodownload = reqwest::Client::new()
                                        .get(
                                            library["downloads"]["artifact"]["url"]
                                                .as_str()
                                                .unwrap(),
                                        )
                                        .send()
                                        .await
                                        .unwrap()
                                        .bytes()
                                        .await
                                        .unwrap();

                                    let directorytocreate = Path::new(&libpath).parent().unwrap();
                                    fs::create_dir_all(directorytocreate).unwrap();
                                    let mut newlib = File::create(&libpath).unwrap();
                                    newlib.write_all(&libtodownload).unwrap();
                                    println!("Downloaded successfully.");
                                } else {
                                    println!("Library {} exists. Skipping.", &libpath)
                                }
                            } else if library["natives"][os].is_null() {
                                let libpath = format!(
                                    "{}{}/{}/{}-{}.jar",
                                    lib_dir,
                                    &firstpiece,
                                    &lpieces.join("/"),
                                    &lpieces[&lpieces.len() - 2],
                                    &lpieces[&lpieces.len() - 1]
                                );

                                if !Path::exists(Path::new(&libpath)) {
                                    println!("Downloading library to {}", &libpath);
                                    let libtodownload = reqwest::Client::new()
                                        .get(
                                            library["downloads"]["artifact"]["url"]
                                                .as_str()
                                                .unwrap(),
                                        )
                                        .send()
                                        .await
                                        .unwrap()
                                        .bytes()
                                        .await
                                        .unwrap();

                                    let directorytocreate = Path::new(&libpath).parent().unwrap();
                                    fs::create_dir_all(directorytocreate).unwrap();
                                    let mut newlib = File::create(&libpath).unwrap();
                                    newlib.write_all(&libtodownload).unwrap();
                                    println!("Downloaded successfully.");
                                } else {
                                    println!("Library {} exists. Skipping.", &libpath)
                                }
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

                                if !Path::exists(Path::new(&libpath)) {
                                    if !library["downloads"]["artifact"]["url"].is_null() {
                                        println!("Downloading library to {}", &libpath);
                                        let libtodownload = reqwest::Client::new()
                                            .get(
                                                library["downloads"]["artifact"]["url"]
                                                    .as_str()
                                                    .unwrap(),
                                            )
                                            .send()
                                            .await
                                            .unwrap()
                                            .bytes()
                                            .await
                                            .unwrap();

                                        let directorytocreate =
                                            Path::new(&libpath).parent().unwrap();
                                        fs::create_dir_all(directorytocreate).unwrap();
                                        let mut newlib = File::create(&libpath).unwrap();
                                        newlib.write_all(&libtodownload).unwrap();
                                        println!("Downloaded successfully.");
                                    } else {
                                        println!("Library {} exists. Skipping.", &libpath)
                                    }
                                }
                            }
                        }

                        if !library["downloads"]["classifiers"][format!("natives-{}", os)].is_null()
                        {
                            println!("Downloading native {}", library["name"]);
                            let versionnatives = reqwest::Client::new()
                                .get(
                                    library["downloads"]["classifiers"][format!("natives-{}", os)]
                                        ["url"]
                                        .as_str()
                                        .unwrap(),
                                )
                                .send()
                                .await?
                                .bytes()
                                .await?;
                            let pathtocreate = format!("{}/natives", foldertosave);
                            fs::create_dir_all(&pathtocreate).unwrap();
                            let nativesfilepath = format!("{}/natives.jar", &pathtocreate);
                            let mut nativesfile = File::create(&nativesfilepath).unwrap();
                            nativesfile.write_all(&versionnatives).unwrap();
                            println!("Extracting {}", library["name"]);

                            drop(nativesfile);
                            let nativesfile = File::open(&nativesfilepath).unwrap();
                            let reader = BufReader::new(nativesfile);
                            let mut archive = ZipArchive::new(reader).unwrap();

                            for i in 0..archive.len() {
                                let mut file = archive.by_index(i).unwrap();
                                let outpath = format!(
                                    "{}/{}",
                                    &pathtocreate,
                                    file.mangled_name().to_string_lossy()
                                );
                                if file.is_dir() {
                                    println!("Creating directory: {:?}", outpath);
                                    std::fs::create_dir_all(&outpath).unwrap();
                                } else {
                                    println!("Extracting file: {:?}", outpath);
                                    let mut outfile = File::create(&outpath).unwrap();
                                    std::io::copy(&mut file, &mut outfile).unwrap();
                                }
                            }
                            fs::remove_file(&nativesfilepath).unwrap();
                            println!("Natives extracted successfully.");
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[tokio::main]
pub async fn getversionlist(
    showallversions: bool,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let versionlistjson = reqwest::Client::new()
        .get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await?
        .text()
        .await?;

    let content = serde_json::from_str(&versionlistjson);

    let p: Value = content?;

    let mut versionlist: Vec<String> = vec![];
    if let Some(versions) = p["versions"].as_array() {
        if showallversions {
            for i in versions {
                versionlist.push(i["id"].to_string())
            }
        } else {
            for i in versions {
                if i["type"] == "release" {
                    versionlist.push(i["id"].to_string())
                }
            }
        }
    }
    Ok(versionlist)
}
#[tokio::main]
pub async fn downloadjava(new: bool) -> Result<(), reqwest::Error> {
    let client = Client::new();

    match std::env::consts::OS {
        "linux" => {
            let foldertostore = &format!("{}/.minecraft/java", std::env::var("HOME").unwrap());
            let url = if new {
                "https://raw.githubusercontent.com/JafKc/siglauncher-jvm/main/binaries/java17-linux.zip"
            } else {
                "https://raw.githubusercontent.com/JafKc/siglauncher-jvm/main/binaries/java8-linux.zip"
            };

            let download = client
                .get(url)
                .header("User-Agent", "Siglauncher")
                .send()
                .await?
                .bytes()
                .await?;

            fs::create_dir_all(foldertostore).unwrap();
            let mut compressed =
                File::create(format!("{}/compressedjava.zip", foldertostore)).unwrap();
            compressed.write_all(&download).unwrap();
            let compressed = File::open(format!("{}/compressedjava.zip", foldertostore)).unwrap();

            let mut archive = ZipArchive::new(BufReader::new(compressed)).unwrap();

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).unwrap();
                let outpath = format!(
                    "{}/{}",
                    &foldertostore,
                    file.mangled_name().to_string_lossy()
                );
                if file.is_dir() {
                    println!("Creating directory: {:?}", outpath);
                    std::fs::create_dir_all(&outpath).unwrap();
                } else {
                    println!("Extracting file: {:?}", outpath);
                    let mut outfile = File::create(&outpath).unwrap();
                    std::io::copy(&mut file, &mut outfile).unwrap();
                }
            }
            fs::remove_file(format!("{}/compressedjava.zip", foldertostore)).unwrap();
            println!("Java was installed successfully.");
        }

        "windows" => todo!(),
        _ => panic!("System not supported."),
    }
    Ok(())
}
