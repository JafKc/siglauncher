use reqwest::Client;
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
    path::Path,
};
use zip::read::ZipArchive;

#[tokio::main]
pub async fn installversion(version: String, version_type: u8) -> Result<(), reqwest::Error> {
    //version_type -> 1 for vanilla and 2 for fabric.

    let os = std::env::consts::OS;
    let mc_dir = match std::env::consts::OS {
        "linux" => format!("{}/.minecraft", std::env::var("HOME").unwrap()),
        "windows" => format!(
            "{}/AppData/Roaming/.minecraft",
            std::env::var("USERPROFILE").unwrap().replace('\\', "/")
        ),
        _ => panic!("System not supported."),
    };

    let versionname = match version_type {
        1 => version.to_owned(),
        2 => format!("{}-Fabric", &version),
        _ => panic!("Version type doesn't exists!"),
    };

    let foldertosave = format!("{}/versions/{}", &mc_dir, versionname);

    println!("Downloading json...");
    downloadversionjson(version_type, &version, &foldertosave).await?;
    //
    let jpathstring = format!("{}/versions/{}/{}.json", &mc_dir, versionname, versionname);
    let jsonpath = Path::new(&jpathstring);

    let mut file = File::open(jsonpath).unwrap();
    let mut fcontent = String::new();
    file.read_to_string(&mut fcontent).unwrap();
    let content = serde_json::from_str(&fcontent);

    let p: Value = content.unwrap();

    println!("Downloading jar...");
    downloadversionjar(version_type, &p, &foldertosave, &version).await?;

    //assetindex and assets
    println!("Downloading asset index...");
    if version_type == 1 {
        downloadassets(&mc_dir, &p).await?;
    } else {
        downloadassets(
            &mc_dir,
            &getjson(format!(
                "{}/versions/{}/{}.json",
                &mc_dir, versionname, version
            )),
        )
        .await?
    }

    if let Some(libraries) = p["libraries"].as_array() {
        downloadlibraries(&mc_dir, os, libraries, &foldertosave).await?;
    }

    if version_type != 1 {
        if let Some(libraries) = getjson(format!(
            "{}/versions/{}/{}.json",
            &mc_dir, versionname, version
        ))["libraries"]
            .as_array()
        {
            downloadlibraries(&mc_dir, os, libraries, &foldertosave).await?;
        }
    }
    Ok(())
}

