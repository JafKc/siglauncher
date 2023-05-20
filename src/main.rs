use iced::widget::{
    button, column, container, pick_list, slider, svg, text, text_input, toggler, Row};
use iced::{alignment, executor, window, Alignment, Application, Command, Length, Settings};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::env::{self, set_current_dir};
use std::fmt::Debug;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

mod backend;
mod theme;
use self::widget::Element;

#[derive(Serialize, Deserialize)]
struct JVMs {
    name: String,
    path: String,
    flags: String,
}
#[derive(Serialize)]
struct GameWorkingDirectory {
    name: String,
    path: String,
}

#[derive(Default, Serialize, Deserialize)]
struct Siglauncher {
    username: String,
    version: Option<String>,
    ram: f64,
    currentjavaname: String,
    gamemodelinux: bool,
    currentworkingdirectory: String,
    #[serde(skip_serializing)]
    jvms: Vec<String>,
    #[serde(skip_serializing)]
    jvm: Vec<String>,
    #[serde(skip_serializing)]
    screen: i8,
    #[serde(skip_serializing)]
    versions: Vec<String>,
    #[serde(skip_serializing)]
    downloadlist: Vec<String>,
    #[serde(skip_serializing)]
    versiontodownload: String,
    #[serde(skip_serializing)]
    pdirectories: Vec<String>,
    #[serde(skip_serializing)]
    directoryprofile: Vec<String>,

    //add jvm
    jvmaddname: String,
    jvmaddpath: String,
    jvmaddflags: String,
    //add directory profile
    daddname: String,
    daddpath: String,
}

