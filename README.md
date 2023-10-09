# Siglauncher
This is Siglauncher, a Minecraft launcher made with Rust and the Iced GUI library. 
The launcher is compatible with Vanilla, Fabric, and Forge, and is designed to run on both Windows and Linux.

Note: For now the launcher only works in offline mode.


### Features
* Simple and intuitive GUI
* Version installer
* Compatibility: works with any vanilla release, Fabric and Forge
* Game performance: optimized Java flags
* Works in offline mode
* No need to install Java, the launcher provides both Java 8 and Java 17

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


### Wrapper commands
tip: if you use linux and have GameMode installed, you can add "gamemoderun" to wrapper commands to improve game performance.