pub async fn downloadlibraries(
    mc_dir: &String,
    os: &str,
    libraries: &Vec<Value>,
    foldertosave: &String,
) -> Result<(), reqwest::Error> {
    //libraries and natives
    let lib_dir = format!("{}/libraries/", mc_dir);

    for library in libraries {
        if library["rules"][0]["os"]["name"] == os || library["rules"][0]["os"]["name"].is_null() {
            let libraryname = library["name"].as_str().unwrap();
            let mut lpieces: Vec<&str> = libraryname.split(':').collect();
            let firstpiece = lpieces[0].replace('.', "/");
            lpieces.remove(0);

            if library["name"].as_str().unwrap().contains("natives") {
                lpieces.remove(lpieces.len() - 1);
                let lib = format!(
                    "{}/{}/{}-{}-natives-{}.jar",
                    &firstpiece,
                    &lpieces.join("/"),
                    &lpieces[&lpieces.len() - 2],
                    &lpieces[&lpieces.len() - 1],
                    os
                );
                let libpath = format!("{}{}", lib_dir, lib);

                if !Path::exists(Path::new(&libpath)) {
                    let unmodifiedurl = library["downloads"]["artifacts"]["url"]
                        .as_str()
                        .unwrap_or(library["url"].as_str().unwrap());
                    let mut url = unmodifiedurl.to_owned();
                    if unmodifiedurl == "https://maven.fabricmc.net/" {
                        url = format!("{}{}", url, lib)
                    }
                    println!("Downloading library to {}", &libpath);
                    let libtodownload = reqwest::Client::new()
                        .get(url)
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
                let lib = format!(
                    "{}/{}/{}-{}.jar",
                    &firstpiece,
                    &lpieces.join("/"),
                    &lpieces[&lpieces.len() - 2],
                    &lpieces[&lpieces.len() - 1]
                );
                let libpath = format!("{}{}", lib_dir, lib);

                if !Path::exists(Path::new(&libpath)) {
                    let unmodifiedurl = library["downloads"]["artifacts"]["url"]
                        .as_str()
                        .unwrap_or(library["url"].as_str().unwrap());
                    let mut url = unmodifiedurl.to_owned();
                    if unmodifiedurl == "https://maven.fabricmc.net/" {
                        url = format!("{}{}", url, lib)
                    }

                    println!("Downloading library to {}", &libpath);
                    let libtodownload = reqwest::Client::new()
                        .get(url)
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
                let lib = format!(
                    "{}/{}/{}-{}-natives-{}.jar",
                    &firstpiece,
                    &lpieces.join("/"),
                    &lpieces[&lpieces.len() - 2],
                    &lpieces[&lpieces.len() - 1],
                    os
                );

                let libpath = format!("{}{}", lib_dir, lib);

                if !Path::exists(Path::new(&libpath)) {
                    let unmodifiedurl = if library["downloads"]["artifacts"]["url"].is_string() {
                        library["downloads"]["artifacts"]["url"].as_str().unwrap()
                    } else if library["url"].is_string() {
                        library["url"].as_str().unwrap()
                    } else if library["downloads"]["classifiers"][format!("natives-{}", os)]["url"]
                        .is_string()
                        || library["downloads"]["classifiers"][format!("natives-{}-64", os)]["url"]
                            .is_string()
                    {
                        library["downloads"]["classifiers"][format!("natives-{}", os)]["url"]
                            .as_str()
                            .unwrap_or(
                                library["downloads"]["classifiers"][format!("natives-{}-64", os)]
                                    ["url"]
                                    .as_str()
                                    .unwrap(),
                            )
                    } else {
                        ""
                    };
                    let mut url = unmodifiedurl.to_owned();
                    if unmodifiedurl == "https://maven.fabricmc.net/" {
                        url = format!("{}{}", url, lib)
                    }

                    if !url.is_empty() {
                        println!("Downloading library to {}", &libpath);
                        let libtodownload = reqwest::Client::new()
                            .get(url)
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
                }
            }
        }

        if !library["downloads"]["classifiers"][format!("natives-{}", os)].is_null() {
            println!("Downloading native {}", library["name"]);
            let versionnatives = reqwest::Client::new()
                .get(
                    library["downloads"]["classifiers"][format!("natives-{}", os)]["url"]
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
    Ok(())
}

#[tokio::main]
pub async fn getversionlist(
    showallversions: bool,
) -> Result<Vec<Vec<String>>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    // vanilla
    let vanillaversionlistjson = client
        .get("https://launchermeta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await?
        .text()
        .await?;

    let content = serde_json::from_str(&vanillaversionlistjson);

    let p: Value = content?;

    let mut vanillaversionlist: Vec<String> = vec![];
    if let Some(versions) = p["versions"].as_array() {
        if showallversions {
            for i in versions {
                vanillaversionlist.push(i["id"].to_string())
            }
        } else {
            for i in versions {
                if i["type"] == "release" {
                    vanillaversionlist.push(i["id"].to_string())
                }
            }
        }
    }
    // fabric
    let fabricversionlistjson = client
        .get("https://meta.fabricmc.net/v2/versions/game")
        .send()
        .await?
        .text()
        .await?;

    let content = serde_json::from_str(&fabricversionlistjson);

    let p: Value = content?;

    let mut fabricversionlist: Vec<String> = vec![];
    if let Some(versions) = p.as_array() {
        if showallversions {
            for i in versions {
                fabricversionlist.push(i["version"].to_string())
            }
        } else {
            for i in versions {
                if i["stable"] == true {
                    fabricversionlist.push(i["version"].to_string())
                }
            }
        }
    }
    println!("{:?}", fabricversionlist);

    Ok(vec![vanillaversionlist, fabricversionlist])
}
pub async fn downloadjava(new: bool) -> Result<(), reqwest::Error> {
    let client = Client::new();

    let foldertostore;
    let url;

    match std::env::consts::OS {
        "linux" => {
            foldertostore = format!("{}/.minecraft/java", std::env::var("HOME").unwrap());
            url = if new {
                "https://raw.githubusercontent.com/JafKc/siglauncher-jvm/main/binaries/java17-linux.zip"
            } else {
                "https://raw.githubusercontent.com/JafKc/siglauncher-jvm/main/binaries/java8-linux.zip"
            };
        }
        "windows" => {
            foldertostore = format!(
                "{}/AppData/Roaming/.minecraft/java",
                std::env::var("USERPROFILE").unwrap()
            );
            url = if new {
                "https://raw.githubusercontent.com/JafKc/siglauncher-jvm/main/binaries/java17-windows.zip"
            } else {
                "https://raw.githubusercontent.com/JafKc/siglauncher-jvm/main/binaries/java8-windows.zip"
            };
        }
        _ => panic!("system not supported!"),
    }

    let download = client
        .get(url)
        .header("User-Agent", "Siglauncher")
        .send()
        .await?
        .bytes()
        .await?;

    fs::create_dir_all(&foldertostore).unwrap();
    let mut compressed = File::create(format!("{}/compressedjava.zip", &foldertostore)).unwrap();
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

    Ok(())
}
pub async fn downloadversionjson(
    version_type: u8,
    version: &String,
    foldertosave: &String,
) -> Result<(), reqwest::Error> {
    match version_type {
        1 => {
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
                    if i["id"].as_str().unwrap() == version {
                        println!("Downloading json...");
                        let versionjson = reqwest::Client::new()
                            .get(i["url"].as_str().unwrap())
                            .send()
                            .await?
                            .bytes()
                            .await?;

                        let jfilelocation = format!("{}/{}.json", foldertosave, version);
                        fs::create_dir_all(&foldertosave).unwrap();
                        let mut jfile = File::create(jfilelocation).unwrap();

                        jfile.write_all(&versionjson).unwrap();
                        println!("Json downloaded successfully.");
                    }
                }
            }
        }
        2 => {
            let downloader = reqwest::Client::new();

            let fabricloaderlist = downloader
                .get("https://meta.fabricmc.net/v2/versions/loader")
                .send()
                .await?
                .text()
                .await?;

            let content: Value = serde_json::from_str(&fabricloaderlist).unwrap();

            let fabricloaderversion =
                if let Some(first_object) = content.as_array().and_then(|arr| arr.first()) {
                    first_object["version"].as_str().unwrap()
                } else {
                    panic!("Failed to get fabric loader name")
                };

            let verjson = downloader
                .get(format!(
                    "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
                    version, fabricloaderversion
                ))
                .send()
                .await?
                .bytes()
                .await?;

            let jfilelocation = format!("{}/{}-Fabric.json", foldertosave, version);
            fs::create_dir_all(&foldertosave).unwrap();
            let mut jfile = File::create(jfilelocation).unwrap();

            jfile.write_all(&verjson).unwrap();
            println!("Json downloaded successfully.");
        }
        _ => panic!("Version type doesn't exists!"),
    }

    Ok(())
}
pub async fn downloadversionjar(
    version_type: u8,
    p: &Value,
    foldertosave: &String,
    version: &String,
) -> Result<(), reqwest::Error> {
    match version_type {
        1 => {
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
            println!("Jar file downloaded successfully.");
        }
        2 => {
            downloadversionjson(1, version, foldertosave).await?;

            let jpathstring = format!("{}/{}.json", &foldertosave, version);
            let vanillap = getjson(jpathstring);

            let versionjar = reqwest::Client::new()
                .get(vanillap["downloads"]["client"]["url"].as_str().unwrap())
                .send()
                .await?
                .bytes()
                .await?;

            let verfilelocation = format!("{}/{}-Fabric.jar", foldertosave, version);
            let mut verfile = File::create(verfilelocation).unwrap();
            verfile.write_all(&versionjar).unwrap();
            println!("Jar file downloaded successfully.");
        }
        _ => panic!("Version type doesn't exists!"),
    }
    Ok(())
}

async fn downloadassets(mc_dir: &String, p: &Value) -> Result<(), reqwest::Error> {
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
                if !Path::new(&format!("{}/{}/{}", &assetdir, &hash[0..2], &hash)).exists() {
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
                    fs::create_dir_all(format!("{}/{}", &assetdir, &hash[0..2])).unwrap();
                    let mut assetfile =
                        File::create(format!("{}/{}/{}", &assetdir, &hash[0..2], &hash)).unwrap();
                    assetfile.write_all(&a).unwrap();
                    println!("Downloaded asset {}", key);
                } else {
                    println!("Asset {} already exists. Skipping", key)
                }
            }
        }
        println!("Assets downloaded successfully.");
    }
    Ok(())
}

fn getjson(jpathstring: String) -> Value {
    let jsonpath = Path::new(&jpathstring);

    let mut file = File::open(jsonpath).unwrap();
    let mut fcontent = String::new();
    file.read_to_string(&mut fcontent).unwrap();
    serde_json::from_str(&fcontent).unwrap()
}