#[tokio::main]
pub async fn main() -> iced::Result {
    checksettingsfile();
    let icon = include_bytes!("icons/siglaunchericon.png");

    Siglauncher::run(Settings {
        window: window::Settings {
            size: (800, 500),

            icon: Some(window::icon::from_file_data(icon, None).unwrap()),

            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

#[derive(Debug, PartialEq, Clone)]
enum Message {
    UserChanged(String),
    VerChanged(String),
    LaunchPressed,
    InstallPressed,
    OptionsPressed,
    GithubPressed,

    GoJavaMan,
    GoDprofileMan,
    Launched(()),
    Gotlist(Vec<String>),
    DownloadChanged(String),

    RamChanged(f64),
    Apply,
    Return,
    JVMChanged(String),
    DProfileChanged(String),
    GamemodeChanged(bool),

    InstallVersion,
    Downloaded(()),

    JVMname(String),
    JVMpath(String),
    JVMflags(String),
    AddJVM,

    Directoryname(String),
    Directorypath(String),
    AddDirectory,
}

impl Application for Siglauncher {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();
    type Theme = theme::Theme;

    fn new(_flags: ()) -> (Siglauncher, iced::Command<Message>) {
        let mut file = File::open("launchsettings.json").unwrap();
        let mut fcontent = String::new();
        file.read_to_string(&mut fcontent).unwrap();
        let content = serde_json::from_str(&fcontent);
        let p: Value = content.unwrap();

        let mut currentjvm = Vec::new();

        let mut jvmnames: Vec<String> = Vec::new();

        if let Some(jvms) = p["JVMs"].as_array() {
            for jvm in jvms {
                jvmnames.push(jvm["name"].to_string());
                if jvm["name"] == p["currentjavaname"] {
                    currentjvm.push(jvm["name"].to_string());
                    currentjvm.push(jvm["path"].to_string());
                    currentjvm.push(jvm["flags"].to_string());
                }
            }
        }
        let currentjavaname = &currentjvm[0];

        let mut currentdirectoryprofile = Vec::new();
        let mut directorynames: Vec<String> = Vec::new();

        if let Some(directories) = p["Game working directories"].as_array() {
            for directory in directories {
                directorynames.push(directory["name"].to_string());
                if directory["name"] == p["currentworkingdirectory"] {
                    currentdirectoryprofile.push(directory["name"].to_string());
                    currentdirectoryprofile.push(directory["path"].to_string());
                }
            }
        }

        (
            Siglauncher {
                username: p["username"].to_string().replace("\"", ""),
                version: Some(p["version"].to_string().replace("\"", "")),
                screen: 1,
                versions: backend::getinstalledversions(),
                ram: p["ram"].as_f64().unwrap(),
                jvm: currentjvm.clone(),
                jvms: jvmnames,
                currentjavaname: currentjavaname.to_string(),
                gamemodelinux: p["gamemodelinux"].as_bool().unwrap(),
                currentworkingdirectory: p["currentworkingdirectory"].to_string(),
                pdirectories: directorynames,
                directoryprofile: currentdirectoryprofile,
                ..Default::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("SigLauncher")
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::LaunchPressed => {
                if self.version.as_ref().unwrap().is_empty() == false {
                    updateusersettingsfile(
                        self.username.to_owned(),
                        self.version.as_ref().unwrap().to_owned(),
                    )
                    .unwrap();

                    let username = self.username.clone();
                    let version = Some(self.version.clone());
                    let java = self.jvm.clone();
                    let jvmargss = java[2].replace("\"", "");
                    let jvmargsvec = jvmargss.split(' ').map(|s| s.to_owned()).collect();
                    let ram = self.ram.clone();
                    let gamemode = self.gamemodelinux;
                    let dprofile = self.directoryprofile.clone();
                    println!("{}", dprofile[1]);
                    let dprofilepath = dprofile[1].replace("\"", "");

                    Command::perform(
                        async move {
                            backend::start(
                                username.as_str(),
                                version.unwrap().expect("a").as_str(),
                                &java[1].replace("\"", "").as_str(),
                                jvmargsvec,
                                ram,
                                gamemode,
                                dprofilepath,
                            )
                            .await
                        },
                        Message::Launched,
                    )
                } else {
                    println!("You need to select a version!");
                    Command::none()
                }
            }
            Message::UserChanged(username) => {
                self.username = username;
                Command::none()
            }
            Message::VerChanged(version) => {
                self.version = Some(version);
                Command::none()
            }
            Message::Launched(_) => {
                println!("Backend finished.");
                Command::none()
            }

            Message::InstallPressed => {
                self.screen = 2;
                Command::perform(
                    async move { backend::version_installer::getversionlist() },
                    Message::Gotlist,
                )
            }
            Message::OptionsPressed => {
                self.screen = 3;
                Command::none()
            }
            Message::RamChanged(ram) => {
                self.ram = ram;
                Command::none()
            }
            Message::Apply => {
                updatesettingsfile(
                    self.ram,
                    self.currentjavaname.clone(),
                    self.currentworkingdirectory.clone(),
                )
                .unwrap();
                self.screen = 1;
                Command::none()
            }
            Message::Return => {
                self.versions = backend::getinstalledversions();
                self.screen = 1;
                Command::none()
            }

            Message::Gotlist(a) => {
                for i in a {
                    let ii = i.replace("\"", "");
                    self.downloadlist.push(ii);
                }
                Command::none()
            }
            Message::DownloadChanged(a) => {
                self.versiontodownload = a;
                Command::none()
            }
            Message::InstallVersion => {
                let ver = self.versiontodownload.clone().replace("\"", "");
                Command::perform(
                    async move { backend::version_installer::installversion(ver).unwrap_or(()) },
                    Message::Downloaded,
                )
            }
            Message::Downloaded(_) => {
                println!("Version installed successfully.");
                self.screen = 1;
                Command::none()
            }
            Message::GoJavaMan => {
                self.screen = 4;
                Command::none()
            }
            Message::JVMname(value) => {
                self.jvmaddname = value;
                Command::none()
            }
            Message::JVMpath(value) => {
                self.jvmaddpath = value;
                Command::none()
            }
            Message::JVMflags(value) => {
                self.jvmaddflags = value;
                Command::none()
            }
            Message::AddJVM => {
                if self.jvmaddname.is_empty() == false
                    && self.jvmaddpath.is_empty() == false
                    && self.jvmaddflags.is_empty() == false
                {
                    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

                    let mut file = File::open("launchsettings.json").unwrap();
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();

                    let mut data: Value = serde_json::from_str(&contents).unwrap();
                    let jvms = JVMs {
                        name: self.jvmaddname.clone(),
                        path: self.jvmaddpath.clone(),
                        flags: self.jvmaddflags.clone(),
                    };
                    if let Value::Array(arr) = &mut data["JVMs"] {
                        arr.push(serde_json::json!(jvms));
                        data["JVMs"] = serde_json::json!(arr)
                    }

                    let mut updatedjvmlist = Vec::new();

                    if let Some(jvms) = data["JVMs"].as_array() {
                        for jvm in jvms {
                            updatedjvmlist.push(jvm["name"].to_string());
                        }
                    }
                    self.jvms = updatedjvmlist;
                    let serialized = serde_json::to_string_pretty(&data).unwrap();

                    let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open("launchsettings.json")
                        .unwrap();
                    file.write_all(serialized.as_bytes()).unwrap();
                    self.screen = 3;
                    Command::none()
                } else {
                    println!("You need to fill the required fields!");
                    Command::none()
                }
            }
            Message::JVMChanged(value) => {
                set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

                let mut file = File::open("launchsettings.json").unwrap();
                let mut fcontent = String::new();
                file.read_to_string(&mut fcontent).unwrap();
                let content = serde_json::from_str(&fcontent);
                let p: Value = content.unwrap();

                let mut newjvm: Vec<String> = Vec::new();
                let mut newjvmname: String = String::new();

                if let Some(jvms) = p["JVMs"].as_array() {
                    for jvm in jvms {
                        if jvm["name"] == value.replace("\"", "") {
                            newjvm.push(jvm["name"].to_string());
                            newjvm.push(jvm["path"].to_string());
                            newjvm.push(jvm["flags"].to_string());

                            newjvmname = jvm["name"].to_string();
                        }
                    }
                }

                self.currentjavaname = newjvmname;
                self.jvm = newjvm;
                Command::none()
            }
            Message::GamemodeChanged(bool) => {
                self.gamemodelinux = bool;
                Command::none()
            }
            Message::Directoryname(name) => {
                self.daddname = name;
                Command::none()
            }
            Message::Directorypath(dpath) => {
                self.daddpath = dpath;
                Command::none()
            }
            Message::AddDirectory => {
                if self.daddname.is_empty() == false && self.daddpath.is_empty() == false {
                    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

                    let mut file = File::open("launchsettings.json").unwrap();
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();

                    let mut data: Value = serde_json::from_str(&contents).unwrap();
                    let directoryprofile = GameWorkingDirectory {
                        name: self.daddname.clone(),
                        path: self.daddpath.clone(),
                    };
                    if let Value::Array(arr) = &mut data["Game working directories"] {
                        arr.push(serde_json::json!(directoryprofile));
                        data["Game working directories"] = serde_json::json!(arr)
                    }

                    let mut updateddirectorieslist = Vec::new();

                    if let Some(directories) = data["Game working directories"].as_array() {
                        for directory in directories {
                            updateddirectorieslist.push(directory["name"].to_string());
                        }
                    }
                    self.pdirectories = updateddirectorieslist;
                    let serialized = serde_json::to_string_pretty(&data).unwrap();

                    let mut file = OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .open("launchsettings.json")
                        .unwrap();
                    file.write_all(serialized.as_bytes()).unwrap();
                    self.screen = 3;
                    Command::none()
                } else {
                    println!("You need to fill the required fields!");
                    Command::none()
                }
            }
            Message::DProfileChanged(value) => {
                set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

                let mut file = File::open("launchsettings.json").unwrap();
                let mut fcontent = String::new();
                file.read_to_string(&mut fcontent).unwrap();
                let content = serde_json::from_str(&fcontent);
                let p: Value = content.unwrap();

                let mut newprofile: Vec<String> = Vec::new();
                let mut newprofilename: String = String::new();

                if let Some(dprofiles) = p["Game working directories"].as_array() {
                    for dprofile in dprofiles {
                        if dprofile["name"] == value.replace("\"", "") {
                            newprofile.push(dprofile["name"].to_string());
                            newprofile.push(dprofile["path"].to_string());

                            newprofilename = dprofile["name"].to_string();
                        }
                    }
                }

                self.currentworkingdirectory = newprofilename;
                self.directoryprofile = newprofile;
                Command::none()
            }
            Message::GoDprofileMan => {
                self.screen = 5;
                Command::none()
            }
            Message::GithubPressed => {
                webbrowser::open("github.com/jafkc/siglauncher").unwrap();
                Command::none()
            },
        }
    }
    fn view(&self) -> Element<Message> {
        //used in mainscreen
        let title = text("SigLauncher")
            .size(50)
            .horizontal_alignment(alignment::Horizontal::Center);
        let userinput = text_input("Username", &self.username)
            .on_input(Message::UserChanged)
            .size(25)
            .width(250);
        let verpicker = pick_list(
            &self.versions,
            Some(format!("{:?}", &self.version.as_ref().unwrap())).map(|s| s.replace("\"", "")),
            Message::VerChanged,
        )
        .placeholder("Select a version")
        .width(250)
        .text_size(25);
        let launchlabel = text("Launch")
            .size(30)
            .horizontal_alignment(alignment::Horizontal::Center);
        let launchbutton = button(launchlabel)
            .width(275)
            .height(40)
            .on_press(Message::LaunchPressed);
        let verinstalllabel = text("Version installer")
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Center);
        let verinstallbutton = button(verinstalllabel)
            .width(250)
            .height(30)
            .on_press(Message::InstallPressed)
            .style(theme::Button::Secondary);
        let optionslabel = text("Options")
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Center);
        let optionsbutton = button(optionslabel)
            .width(250)
            .height(30)
            .on_press(Message::OptionsPressed);
        let githubhandle = svg::Handle::from_memory(include_bytes!("icons/github.svg").as_slice());
        let githubsvg = svg(githubhandle)
            .width(Length::Fixed(30.))
            .height(Length::Fixed(30.));
        let githubbutton = button(githubsvg).on_press(Message::GithubPressed);
        //options
        let otitle = text("Options")
            .size(50)
            .horizontal_alignment(alignment::Horizontal::Center);
        let javaoptions = column![
            text("JVM:").horizontal_alignment(alignment::Horizontal::Center),
            pick_list(
                &self.jvms,
                Some(&self.currentjavaname).map(|s| s.replace("\"", "")),
                Message::JVMChanged
            )
            .width(250)
            .text_size(25),
            button(
                text("Manage JVMS")
                    .size(20)
                    .width(250)
                    .horizontal_alignment(alignment::Horizontal::Center)
            )
            .height(30)
            .on_press(Message::GoJavaMan)
        ]
        .spacing(10)
        .max_width(800)
        .align_items(Alignment::Center);

        let directoryprofileoptions = column![
            text("Profile folder:").horizontal_alignment(alignment::Horizontal::Center),
            pick_list(
                &self.pdirectories,
                Some(&self.currentworkingdirectory).map(|s| s.replace("\"", "")),
                Message::DProfileChanged
            )
            .width(250)
            .text_size(25),
            button(
                text("Manage profile folders")
                    .size(20)
                    .width(250)
                    .horizontal_alignment(alignment::Horizontal::Center)
            )
            .height(30)
            .on_press(Message::GoDprofileMan)
        ]
        .spacing(10)
        .max_width(800)
        .align_items(Alignment::Center);
        let mut java_dprofiles_row = Row::new().spacing(50);
        java_dprofiles_row = java_dprofiles_row.push(javaoptions);
        java_dprofiles_row = java_dprofiles_row.push(directoryprofileoptions);

        let ramslider = slider(0.5..=16.0, self.ram, Message::RamChanged)
            .width(250)
            .step(0.5);
        let ramlabel = text(format!("Allocated memory: {}GiB", self.ram))
            .size(30)
            .horizontal_alignment(alignment::Horizontal::Center);
        let applytext = text("Save")
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Center);
        let applybutton = button(applytext)
            .width(135)
            .height(30)
            .on_press(Message::Apply);
        let returntext = text("Return")
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Center);
        let returnbutton = button(returntext)
            .width(135)
            .height(30)
            .on_press(Message::Return);
        let mut orow = Row::new().spacing(50);
        orow = orow.push(returnbutton);
        orow = orow.push(applybutton);
        let gamemodetext = text("Use Feral's GameMode (Linux only)")
            .horizontal_alignment(alignment::Horizontal::Center);
        let gamemode = toggler(String::new(), self.gamemodelinux, Message::GamemodeChanged)
            .width(Length::Shrink);
        let mut grow = Row::new().spacing(10);
        grow = grow.push(gamemode);
        grow = grow.push(gamemodetext);

        //installer
        let ititle = text("Version installer")
            .size(50)
            .horizontal_alignment(alignment::Horizontal::Center);
        let installpicker = pick_list(
            self.downloadlist.clone(),
            Some(format!("{:?}", &self.versiontodownload)).map(|s| s.replace("\"", "")),
            Message::DownloadChanged,
        )
        .placeholder("Select a version")
        .width(250)
        .text_size(25);
        let installlabel = text("Install")
            .size(30)
            .horizontal_alignment(alignment::Horizontal::Center);
        let installbutton = button(installlabel)
            .width(250)
            .height(40)
            .on_press(Message::InstallVersion);
        let ireturntext = text("Return")
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Center);
        let ireturnbutton = button(ireturntext)
            .width(250)
            .height(30)
            .on_press(Message::Return);
        //java manager
        let jtitle = text("Add JVM")
            .size(50)
            .horizontal_alignment(alignment::Horizontal::Center);
        let anameinput = text_input("", &self.jvmaddname)
            .on_input(Message::JVMname)
            .size(25)
            .width(250);
        let apathinput = text_input("", &self.jvmaddpath)
            .on_input(Message::JVMpath)
            .size(25)
            .width(250);
        let aflagsinput = text_input("", &self.jvmaddflags)
            .on_input(Message::JVMflags)
            .size(25)
            .width(250);
        let addtext = text("Add")
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Center);
        let addbutton = button(addtext)
            .width(135)
            .height(30)
            .on_press(Message::AddJVM);
        let jreturntext = text("Return")
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Center);
        let jreturnbutton = button(jreturntext)
            .width(135)
            .height(30)
            .on_press(Message::Return);
        let mut jrow = Row::new().spacing(50);
        jrow = jrow.push(jreturnbutton);
        jrow = jrow.push(addbutton);
        //directorymanager
        let dtitle = text("Add Directory")
            .size(50)
            .horizontal_alignment(alignment::Horizontal::Center);
        let dnameinput = text_input("", &self.daddname)
            .on_input(Message::Directoryname)
            .size(25)
            .width(250);
        let dpathinput = text_input("", &self.daddpath)
            .on_input(Message::Directorypath)
            .size(25)
            .width(250);
        let daddtext = text("Add")
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Center);
        let daddbutton = button(daddtext)
            .width(135)
            .height(30)
            .on_press(Message::AddDirectory);
        let dreturntext = text("Return")
            .size(20)
            .horizontal_alignment(alignment::Horizontal::Center);
        let dreturnbutton = button(dreturntext)
            .width(135)
            .height(30)
            .on_press(Message::Return);
        let mut drow = Row::new().spacing(50);
        drow = drow.push(dreturnbutton);
        drow = drow.push(daddbutton);
        let content;
        match self.screen {
            1 => {
                content = column![
                    title,
                    column![text("Username:"), userinput, text("Version:"), verpicker].spacing(5),
                    launchbutton,
                    optionsbutton,
                    verinstallbutton,
                    githubbutton
                ]
                .spacing(15)
                .max_width(800)
                .align_items(Alignment::Center)
            }
            2 => {
                content = column![ititle, installpicker, installbutton, ireturnbutton,]
                    .spacing(15)
                    .max_width(800)
                    .align_items(Alignment::Center)
            }
            3 => {
                content = column![otitle, java_dprofiles_row, ramlabel, ramslider, grow, orow,]
                    .spacing(25)
                    .max_width(800)
                    .align_items(Alignment::Center)
            }
            4 => {
                content = column![
                    jtitle,
                    text("JVM name:"),
                    anameinput,
                    text("JVM path:"),
                    apathinput,
                    text("JVM flags:"),
                    aflagsinput,
                    jrow,
                ]
                .spacing(15)
                .max_width(800)
                .align_items(Alignment::Center)
            }
            5 => {
                content = column![
                    dtitle,
                    text("Directory profile name:"),
                    dnameinput,
                    text("Directory profile path:"),
                    dpathinput,
                    drow,
                ]
                .spacing(15)
                .max_width(800)
                .align_items(Alignment::Center)
            }
            _ => panic!("Crashed"),
        }

        container(content)
            .width(Length::Fill)
            .padding(20)
            .center_x()
            .into()
    }
}

