# Autm Minecraft Launcher 
***Temporary Name***

A Minecraft launcher in its early statges of development.   
Autm is written in **Rust** + **Svelte** using tauri and aims to be a fast, reliable launcher that provides enough tooling to make nearly any change to an instance without leaving the launcher. 

## Features

- Multiple Instances
- Download and install Fabric
- View game logs in launcher

## Development
There are a few environment variables that can be set to enable logging of different launcher modules.
All of these environment variables can be enabled by setting them to '1'.
The following is a list of environment variables and what they enable/disable:
|Environment Variable|Description|
|---|---|
|DEBUG|Enables debug logging|
|REQWEST_DEBUG| Enables debug logging for the `reqwest` crate
|AUTHENTICATION|Enables debug logging for sensitive information related to authentication|