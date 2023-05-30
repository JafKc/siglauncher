# Siglauncher
This is Siglauncher, a Minecraft launcher made with Rust and the Iced GUI library. 
The launcher is compatible with Vanilla, Fabric, and Forge, and is designed to run on both Windows and Linux.


### Features
* Simple and intuitive GUI
* Version installer
* Compatibility: works with any vanilla release, Fabric and Forge.
* Game performance: optimized java flags and Feral's Gamemode(linux).
* Works in offline mode.

![image](https://github.com/JafKc/siglauncher/assets/109480612/48401a3d-0e08-4843-9f71-a75145661eea)


### Installation
###### Build method
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

###### Release method
Download from [releases](https://github.com/JafKc/siglauncher/releases).

Releases may be outdated and lack newer features.

### Mods
For mods, you can choose between [Fabric](https://fabricmc.net/) or [Forge](https://files.minecraftforge.net/net/minecraftforge/forge/). Download mods from [Mondrith](https://modrinth.com/mods) and paste them into the mods folder within your Minecraft directory.

### Java warning
Siglauncher doesn't provide Java, you can download Java from the following links:

[java 17](https://adoptium.net/temurin/releases/?version=17), for newer versions of Minecraft

[java 8](https://adoptium.net/temurin/releases/?version=8), for older versions (before 1.15)

### GameMode warning
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


Note: The launcher only works in offline mode; it is not yet possible to log in with a Microsoft account.
