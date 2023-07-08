# Siglauncher
This is Siglauncher, a Minecraft launcher made with Rust and the Iced GUI library. 
The launcher is compatible with Vanilla, Fabric, and Forge, and is designed to run on both Windows and Linux.


### Features
* Simple and intuitive GUI
* Version installer
* Compatibility: works with any vanilla release, Fabric and Forge.
* Game performance: optimized Java flags and Feral's Gamemode (Linux).
* Works in offline mode.

![image](https://github.com/JafKc/siglauncher/assets/109480612/fcf86c91-48db-44c9-8600-16657e6d7b79)


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

You can download Fabric versions from the launcher. If you want to use Forge then download it from [here](https://files.minecraftforge.net/net/minecraftforge/forge/).


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
