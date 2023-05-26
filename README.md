# Siglauncher
This is Siglauncher, a Minecraft launcher made with Rust and the Iced GUI library. 
Siglauncher supports both Vanilla and Fabric and works only with Windows and Linux.

![image](https://github.com/JafKc/siglauncher/assets/109480612/48401a3d-0e08-4843-9f71-a75145661eea)


### Installation
##### Build method
Requires Git and Rust to be installed. Type the following commands:

```bash
git clone https://github.com/JafKc/siglauncher.git
```
```bash
cd siglauncher
```
```bash
cargo build --release
```
The executable will appear inside **target/release**.

##### Release method
Download from [releases](https://github.com/JafKc/siglauncher/releases).

Releases may be outdated and lack newer features.

#### Java warning
Siglauncher doesn't provide Java, you can download Java from the following links:

[java 17](https://adoptium.net/temurin/releases/?version=17), for newer versions of Minecraft

[java 8](https://adoptium.net/temurin/releases/?version=8), for older versions (before 1.15)

#### GameMode warning (Linux)
To make Feral's GameMode setting work, you need to have gamemode installed. To install it, type one of the following commands in your terminal:

For Arch-based distros: 
```bash
sudo pacman -S gamemode lib32-gamemode
```
For Debian and Ubuntu-based distros:
```bash
sudo apt install gamemode
```
For Fedora-based distros:
```bash
dnf install gamemode
```


