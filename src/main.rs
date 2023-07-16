use iced::widget::{
    button, column, container, pick_list, row, slider, svg, text, text_input, toggler,
};
use iced::{alignment, executor, window, Alignment, Application, Command, Length, Settings};
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};
use std::env::{self, set_current_dir};
use std::fmt::Debug;
use std::fs::{self, File, OpenOptions};
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

#[derive(Default, Serialize, Deserialize)]
struct Siglauncher {
    username: String,
    version: Option<String>,
    ram: f64,
    currentjavaname: String,
    gamemodelinux: bool,
    currentprofilefolder: String,
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
    fabricdownloadlist: Vec<String>,
    #[serde(skip_serializing)]
    versiontodownload: String,
    #[serde(skip_serializing)]
    fabricversiontodownload: String,
    #[serde(skip_serializing)]
    pdirectories: Vec<String>,

    //add jvm
    jvmaddname: String,
    jvmaddpath: String,
    jvmaddflags: String,
    //add directory profile
    daddname: String,
    //
    state: String,

    showallversions: bool,
}

#[tokio::main]
async fn main() -> iced::Result {
    checksettingsfile();
    let icon = include_bytes!("icons/siglaunchericon.png");

    Siglauncher::run(Settings {
        window: window::Settings {
            size: (800, 450),

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
    OpenGameFolder,
    GithubPressed,
    InstallationScreenButton,

    GoJavaMan,
    GoDprofileMan,
    Launched(String),
    Gotlist(Vec<Vec<String>>),
    DownloadChanged(String),
    FabricDownloadChanged(String),

    RamChanged(f64),
    Return(i8),
    JVMChanged(String),
    ProfileFChanged(String),
    GamemodeChanged(bool),
    ShowVersionsChanged(bool),

    InstallVersion(u8),
    Downloaded(String),

    JVMname(String),
    JVMpath(String),
    JVMflags(String),
    AddJVM,

    Directoryname(String),
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
                jvmnames.push(jvm["name"].as_str().unwrap().to_owned());
                if jvm["name"] == p["currentjavaname"] {
                    currentjvm.push(jvm["name"].as_str().unwrap().to_owned());
                    currentjvm.push(jvm["path"].as_str().unwrap().to_owned());
                    currentjvm.push(jvm["flags"].as_str().unwrap().to_owned());
                }
            }
        }
        let currentjavaname = &currentjvm[0];

        let profileslocation = format!("{}/siglauncher_profiles", backend::get_minecraft_dir());

        if !Path::new(&profileslocation).exists() {
            fs::create_dir_all(&profileslocation).unwrap()
        }

        let entries = fs::read_dir(profileslocation).unwrap();

        let mut directorynames = entries
            .filter_map(|entry| {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    Some(path.file_name().unwrap().to_string_lossy().to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        directorynames.push("Default".to_string());

        (
            Siglauncher {
                username: p["username"].as_str().unwrap().to_owned(),
                version: Some(p["version"].as_str().unwrap().to_owned()),
                screen: 1,
                versions: backend::getinstalledversions(),
                ram: p["ram"].as_f64().unwrap(),
                jvm: currentjvm.clone(),
                jvms: jvmnames,
                currentjavaname: currentjavaname.to_string(),
                gamemodelinux: p["gamemodelinux"].as_bool().unwrap(),
                showallversions: p["showallversions"].as_bool().unwrap(),
                currentprofilefolder: p["currentprofilefolder"].as_str().unwrap().to_owned(),
                pdirectories: directorynames,
                ..Default::default()
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Siglauncher")
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::LaunchPressed => {
                if !self.version.as_ref().unwrap().is_empty() {
                    updateusersettingsfile(
                        self.username.to_owned(),
                        self.version.as_ref().unwrap().to_owned(),
                    )
                    .unwrap();

                    let username = self.username.clone();
                    let version = Some(self.version.clone());
                    let java = self.jvm.clone();
                    let jvmargss = java[2].clone();
                    let jvmargsvec = jvmargss.split(' ').map(|s| s.to_owned()).collect();
                    let ram = self.ram;
                    let gamemode = self.gamemodelinux;
                    let profilefolder = self.currentprofilefolder.clone();

                    let autojava = self.currentjavaname == "Automatic";

                    self.state = String::from("Launching...");

                    Command::perform(
                        async move {
                            match backend::start(
                                username.as_str(),
                                version.unwrap().expect("a").as_str(),
                                java[1].as_str(),
                                jvmargsvec,
                                ram,
                                gamemode,
                                profilefolder,
                                autojava,
                            ) {
                                Ok(()) => String::from("Launched!"),
                                Err(_) => String::from("Error! Game didn't launch"),
                            }
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
            Message::Launched(result) => {
                println!("Backend finished.");
                self.state = result;
                Command::none()
            }

            Message::InstallationScreenButton => {
                let showallversions = self.showallversions;
                self.screen = 2;
                Command::perform(
                    async move {
                        match backend::installer::getversionlist(showallversions) {
                            Ok(a) => a,
                            Err(_) => vec![],
                        }
                    },
                    Message::Gotlist,
                )
            }

            Message::RamChanged(ram) => {
                self.ram = ram;
                Command::none()
            }
            Message::Return(s) => {
                if self.screen == 3 {
                    updatesettingsfile(
                        self.ram,
                        self.currentjavaname.clone(),
                        self.currentprofilefolder.clone(),
                        self.gamemodelinux,
                        self.showallversions,
                    )
                    .unwrap();
                }
                self.screen = s;
                self.state.clear();
                Command::none()
            }

            Message::Gotlist(a) => {
                if a.is_empty() {
                    self.state = "Failed to get version list".to_string()
                } else {
                    self.downloadlist.clear();
                    self.fabricdownloadlist.clear();
                    for i in &a[0] {
                        let ii = i;
                        self.downloadlist.push(ii.to_string());
                    }
                    for i in &a[1] {
                        let ii = i;
                        self.fabricdownloadlist.push(ii.to_string());
                    }
                }

                Command::none()
            }
            Message::DownloadChanged(a) => {
                self.versiontodownload = a;
                Command::none()
            }

            Message::FabricDownloadChanged(a) => {
                self.fabricversiontodownload = a;
                Command::none()
            }

            Message::InstallVersion(versiontype) => {
                // 1 for vanilla, 2 for fabric and 3 for forge
                self.state = String::from("Downloading version...");
                let ver = match versiontype {
                    1 => self.versiontodownload.clone(),
                    2 => self.fabricversiontodownload.clone(),
                    _ => panic!("Version type doesn't exists!"),
                };
                Command::perform(
                    async move {
                        match backend::installer::installversion(ver, versiontype) {
                            Ok(()) => "Installed successfully".to_string(),
                            Err(_) => "An error ocurred and version was not installed".to_string(),
                        }
                    },
                    Message::Downloaded,
                )
            }
            Message::Downloaded(result) => {
                self.state = result;
                self.screen = 1;
                self.versions = backend::getinstalledversions();
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
                if !self.jvmaddname.is_empty()
                    && !self.jvmaddpath.is_empty()
                    && !self.jvmaddflags.is_empty()
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
                            updatedjvmlist.push(jvm["name"].as_str().unwrap().to_owned());
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
                        if jvm["name"] == value {
                            newjvm.push(jvm["name"].as_str().unwrap().to_owned());
                            newjvm.push(jvm["path"].as_str().unwrap().to_owned());
                            newjvm.push(jvm["flags"].as_str().unwrap().to_owned());

                            newjvmname = jvm["name"].as_str().unwrap().to_owned();
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
            Message::AddDirectory => {
                if !self.daddname.is_empty() {
                    fs::create_dir_all(format!(
                        "{}/siglauncher_profiles/{}",
                        backend::get_minecraft_dir(),
                        self.daddname
                    ))
                    .expect("Failed to create directory!");

                    let entries = fs::read_dir(format!(
                        "{}/siglauncher_profiles",
                        backend::get_minecraft_dir()
                    ))
                    .unwrap();

                    let mut directorynames = entries
                        .filter_map(|entry| {
                            let path = entry.unwrap().path();
                            if path.is_dir() {
                                Some(path.file_name().unwrap().to_string_lossy().to_string())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    directorynames.push("Default".to_string());

                    self.pdirectories = directorynames;

                    self.screen = 3;
                    Command::none()
                } else {
                    println!("You need to fill the required fields!");
                    Command::none()
                }
            }
            Message::ProfileFChanged(value) => {
                self.currentprofilefolder = value;
                Command::none()
            }
            Message::GoDprofileMan => {
                self.screen = 5;
                Command::none()
            }
            Message::GithubPressed => {
                webbrowser::open("https://github.com/jafkc/siglauncher").unwrap();
                Command::none()
            }
            Message::ShowVersionsChanged(bool) => {
                self.showallversions = bool;
                Command::none()
            }
            Message::OpenGameFolder => {
                let mc_dir = match std::env::consts::OS {
                    "linux" => format!("{}/.minecraft", std::env::var("HOME").unwrap()),
                    "windows" => format!(
                        "{}/AppData/Roaming/.minecraft",
                        std::env::var("USERPROFILE").unwrap().replace('\\', "/")
                    ),
                    _ => panic!("System not supported."),
                };
                if !Path::new(&mc_dir).is_dir() && fs::create_dir_all(&mc_dir).is_err() {
                    println!("Failed to create game folder")
                }

                if let Err(err) = open::that(mc_dir) {
                    println!("Failed to open game folder: {}", err);
                }
                Command::none()
            }
        }
    }
    fn view(&self) -> Element<Message> {
        //sidebar
        let homehandle = svg::Handle::from_memory(include_bytes!("icons/home.svg").as_slice());
        let homesvg = svg(homehandle);
        let homebutton = button(homesvg)
            .on_press(Message::Return(1))
            .style(theme::Button::Transparent)
            .width(Length::Fixed(40.))
            .height(Length::Fixed(40.));

        let optionshandle =
            svg::Handle::from_memory(include_bytes!("icons/options.svg").as_slice());
        let optionssvg = svg(optionshandle);
        let optionsbutton = button(optionssvg)
            .on_press(Message::Return(3))
            .style(theme::Button::Transparent)
            .width(Length::Fixed(40.))
            .height(Length::Fixed(40.));

        let downloadhandle =
            svg::Handle::from_memory(include_bytes!("icons/download.svg").as_slice());
        let downloadsvg = svg(downloadhandle);
        let downloadbutton = button(downloadsvg)
            .on_press(Message::InstallationScreenButton)
            .style(theme::Button::Transparent)
            .width(Length::Fixed(40.))
            .height(Length::Fixed(40.));

        let profilehandle =
            svg::Handle::from_memory(include_bytes!("icons/profile.svg").as_slice());
        let profilesvg = svg(profilehandle);
        let profilebutton = button(profilesvg)
            .style(theme::Button::Transparent)
            .width(Length::Fixed(40.))
            .height(Length::Fixed(40.));

        let githubhandlea = svg::Handle::from_memory(include_bytes!("icons/github.svg").as_slice());
        let githubsvga = svg(githubhandlea)
            .width(Length::Fixed(30.))
            .height(Length::Fixed(30.));
        let githubbuttona = button(githubsvga)
            .on_press(Message::GithubPressed)
            .style(theme::Button::Transparent);

        let sidebar = column![
            homebutton,
            optionsbutton,
            downloadbutton,
            profilebutton,
            githubbuttona,
        ]
        .spacing(25)
        .align_items(Alignment::Center);
        let containersidebar = container(sidebar)
            .style(theme::Container::BlackContainer)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .width(50)
            .height(Length::Fill);

        let state = text(&self.state)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Bottom);

        let content = match self.screen {
            1 => column![
                //mainscreen
                //title
                column![
                    text("Siglauncher").size(65),
                    text(format!("Hello {}!", self.username)).style(theme::Text::Peach)
                ]
                .spacing(5),
                //username and version input
                row![
                container(
                column![
                    text("Username:"),
                    text_input("Username", &self.username)
                        .on_input(Message::UserChanged)
                        .size(25)
                        .width(285),
                    text("Version:"),
                    pick_list(
                        &self.versions,
                        self.version.clone(),
                        Message::VerChanged,
                    )
                    .placeholder("Select a version")
                    .width(285)
                    .text_size(25)
                ]
                .spacing(10)).style(theme::Container::BlackContainer).padding(10), container(column![
                    button(text("Open game folder").size(20).horizontal_alignment(alignment::Horizontal::Center)).width(250).height(30).on_press(Message::OpenGameFolder),
                ].spacing(10)).style(theme::Container::BlackContainer).padding(20)].spacing(15),
                //launchbutton
                button(
                    text("Launch")
                        .size(50)
                        .horizontal_alignment(alignment::Horizontal::Center)
                )
                .width(285)
                .height(60)
                .on_press(Message::LaunchPressed),
                state
            ]
            .spacing(25)
            .max_width(800),
            2 => column![
                //installerscreen
                //title
                text("Version installer").size(50),

                row![
                //vanilla
                container(
                column![
                    text("Vanilla"),
                pick_list(
                    self.downloadlist.clone(),
                    Some(self.versiontodownload.clone()),
                    Message::DownloadChanged,
                )
                .placeholder("Select a version")
                .width(250)
                .text_size(25),
                //installbutton
                button(
                    text("Install")
                        .size(30)
                        .horizontal_alignment(alignment::Horizontal::Center)
                )
                .width(250)
                .height(40)
                .on_press(Message::InstallVersion(1))
                .style(theme::Button::Secondary)].spacing(15)).style(theme::Container::BlackContainer).padding(10),

                //fabric
                container(
                    column![
                        text("Fabric"),
                    pick_list(
                        self.fabricdownloadlist.clone(),
                        Some(self.fabricversiontodownload.clone()),
                        Message::FabricDownloadChanged,
                    )
                    .placeholder("Select a version")
                    .width(250)
                    .text_size(25),
                    //installbutton
                    button(
                        text("Install")
                            .size(30)
                            .horizontal_alignment(alignment::Horizontal::Center)
                    )
                    .width(250)
                    .height(40)
                    .on_press(Message::InstallVersion(2))
                    .style(theme::Button::Secondary)].spacing(15)).style(theme::Container::BlackContainer).padding(10)].spacing(15),

                if !self.showallversions{
                    text("Enable the \"Show all versions in installer\" setting to download snapshots.").style(theme::Text::Green)
                } else{
                    text("")
                },
                state
            ]
            .spacing(15)
            .max_width(800),

            3 => column![
                //optionsscreen
                //title
                text("Options").size(50),
                //jvm and profile management
                row![
                    container(
                        column![
                            column![
                                text("JVM:").horizontal_alignment(alignment::Horizontal::Center),
                                pick_list(
                                    &self.jvms,
                                    Some(self.currentjavaname.clone()),
                                    Message::JVMChanged
                                )
                                .width(250)
                                .text_size(25),
                                button(
                                    text("Manage JVMs")
                                        .size(20)
                                        .width(250)
                                        .horizontal_alignment(alignment::Horizontal::Center)
                                )
                                .height(30)
                                .on_press(Message::GoJavaMan)
                            ]
                            .spacing(10)
                            .max_width(800)
                            .align_items(Alignment::Center),
                            column![
                                text("Profile folder:")
                                    .horizontal_alignment(alignment::Horizontal::Center),
                                pick_list(
                                    &self.pdirectories,
                                    Some(self.currentprofilefolder.clone()),
                                    Message::ProfileFChanged
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
                            .align_items(Alignment::Center)
                        ]
                        .spacing(10)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(10),
                    //memory, gamemode and showallversions option
                    container(
                        column![
                            column![
                                text(format!("Allocated memory: {}GiB", self.ram))
                                    .size(30)
                                    .horizontal_alignment(alignment::Horizontal::Center),
                                slider(0.5..=16.0, self.ram, Message::RamChanged)
                                    .width(250)
                                    .step(0.5)
                            ],
                            row![
                                text("Show all versions in installer")
                                    .horizontal_alignment(alignment::Horizontal::Center),
                                toggler(
                                    String::new(),
                                    self.showallversions,
                                    Message::ShowVersionsChanged
                                )
                                .width(Length::Shrink)
                            ]
                            .spacing(10),
                            row![
                                text("Use Feral GameMode (Linux only)")
                                    .horizontal_alignment(alignment::Horizontal::Center),
                                toggler(
                                    String::new(),
                                    self.gamemodelinux,
                                    Message::GamemodeChanged
                                )
                                .width(Length::Shrink)
                            ]
                            .spacing(10)
                        ]
                        .spacing(50)
                    )
                    .style(theme::Container::BlackContainer)
                    .padding(10)
                ]
                .spacing(15),
            ]
            .spacing(15)
            .max_width(800),

            4 => column![
                text("Manage JVMs")
                    .size(50)
                    .horizontal_alignment(alignment::Horizontal::Center),
                container(column![
                text("Add"),
                text("JVM name:"),
                text_input("", &self.jvmaddname)
                    .on_input(Message::JVMname)
                    .size(25)
                    .width(250),
                text("JVM path:"),
                text_input("", &self.jvmaddpath)
                    .on_input(Message::JVMpath)
                    .size(25)
                    .width(250),
                text("JVM flags:"),
                text_input("", &self.jvmaddflags)
                    .on_input(Message::JVMflags)
                    .size(25)
                    .width(250),
                button(
                    text("Add")
                        .size(20)
                        .horizontal_alignment(alignment::Horizontal::Center)
                )
                .width(135)
                .height(30)
                .on_press(Message::AddJVM)].spacing(5)).style(theme::Container::BlackContainer).padding(15)
            ]
            .spacing(15)
            .max_width(800),
            5 => column![
                text("Manage directory profiles")
                    .size(50)
                    .horizontal_alignment(alignment::Horizontal::Center),
                container(
                    column![
                text("Add"),
                text("Directory profile name:"),
                text_input("", &self.daddname)
                    .on_input(Message::Directoryname)
                    .size(25)
                    .width(250),
                button(
                    text("Add")
                        .size(20)
                        .horizontal_alignment(alignment::Horizontal::Center)
                )
                .width(135)
                .height(30)
                .on_press(Message::AddDirectory)].spacing(15)).style(theme::Container::BlackContainer).padding(15)
            ]
            .spacing(15)
            .max_width(800),

            _ => panic!("Crashed"),
        };

        container(row![containersidebar, content].spacing(65))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(alignment::Vertical::Center)
            .padding(20)
            .into()
    }
}

fn checksettingsfile() {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    if !Path::new("launchsettings.json").exists() {
        let launchsettings = Siglauncher {
            username: "Player".to_string(),
            version: Some(String::new()),
            ram: 2.5,
            currentjavaname: "Automatic".to_string(),
            gamemodelinux: false,
            currentprofilefolder: "Default".to_string(),
            showallversions: false,
            ..Default::default()
        };

        let jvm = vec![
            JVMs{name: "Automatic".to_string(), path: String::new(), flags: String::new()},
            JVMs{name:"System Java".to_string(),path:"java".to_string(),flags:"-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+UseNUMA -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=400M -XX:NonNMethodCodeHeapSize=12M -XX:ProfiledCodeHeapSize=194M -XX:NonProfiledCodeHeapSize=194M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseVectorCmov -XX:+PerfDisableSharedMem -XX:+UseFastUnorderedTimeStamps -XX:+UseCriticalJavaThreadPriority -XX:ThreadPriorityPolicy=1 -XX:AllocatePrefetchStyle=3".to_string()}
        ];

        let mut json = serde_json::json!({ "JVMs": jvm });

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
                "currentprofilefolder".to_owned(),
                serde_json::to_value(launchsettings.currentprofilefolder).unwrap(),
            );
            map.insert(
                "showallversions".to_owned(),
                serde_json::to_value(launchsettings.showallversions).unwrap(),
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
    currentprofilefolder: String,
    gamemode: bool,
    showallversions: bool,
) -> std::io::Result<()> {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    let mut file = File::open("launchsettings.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = serde_json::from_str(&contents)?;

    data["ram"] = serde_json::Value::Number(Number::from_f64(ram).unwrap());
    data["currentjavaname"] = serde_json::Value::String(currentjvm);
    data["currentprofilefolder"] = serde_json::Value::String(currentprofilefolder);
    data["gamemodelinux"] = serde_json::Value::Bool(gamemode);
    data["showallversions"] = serde_json::Value::Bool(showallversions);

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
