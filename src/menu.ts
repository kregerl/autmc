export enum MenuId {
    Instances,
    Screenshots,
    Logs,
    Servers,
}

export enum OpenModalType {
    None,
    SideMenu,
    Settings,
}

export enum InstanceType {
    Vanilla,
    Curseforge,
    Modrinth,
    Zip
}

export enum ModloaderType {
    None,
    Fabric,
    Forge
}

export function modloaderTypeToString(type: ModloaderType): string {
    switch (type) {
        case ModloaderType.Fabric: return "Fabric";
        case ModloaderType.Forge: return "Forge";
        default: return "";
    }
}

export enum Emphasis {
    High,
    Medium,
    Low
}

export function classFromEmphasis(emphasis: Emphasis): string {
    switch (emphasis) {
        case Emphasis.High: return "high-emphasis"
        case Emphasis.Medium: return "medium-emphasis"
        case Emphasis.Low: return "low-emphasis"
    }
}