fn checksettingsfile() {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    if Path::new("launchsettings.json").exists() == false {
        let launchsettings = Siglauncher {
            username: "Player".to_string(),
            version: Some(String::new()),
            ram: 2.5,
            currentjavaname: "Default".to_string(),
            gamemodelinux: false,
            currentworkingdirectory: "Default".to_string(),
            ..Default::default()
        };

        let jvm = vec![
            JVMs{name:"Default".to_string(),path:"java".to_string(),flags:"-XX:+UnlockExperimentalVMOptions -XX:+UseG1GC -XX:G1NewSizePercent=20 -XX:G1ReservePercent=20 -XX:MaxGCPauseMillis=50 -XX:G1HeapRegionSize=16M".to_string()}
        ];

        let gamedirectories = vec![GameWorkingDirectory {
            name: "Default".to_string(),
            path: String::new(),
        }];
        let mut json =
            serde_json::json!({"JVMs" : jvm, "Game working directories": gamedirectories});

        if let Value::Object(map) = &mut json {
            map.insert(
                "username".to_owned(),
                serde_json::to_value(launchsettings.username).unwrap(),
            );
            map.insert(
                "version".to_owned(),
                serde_json::to_value(launchsettings.version).unwrap(),
            );
            map.insert(
                "ram".to_owned(),
                serde_json::to_value(launchsettings.ram).unwrap(),
            );
            map.insert(
                "currentjavaname".to_owned(),
                serde_json::to_value(launchsettings.currentjavaname).unwrap(),
            );
            map.insert(
                "gamemodelinux".to_owned(),
                serde_json::to_value(launchsettings.gamemodelinux).unwrap(),
            );
            map.insert(
                "currentworkingdirectory".to_owned(),
                serde_json::to_value(launchsettings.currentworkingdirectory).unwrap(),
            );
        }

        let serializedjson = serde_json::to_string_pretty(&json).unwrap();

        let mut file = File::create("launchsettings.json").unwrap();
        file.write_all(serializedjson.as_bytes()).unwrap();
        println!("New Json file created.")
    }
}

fn updateusersettingsfile(username: String, version: String) -> std::io::Result<()> {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    let mut file = File::open("launchsettings.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = serde_json::from_str(&contents)?;

    data["username"] = serde_json::Value::String(username);
    data["version"] = serde_json::Value::String(version);

    let serialized = serde_json::to_string_pretty(&data)?;

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("launchsettings.json")?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}

fn updatesettingsfile(
    ram: f64,
    currentjvm: String,
    currentworkingdirectory: String,
) -> std::io::Result<()> {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    let mut file = File::open("launchsettings.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = serde_json::from_str(&contents)?;

    data["ram"] = serde_json::Value::Number(Number::from_f64(ram).unwrap());
    data["currentjavaname"] = serde_json::Value::String(currentjvm.replace("\"", ""));
    data["currentworkingdirectory"] =
        serde_json::Value::String(currentworkingdirectory.replace("\"", ""));

    let serialized = serde_json::to_string_pretty(&data)?;

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("launchsettings.json")?;
    file.write_all(serialized.as_bytes())?;

    Ok(())
}

mod widget {
    use crate::theme::Theme;

    pub type Renderer = iced::Renderer<Theme>;
    pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
}
