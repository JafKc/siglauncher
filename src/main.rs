use iced::widget::{
    button, column, container, pick_list, row, slider, svg, text, text_input, toggler, Row,
};
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
    versiontodownload: String,
    #[serde(skip_serializing)]
    pdirectories: Vec<String>,
    #[serde(skip_serializing)]
    profilefolder: Vec<String>,

    //add jvm
    jvmaddname: String,
    jvmaddpath: String,
    jvmaddflags: String,
    //add directory profile
    daddname: String,
    daddpath: String,

    state: String,
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
    GithubPressed,
    InstallationScreenButton,

    GoJavaMan,
    GoDprofileMan,
    Launched(Result<(), String>),
    Gotlist(Vec<String>),
    DownloadChanged(String),

    RamChanged(f64),
    Apply,
    Return(i8),
    JVMChanged(String),
    ProfileFChanged(String),
    GamemodeChanged(bool),

    InstallVersion,
    Downloaded(String),

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

        let mut currentprofilefolder = Vec::new();
        let mut directorynames: Vec<String> = Vec::new();

        if let Some(directories) = p["Game profile folders"].as_array() {
            for directory in directories {
                directorynames.push(directory["name"].to_string());
                if directory["name"] == p["currentprofilefolder"] {
                    currentprofilefolder.push(directory["name"].to_string());
                    currentprofilefolder.push(directory["path"].to_string());
                }
            }
        }

        (
            Siglauncher {
                username: p["username"].to_string().replace('\"', ""),
                version: Some(p["version"].to_string().replace('\"', "")),
                screen: 1,
                versions: backend::getinstalledversions(),
                ram: p["ram"].as_f64().unwrap(),
                jvm: currentjvm.clone(),
                jvms: jvmnames,
                currentjavaname: currentjavaname.to_string(),
                gamemodelinux: p["gamemodelinux"].as_bool().unwrap(),
                currentprofilefolder: p["currentprofilefolder"].to_string(),
                pdirectories: directorynames,
                profilefolder: currentprofilefolder,
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
                    let jvmargss = java[2].replace('\"', "");
                    let jvmargsvec = jvmargss.split(' ').map(|s| s.to_owned()).collect();
                    let ram = self.ram;
                    let gamemode = self.gamemodelinux;
                    let dprofile = self.profilefolder.clone();
                    println!("{}", dprofile[1]);
                    let dprofilepath = dprofile[1].replace('\"', "");

                    self.state = String::from("Launching...");

                    Command::perform(
                        async move {
                            backend::start(
                                username.as_str(),
                                version.unwrap().expect("a").as_str(),
                                java[1].replace('\"', "").as_str(),
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
            Message::Launched(result) => {
                println!("Backend finished.");
                if result.is_ok() {
                    self.state = String::from("Launched.");
                } else {
                    self.state = result.err().unwrap();
                }
                Command::none()
            }

            Message::InstallationScreenButton => {
                self.screen = 2;
                Command::perform(
                    async move {
                        match backend::version_installer::getversionlist() {
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
            Message::Apply => {
                updatesettingsfile(
                    self.ram,
                    self.currentjavaname.clone(),
                    self.currentprofilefolder.clone(),
                    self.gamemodelinux,
                )
                .unwrap();
                self.screen = 1;
                Command::none()
            }
            Message::Return(s) => {
                self.versions = backend::getinstalledversions();
                self.screen = s;
                self.state.clear();
                Command::none()
            }

            Message::Gotlist(a) => {
                if a.is_empty() {
                    self.state = "Failed to get version list".to_string()
                }

                if self.downloadlist.is_empty() {
                    for i in a {
                        let ii = i.replace('\"', "");
                        self.downloadlist.push(ii);
                    }
                }

                Command::none()
            }
            Message::DownloadChanged(a) => {
                self.versiontodownload = a;
                Command::none()
            }
            Message::InstallVersion => {
                self.state = String::from("Downloading version...");
                let ver = self.versiontodownload.clone().replace('\"', "");
                Command::perform(
                    async move {
                        match backend::version_installer::installversion(ver) {
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
                        if jvm["name"] == value.replace('\"', "") {
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
                if !self.daddname.is_empty() && !self.daddpath.is_empty() {
                    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

                    let mut file = File::open("launchsettings.json").unwrap();
                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();

                    let mut data: Value = serde_json::from_str(&contents).unwrap();
                    let profilefolder = GameWorkingDirectory {
                        name: self.daddname.clone(),
                        path: self.daddpath.clone(),
                    };
                    if let Value::Array(arr) = &mut data["Game profile folders"] {
                        arr.push(serde_json::json!(profilefolder));
                        data["Game profile folders"] = serde_json::json!(arr)
                    }

                    let mut updateddirectorieslist = Vec::new();

                    if let Some(directories) = data["Game profile folders"].as_array() {
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
            Message::ProfileFChanged(value) => {
                set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

                let mut file = File::open("launchsettings.json").unwrap();
                let mut fcontent = String::new();
                file.read_to_string(&mut fcontent).unwrap();
                let content = serde_json::from_str(&fcontent);
                let p: Value = content.unwrap();

                let mut newprofile: Vec<String> = Vec::new();
                let mut newprofilename: String = String::new();

                if let Some(dprofiles) = p["Game profile folders"].as_array() {
                    for dprofile in dprofiles {
                        if dprofile["name"] == value.replace('\"', "") {
                            newprofile.push(dprofile["name"].to_string());
                            newprofile.push(dprofile["path"].to_string());

                            newprofilename = dprofile["name"].to_string();
                        }
                    }
                }

                self.currentprofilefolder = newprofilename;
                self.profilefolder = newprofile;
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
            .on_press(Message::Return(3))
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

        //used in mainscreen
        let title = text("Siglauncher").size(65);
        let userinput = text_input("Username", &self.username)
            .on_input(Message::UserChanged)
            .size(25)
            .width(285);
        let verpicker = pick_list(
            &self.versions,
            Some(format!("{:?}", &self.version.as_ref().unwrap())).map(|s| s.replace('\"', "")),
            Message::VerChanged,
        )
        .placeholder("Select a version")
        .width(285)
        .text_size(25);
        let launchlabel = text("Launch")
            .size(50)
            .horizontal_alignment(alignment::Horizontal::Center);
        let launchbutton = button(launchlabel)
            .width(285)
            .height(60)
            .on_press(Message::LaunchPressed);

        //options
        let otitle = text("Options").size(50);
        let javaoptions = column![
            text("JVM:").horizontal_alignment(alignment::Horizontal::Center),
            pick_list(
                &self.jvms,
                Some(&self.currentjavaname).map(|s| s.replace('\"', "")),
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
        .align_items(Alignment::Center);

        let profilefolderoptions = column![
            text("Profile folder:").horizontal_alignment(alignment::Horizontal::Center),
            pick_list(
                &self.pdirectories,
                Some(&self.currentprofilefolder).map(|s| s.replace('\"', "")),
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
        .align_items(Alignment::Center);

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

        let gamemodetext = text("Use Feral's GameMode (Linux only)")
            .horizontal_alignment(alignment::Horizontal::Center);
        let gamemode = toggler(String::new(), self.gamemodelinux, Message::GamemodeChanged)
            .width(Length::Shrink);
        let mut grow = Row::new().spacing(10);
        grow = grow.push(gamemode);
        grow = grow.push(gamemodetext);

        //installer
        let ititle = text("Version installer").size(50);
        let installpicker = pick_list(
            self.downloadlist.clone(),
            Some(format!("{:?}", &self.versiontodownload)).map(|s| s.replace('\"', "")),
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
            .on_press(Message::InstallVersion)
            .style(theme::Button::Secondary);

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
            .on_press(Message::Return(3));
        let mut jrow = Row::new().spacing(25);
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
            .on_press(Message::Return(3));
        let mut drow = Row::new().spacing(25);
        drow = drow.push(dreturnbutton);
        drow = drow.push(daddbutton);

        let state = text(&self.state)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Bottom);
        let content = match self.screen {
            1 => row![
                containersidebar,
                column![
                    column![
                        title,
                        text(format!("Hello {}!", self.username)).style(theme::Text::Peach)
                    ]
                    .spacing(5),
                    column![text("Username:"), userinput, text("Version:"), verpicker].spacing(10),
                    launchbutton,
                    state
                ]
                .spacing(25)
                .max_width(800)
            ]
            .spacing(65),
            2 => row![
                containersidebar,
                column![ititle, installpicker, installbutton, state]
                    .spacing(15)
                    .max_width(800)
            ]
            .spacing(65),
            3 => row![
                containersidebar,
                column![
                    otitle,
                    row![
                        container(column![javaoptions, profilefolderoptions].spacing(10))
                            .style(theme::Container::BlackContainer)
                            .padding(10),
                        container(column![column![ramlabel, ramslider], grow].spacing(50))
                            .style(theme::Container::BlackContainer)
                            .padding(10)
                    ]
                    .spacing(15),
                    applybutton,
                ]
                .spacing(25)
                .max_width(800)
            ]
            .spacing(65),
            4 => row![
                containersidebar,
                column![
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
            ]
            .spacing(65),
            5 => row![
                containersidebar,
                column![
                    dtitle,
                    text("Directory profile name:"),
                    dnameinput,
                    text("Directory profile path:"),
                    dpathinput,
                    drow,
                ]
                .spacing(15)
                .max_width(800)
            ]
            .spacing(65),
            _ => panic!("Crashed"),
        };

        container(content)
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
            currentjavaname: "Default".to_string(),
            gamemodelinux: false,
            currentprofilefolder: "Default".to_string(),
            ..Default::default()
        };

        let jvm = vec![
            JVMs{name:"Default".to_string(),path:"java".to_string(),flags:"-XX:+UnlockExperimentalVMOptions -XX:+UnlockDiagnosticVMOptions -XX:+AlwaysActAsServerClassMachine -XX:+AlwaysPreTouch -XX:+DisableExplicitGC -XX:+UseNUMA -XX:NmethodSweepActivity=1 -XX:ReservedCodeCacheSize=400M -XX:NonNMethodCodeHeapSize=12M -XX:ProfiledCodeHeapSize=194M -XX:NonProfiledCodeHeapSize=194M -XX:-DontCompileHugeMethods -XX:MaxNodeLimit=240000 -XX:NodeLimitFudgeFactor=8000 -XX:+UseVectorCmov -XX:+PerfDisableSharedMem -XX:+UseFastUnorderedTimeStamps -XX:+UseCriticalJavaThreadPriority -XX:ThreadPriorityPolicy=1 -XX:AllocatePrefetchStyle=3".to_string()}
        ];

        let gamedirectories = vec![GameWorkingDirectory {
            name: "Default".to_string(),
            path: String::new(),
        }];
        let mut json = serde_json::json!({"JVMs" : jvm, "Game profile folders": gamedirectories});

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
) -> std::io::Result<()> {
    set_current_dir(env::current_exe().unwrap().parent().unwrap()).unwrap();

    let mut file = File::open("launchsettings.json")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut data: Value = serde_json::from_str(&contents)?;

    data["ram"] = serde_json::Value::Number(Number::from_f64(ram).unwrap());
    data["currentjavaname"] = serde_json::Value::String(currentjvm.replace('\"', ""));
    data["currentprofilefolder"] =
        serde_json::Value::String(currentprofilefolder.replace('\"', ""));
    data["gamemodelinux"] = serde_json::Value::Bool(gamemode);

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
